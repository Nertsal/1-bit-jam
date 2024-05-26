use super::*;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::*;

    pub fn download_music(id: Id, data: Vec<u8>, info: &MusicInfo) -> Result<()> {
        let path = music_path(id);
        std::fs::create_dir_all(&path)?;

        std::fs::write(path.join("music.mp3"), data)?;
        std::fs::write(path.join("meta.toml"), toml::to_string_pretty(&info)?)?;

        Ok(())
    }

    pub fn save_group(group: &CachedGroup) -> Result<()> {
        let path = &group.path;
        std::fs::create_dir_all(path)?;

        let writer = std::io::BufWriter::new(std::fs::File::create(path)?);
        bincode::serialize_into(writer, &group.data)?;

        log::debug!("Saved group ({}) successfully", group.data.id);

        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

/// Path to the directory that hold locally saved levels and music.
pub fn base_path() -> PathBuf {
    preferences::base_path()
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
            let name: String = (0..3).map(|_| rng.gen_range('a'..='z')).collect();
            let path = base_path.join(name);
            if !path.exists() {
                return path;
            }
        }
    } else {
        base_path.join(format!("{}", group))
    }
}

pub fn generate_level_path(group_path: impl AsRef<Path>, level: Id) -> PathBuf {
    let group_path = group_path.as_ref();
    if level == 0 {
        // Generate a random string until it is available
        let mut rng = rand::thread_rng();
        loop {
            let name: String = (0..3).map(|_| rng.gen_range('a'..='z')).collect();
            let path = group_path.join(name);
            if !path.exists() {
                return path;
            }
        }
    } else {
        group_path.join(format!("{}", level))
    }
}

impl CachedMusic {
    pub async fn load(manager: &geng::asset::Manager, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let meta_path = path.join("meta.toml");
        let meta: MusicInfo = file::load_detect(&meta_path).await?;

        let file_path = path.join("music.mp3");
        let file: geng::Sound = geng::asset::Load::load(
            manager,
            &file_path,
            &geng::asset::SoundOptions { looped: false },
        )
        .await?;

        Ok(Self {
            meta,
            music: Rc::new(file),
        })
    }
}

impl CachedGroup {
    pub fn new(data: LevelSet) -> Self {
        Self {
            path: fs::generate_group_path(data.id),
            music: None,
            hash: data.calculate_hash(),
            data,
        }
    }
}
