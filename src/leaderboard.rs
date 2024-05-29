use crate::{
    prelude::{HealthConfig, Id, LevelModifiers},
    task::Task,
};

use ctl_client::{
    core::{prelude::Uuid, types::UserLogin, ScoreEntry, SubmitScore},
    Nertboard,
};
use geng::prelude::*;

#[derive(Debug)]
pub enum LeaderboardStatus {
    None,
    Pending,
    Failed,
    Done,
}

struct BoardUpdate {
    scores: Vec<ScoreEntry>,
}

pub struct Leaderboard {
    geng: Geng,
    /// Logged in as user with a name.
    pub user: Option<UserLogin>,
    pub client: Option<Arc<Nertboard>>,
    log_task: Option<Task<ctl_client::Result<Result<UserLogin, String>>>>,
    task: Option<Task<ctl_client::Result<BoardUpdate>>>,
    pub status: LeaderboardStatus,
    pub loaded: LoadedBoard,
}

impl Clone for Leaderboard {
    fn clone(&self) -> Self {
        Self {
            geng: self.geng.clone(),
            user: self.user.clone(),
            client: self.client.clone(),
            log_task: None,
            task: None,
            status: LeaderboardStatus::None,
            loaded: LoadedBoard {
                meta: self.loaded.meta.clone(),
                local_high: self.loaded.local_high.clone(),
                ..LoadedBoard::new()
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedScore {
    pub level: Id,
    pub score: i32,
    pub meta: ScoreMeta,
}

#[derive(Debug)]
pub struct LoadedBoard {
    pub level: Id,
    pub meta: ScoreMeta,
    pub player: Option<Id>,
    pub my_position: Option<usize>,
    pub all_scores: Vec<ScoreEntry>,
    pub filtered: Vec<ScoreEntry>,
    pub local_high: Option<SavedScore>,
}

/// Meta information saved together with the score.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoreMeta {
    pub version: u32,
    pub mods: LevelModifiers,
    pub health: HealthConfig,
}

impl ScoreMeta {
    pub fn new(mods: LevelModifiers, health: HealthConfig) -> Self {
        Self {
            version: 1,
            mods,
            health,
        }
    }
}

impl Leaderboard {
    pub fn new(geng: &Geng, client: Option<&Arc<Nertboard>>) -> Self {
        let mut leaderboard = Self {
            geng: geng.clone(),
            user: None,
            client: client.cloned(),
            log_task: None,
            task: None,
            status: LeaderboardStatus::None,
            loaded: LoadedBoard::new(),
        };
        leaderboard.relogin();
        leaderboard
    }

    pub fn is_online(&self) -> bool {
        self.client
            .as_ref()
            .map_or(false, |client| client.is_online())
    }

    pub fn login_discord(&mut self) {
        if self.log_task.is_some() {
            return;
        }

        if let Some(client) = &self.client {
            let client = Arc::clone(client);
            let future = async move {
                let state = Uuid::new_v4().to_string();
                let redirect_uri = client.url.join("auth/discord")?;
                let url = format!(
                    "{}&state={}&redirect_uri={}",
                    crate::DISCORD_URL,
                    state,
                    redirect_uri
                );
                if let Err(err) = webbrowser::open(&url) {
                    log::error!("failed to open login link: {:?}", err);
                    return Err(ctl_client::ClientError::Connection);
                }
                client.login_external(state).await
            };
            self.log_task = Some(Task::new(&self.geng, future));
            self.user = None;
        }
    }

    /// Attempt to login back using the saved credentials.
    pub fn relogin(&mut self) {
        if self.log_task.is_some() {
            return;
        }

        if let Some(client) = &self.client {
            let Some(user): Option<UserLogin> = preferences::load(crate::PLAYER_LOGIN_STORAGE)
            else {
                return;
            };

            let client = Arc::clone(client);
            let future = async move { client.login_token(user.id, &user.token).await };
            self.log_task = Some(Task::new(&self.geng, future));
            self.user = None;
        }
    }

    // pub fn login(&mut self, creds: Credentials) {
    //     if self.log_task.is_some() {
    //         return;
    //     }

    //     if let Some(client) = &self.client {
    //         let client = Arc::clone(client);
    //         let future = async move { client.login(&creds).await };
    //         self.log_task = Some(Task::new(&self.geng, future));
    //         self.user = None;
    //     }
    // }

    // pub fn register(&mut self, creds: Credentials) {
    //     if self.log_task.is_some() {
    //         return;
    //     }

    //     if let Some(client) = &self.client {
    //         let client = Arc::clone(client);
    //         let future = async move {
    //             if let Err(err) = client.register(&creds).await? {
    //                 return Ok(Err(err));
    //             }
    //             client.login(&creds).await
    //         };
    //         self.log_task = Some(Task::new(&self.geng, future));
    //         self.user = None;
    //     }
    // }

    pub fn logout(&mut self) {
        if self.log_task.is_some() {
            return;
        }

        if let Some(client) = &self.client {
            let client = Arc::clone(client);
            let token = self.user.as_ref().map(|user| user.token.clone());
            let future = async move {
                client.logout(token.as_deref()).await?;
                // TODO: log out is not an error
                Ok(Err("Logged out".to_string()))
            };
            self.log_task = Some(Task::new(&self.geng, future));
            self.user = None;
        }
    }

    /// The leaderboard needs to be polled to make progress.
    pub fn poll(&mut self) {
        if let Some(task) = self.log_task.take() {
            match task.poll() {
                Err(task) => self.log_task = Some(task),
                Ok(res) => {
                    match res {
                        Ok(Ok(user)) => {
                            log::debug!("Logged in as {:?}", user);
                            preferences::save(crate::PLAYER_LOGIN_STORAGE, &user);
                            self.loaded.player = Some(user.id);
                            self.user = Some(user);
                        }
                        Ok(Err(err)) => {
                            if err == "Logged out" {
                                log::debug!("Logged out");
                                preferences::save(crate::PLAYER_LOGIN_STORAGE, &());
                            } else {
                                log::error!("Failed to log in: {}", err);
                                // TODO: notification message
                            }
                        }
                        Err(err) => {
                            log::error!("Failed to log in: {:?}", err);
                        }
                    }
                }
            }
        }

        if let Some(task) = self.task.take() {
            match task.poll() {
                Err(task) => self.task = Some(task),
                Ok(res) => match res {
                    Ok(update) => {
                        log::debug!("Successfully loaded the leaderboard");
                        self.status = LeaderboardStatus::Done;
                        self.load_scores(update.scores);
                    }
                    Err(err) => {
                        log::error!("Loading leaderboard failed: {:?}", err);
                        self.status = LeaderboardStatus::Failed;
                    }
                },
            }
        }
    }

    fn load_scores(&mut self, mut scores: Vec<ScoreEntry>) {
        scores.sort_by_key(|entry| -entry.score);
        self.loaded.all_scores = scores;
        self.loaded.refresh();
    }

    /// Change meta filter using the cached scores if available.
    pub fn change_meta(&mut self, meta: ScoreMeta) {
        if self.loaded.meta == meta {
            return; // Unchanged
        }

        self.loaded.meta = meta;
        self.loaded.reload_local(None);
        match self.status {
            LeaderboardStatus::None | LeaderboardStatus::Failed => {
                self.refetch();
            }
            LeaderboardStatus::Pending => {}
            LeaderboardStatus::Done => {
                self.loaded.refresh();
            }
        }
    }

    /// Fetch scores from the server with the same meta.
    pub fn refetch(&mut self) {
        // Let the active task finish
        if self.task.is_some() {
            return;
        }

        if let Some(client) = &self.client {
            let board = Arc::clone(client);
            let level = self.loaded.level;
            let future = async move {
                log::debug!("Fetching scores for level {}...", level);
                board
                    .fetch_scores(level)
                    .await
                    .map(|scores| BoardUpdate { scores })
            };
            self.task = Some(Task::new(&self.geng, future));
            self.status = LeaderboardStatus::Pending;
        }
    }

    pub fn submit(&mut self, score: Option<i32>, level: Id, meta: ScoreMeta) {
        let score = score.map(|score| SavedScore {
            level,
            score,
            meta: meta.clone(),
        });

        self.loaded.meta = meta.clone();
        self.loaded.reload_local(score.as_ref());

        if let Some(board) = &self.client {
            let board = Arc::clone(board);
            let future = async move {
                let meta_str = meta_str(&meta);
                let score = score.map(|score| SubmitScore {
                    score: score.score,
                    extra_info: Some(meta_str),
                });

                if let Some(score) = &score {
                    log::debug!("Submitting a score...");
                    board.submit_score(level, score).await?;
                }

                log::debug!("Fetching scores...");
                let scores = board.fetch_scores(level).await?;
                Ok(BoardUpdate { scores })
            };
            self.task = Some(Task::new(&self.geng, future));
            self.status = LeaderboardStatus::Pending;
        }
    }
}

impl LoadedBoard {
    fn new() -> Self {
        Self {
            level: 0,
            meta: ScoreMeta::new(LevelModifiers::default(), HealthConfig::default()),
            player: None,
            my_position: None,
            all_scores: Vec::new(),
            filtered: Vec::new(),
            local_high: None,
        }
    }

    pub fn reload_local(&mut self, score: Option<&SavedScore>) {
        log::debug!("Reloading local scores with a new score: {:?}", score);
        let mut highscores: Vec<SavedScore> =
            preferences::load(crate::HIGHSCORES_STORAGE).unwrap_or_default();
        let mut save = false;
        if let Some(highscore) = highscores.iter_mut().find(|s| s.meta == self.meta) {
            if let Some(score) = score {
                if score.score > highscore.score && score.meta == highscore.meta {
                    highscore.score = score.score;
                    save = true;
                }
            }
            self.local_high = Some(highscore.clone());
        } else if let Some(score) = score {
            highscores.push(score.clone());
            save = true;
            self.local_high = Some(score.clone());
        } else {
            self.local_high = None;
        }
        if save {
            preferences::save(crate::HIGHSCORES_STORAGE, &highscores);
        }
    }

    /// Refresh the filter.
    fn refresh(&mut self) {
        log::debug!("Filtering scores with meta\n{:#?}", self.meta);

        let mut scores = self.all_scores.clone();

        // Filter for the same meta
        scores.retain(|entry| {
            !entry.user.name.is_empty()
                && entry.extra_info.as_ref().map_or(false, |info| {
                    serde_json::from_str::<ScoreMeta>(info)
                        .map_or(false, |entry_meta| entry_meta == self.meta)
                })
        });

        {
            // TODO: leave unique on server
            // Only leave unique players
            let mut i = 0;
            let mut ids_seen = HashSet::new();
            while i < scores.len() {
                if !ids_seen.contains(&scores[i].user.id) {
                    ids_seen.insert(scores[i].user.id);
                    i += 1;
                } else {
                    scores.remove(i);
                }
            }
        }

        self.filtered = scores;
        self.my_position = self.local_high.as_ref().and_then(|score| {
            self.filtered
                .iter()
                .position(|this| this.score == score.score)
        });
    }
}

fn meta_str(meta: &ScoreMeta) -> String {
    serde_json::to_string(meta).unwrap() // TODO: more compact?
}
