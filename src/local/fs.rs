#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod web;

use crate::assets::MusicAssets;

use super::*;

pub struct Controller {
    #[cfg(target_arch = "wasm32")]
    rexie: rexie::Rexie,
    geng: Geng,
}

impl Controller {
    pub async fn new(geng: &Geng) -> Result<Self> {
        #[cfg(target_arch = "wasm32")]
        {
            let rexie = match web::build_database().await {
                Ok(rexie) => rexie,
                Err(err) => {
                    log::error!("failed to init web file system: {}", err);
                    anyhow::bail!("check logs");
                }
            };
            log::info!("Connected to browser indexed db");
            Ok(Self {
                rexie,
                geng: geng.clone(),
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let base_path = base_path();
            std::fs::create_dir_all(base_path)?;
            Ok(Self { geng: geng.clone() })
        }
    }

    pub async fn load_music_all(&self) -> Result<Vec<CachedMusic>> {
        log::debug!("Loading all local music");

        #[cfg(target_arch = "wasm32")]
        let music = {
            match web::load_music_all(&self.rexie, &self.geng).await {
                Ok(items) => Ok(items),
                Err(err) => {
                    log::error!("failed to load music from web file system: {}", err);
                    anyhow::bail!("check logs");
                }
            }
        };
        #[cfg(not(target_arch = "wasm32"))]
        let music = { native::load_music_all(&self.geng).await };

        let mut music = music?;
        if let Ok(assets) = load_music_all_assets(&self.geng).await {
            music.extend(assets);
        }
        Ok(music)
    }

    pub async fn load_groups_all(&self) -> Result<Vec<(PathBuf, LevelSet)>> {
        log::debug!("Loading all local groups");

        #[cfg(target_arch = "wasm32")]
        let groups = {
            match web::load_groups_all(&self.rexie).await {
                Ok(items) => Ok(items),
                Err(err) => {
                    log::error!("failed to load groups from web file system: {}", err);
                    anyhow::bail!("check logs");
                }
            }
        };
        #[cfg(not(target_arch = "wasm32"))]
        let groups = { native::load_groups_all().await };

        let mut groups = groups?;
        if let Ok(assets) = load_groups_all_assets().await {
            groups.extend(assets);
        }
        Ok(groups)
    }

    pub async fn save_music(&self, music: &CachedMusic, data: &[u8]) -> Result<()> {
        let id = music.meta.id;
        let info = &music.meta;

        log::debug!("Saving music: {}", id);

        #[cfg(target_arch = "wasm32")]
        {
            if let Err(err) = web::save_music(&self.rexie, id, data, info).await {
                log::error!("failed to save music into web file system: {}", err);
                anyhow::bail!("check logs");
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            native::save_music(id, data, info)?;
        }
        Ok(())
    }

    pub async fn save_group(&self, group: &CachedGroup) -> Result<()> {
        log::debug!("Saving group: {}", group.data.id);
        #[cfg(target_arch = "wasm32")]
        {
            let id = group.data.id.to_string();
            let id = group
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(&id);
            if let Err(err) = web::save_group(&self.rexie, group, id).await {
                log::error!("failed to save group into web file system: {}", err);
                anyhow::bail!("check logs");
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            native::save_group(group)?;
        }
        Ok(())
    }

    pub async fn remove_music(&self, id: Id) -> Result<()> {
        log::debug!("Deleting music: {:?}", id);
        #[cfg(target_arch = "wasm32")]
        {
            if let Err(err) = web::remove_music(&self.rexie, id).await {
                log::error!("failed to remove music from the web file system: {}", err);
                anyhow::bail!("check logs");
            }
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = music_path(id);
            std::fs::remove_file(path)?;
            Ok(())
        }
    }

    pub async fn remove_group(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        log::debug!("Deleting a group: {:?}", path);
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(id) = path.file_name().and_then(|name| name.to_str()) {
                if let Err(err) = web::remove_group(&self.rexie, id).await {
                    log::error!("failed to remove group from the web file system: {}", err);
                    anyhow::bail!("check logs");
                }
            }
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::fs::remove_file(path)?;
            Ok(())
        }
    }
}

/// Path to the directory that hold locally saved levels and music.
pub fn base_path() -> PathBuf {
    #[cfg(target_arch = "wasm32")]
    {
        "close-to-light".into()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let exe = std::env::current_exe().expect("Failed to find current exe");
        let app_name = exe.file_stem().unwrap();
        if let Some(dirs) =
            directories::ProjectDirs::from("", "", app_name.to_str().expect("Exe name is invalid"))
        {
            return dirs.data_dir().to_path_buf();
        }
        if let Some(dir) = exe.parent() {
            return dir.to_path_buf();
        }
        std::env::current_dir().unwrap()
    }
}

pub fn all_music_path() -> PathBuf {
    base_path().join("music")
}

pub fn all_groups_path() -> PathBuf {
    base_path().join("levels")
}

pub fn music_path(music: Id) -> PathBuf {
    all_music_path().join(format!("{}", music))
}

pub fn generate_group_path(group: Id) -> PathBuf {
    let base_path = all_groups_path();
    if group == 0 {
        // Generate a random string until it is available
        let mut rng = rand::thread_rng();
        loop {
            let name: String = (0..5).map(|_| rng.gen_range('a'..='z')).collect();
            let path = base_path.join(name);
            // TODO: validate on web
            if !path.exists() {
                return path;
            }
        }
    } else {
        base_path.join(format!("{}", group))
    }
}

impl CachedMusic {
    pub async fn load(manager: &geng::asset::Manager, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let assets: MusicAssets = geng::asset::Load::load(manager, path, &()).await?;
        Ok(Self::new(assets.meta, assets.music))
    }
}

impl CachedGroup {
    pub fn new(path: PathBuf, data: LevelSet) -> Self {
        let level_hashes = data
            .levels
            .iter()
            .map(|level| level.data.calculate_hash())
            .collect();

        Self {
            path,
            music: None,
            hash: data.calculate_hash(),
            origin: None,
            data,
            level_hashes,
        }
    }
}

async fn load_music_all_assets(geng: &Geng) -> Result<Vec<CachedMusic>> {
    let music_path = run_dir().join("assets").join("groups").join("music");
    if !music_path.exists() {
        return Ok(Vec::new());
    }

    let list: Vec<String> = file::load_detect(music_path.join("_list.ron")).await?;
    let paths: Vec<_> = list
        .into_iter()
        .map(|entry| music_path.join(entry))
        .collect();
    let music_loaders = paths.into_iter().map(|path| load_music(geng, path));
    let music = future::join_all(music_loaders).await;

    let mut res = Vec::new();
    for music in music {
        match music {
            Ok(music) => res.push(music),
            Err(err) => {
                log::error!("failed to load music: {}", err);
            }
        }
    }

    Ok(res)
}

async fn load_music(geng: &Geng, path: PathBuf) -> Result<CachedMusic> {
    let res = async {
        log::debug!("loading music at {:?}", &path);
        let music = CachedMusic::load(geng.asset_manager(), &path).await?;
        Ok(music)
    }
    .await;
    if let Err(err) = &res {
        log::error!("failed to load music: {:?}", err);
    }
    res
}

async fn load_groups_all_assets() -> Result<Vec<(PathBuf, LevelSet)>> {
    let groups_path = run_dir().join("assets").join("groups").join("levels");
    if !groups_path.exists() {
        return Ok(Vec::new());
    }

    let list: Vec<String> = file::load_detect(groups_path.join("_list.ron")).await?;
    let paths: Vec<_> = list
        .into_iter()
        .map(|entry| groups_path.join(entry))
        .collect();

    let load_group = |path| async move {
        let bytes = file::load_bytes(&path).await?;
        let group: LevelSet = bincode::deserialize(&bytes)?;
        anyhow::Ok((path, group))
    };

    let group_loaders = paths.into_iter().map(load_group);
    let groups = future::join_all(group_loaders).await;

    let mut res = Vec::new();
    for group in groups {
        match group {
            Ok((path, group)) => res.push((path, group)),
            Err(err) => {
                log::error!("failed to load group: {}", err);
            }
        }
    }

    Ok(res)
}
