use super::*;

use anyhow::Result;
use assets::Assets;
use ctl_client::core::types::Id;

#[derive(clap::Subcommand)]
pub enum Command {
    /// Just display some dithered text on screen.
    Text {
        text: String,
    },
    Music(MusicArgs),
}

#[derive(clap::Args)]
pub struct MusicArgs {
    #[command(subcommand)]
    pub command: MusicCommand,
}

#[derive(clap::Subcommand)]
pub enum MusicCommand {
    Author(MusicAuthorArgs),
    /// Upload music to the server.
    Upload {
        path: PathBuf,
        #[clap(long)]
        name: String,
        #[clap(long)]
        original: bool,
        #[clap(long)]
        bpm: f32,
    },
    /// Update music info.
    Update {
        id: Id,
        #[clap(long)]
        name: Option<String>,
        #[clap(long)]
        public: Option<bool>,
        #[clap(long)]
        original: Option<bool>,
        #[clap(long)]
        bpm: Option<f32>,
    },
}

#[derive(clap::Args)]
pub struct MusicAuthorArgs {
    #[command(subcommand)]
    pub command: MusicAuthorCommand,
}

#[derive(clap::Subcommand)]
pub enum MusicAuthorCommand {
    Add {
        #[clap(long)]
        music: Id,
        #[clap(long)]
        artist: Id,
    },
    Remove {
        #[clap(long)]
        music: Id,
        #[clap(long)]
        artist: Id,
    },
}

impl Command {
    pub async fn execute(
        self,
        geng: Geng,
        assets: Rc<Assets>,
        secrets: Option<Secrets>,
    ) -> Result<()> {
        match self {
            Command::Text { text } => {
                let state = media::MediaState::new(&geng, &assets).with_text(text);
                geng.run_state(state).await;
            }
            Command::Music(music) => {
                let secrets = secrets.expect("Cannot update music without secrets");
                match music.command {
                    MusicCommand::Upload {
                        path,
                        name,
                        original,
                        bpm,
                    } => {
                        let music = ctl_client::core::types::NewMusic {
                            name,
                            original,
                            bpm,
                        };
                        log::info!("Uploading music from {:?}: {:?}", path, music);

                        let future = async move {
                            let client = ctl_client::Nertboard::new(
                                &secrets.leaderboard.url,
                                Some(secrets.leaderboard.key),
                            )
                            .context("Client initialization failed")?;
                            let music_id = client
                                .upload_music(&path, &music)
                                .await
                                .context("failed to upload music")?;
                            log::info!("Music uploaded successfully, id: {}", music_id);
                            anyhow::Ok(())
                        };
                        execute_task(future)??;
                    }
                    MusicCommand::Update {
                        id,
                        name,
                        public,
                        original,
                        bpm,
                    } => {
                        let update = ctl_client::core::types::MusicUpdate {
                            name,
                            public,
                            original,
                            bpm,
                        };
                        log::info!("Updating music {}: {:#?}", id, update);

                        let future = async move {
                            let client = ctl_client::Nertboard::new(
                                &secrets.leaderboard.url,
                                Some(secrets.leaderboard.key),
                            )
                            .context("Client initialization failed")?;
                            client
                                .update_music(id, &update)
                                .await
                                .context("failed to update music")?;
                            log::info!("Music updated successfully");
                            anyhow::Ok(())
                        };
                        execute_task(future)??;
                    }
                    MusicCommand::Author(author) => match author.command {
                        MusicAuthorCommand::Add { music, artist } => {
                            log::info!("Adding artist {} as author of music {}", artist, music);
                            let future = async move {
                                let client = ctl_client::Nertboard::new(
                                    &secrets.leaderboard.url,
                                    Some(secrets.leaderboard.key),
                                )
                                .context("Client initialization failed")?;
                                client
                                    .music_author_add(music, artist)
                                    .await
                                    .context("when adding artist as author")?;
                                anyhow::Ok(())
                            };
                            execute_task(future)??;
                        }
                        MusicAuthorCommand::Remove { music, artist } => {
                            log::info!("Removing artist {} as author of music {}", artist, music);
                            let future = async move {
                                let client = ctl_client::Nertboard::new(
                                    &secrets.leaderboard.url,
                                    Some(secrets.leaderboard.key),
                                )
                                .context("Client initialization failed")?;
                                client
                                    .music_author_remove(music, artist)
                                    .await
                                    .context("when adding artist as author")?;
                                anyhow::Ok(())
                            };
                            execute_task(future)??;
                        }
                    },
                }
            }
        }

        Ok(())
    }
}

fn execute_task<T: Send + Sync + 'static>(
    future: impl Future<Output = T> + Send + Sync + 'static,
) -> Result<T> {
    let mut task = task::Task::new(future);
    loop {
        if let Some(res) = task.poll() {
            return res.context("when executing a task");
        }
    }
}
