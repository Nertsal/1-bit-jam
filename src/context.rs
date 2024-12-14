use crate::{
    local::{CachedMusic, LevelCache},
    prelude::{Assets, Id, Options, Time},
};

use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use ctl_client::{core::types::MusicInfo, Nertboard};
use geng::prelude::{time::Duration, *};

#[derive(Clone)]
pub struct Context {
    pub geng: Geng,
    pub assets: Rc<Assets>,
    pub music: Rc<MusicManager>,
    pub local: Rc<LevelCache>,
    options: Rc<RefCell<Options>>,
}

impl Context {
    pub async fn new(
        geng: &Geng,
        assets: Rc<Assets>,
        client: Option<&Arc<Nertboard>>,
    ) -> Result<Self> {
        Ok(Self {
            geng: geng.clone(),
            assets,
            music: Rc::new(MusicManager::new()),
            local: Rc::new(LevelCache::load(client, geng).await?),
            options: Rc::new(RefCell::new(
                preferences::load(crate::OPTIONS_STORAGE).unwrap_or_default(),
            )),
        })
    }

    pub fn get_options(&self) -> Options {
        self.options.borrow().clone()
    }

    pub fn set_options(&self, options: Options) {
        let mut old = self.options.borrow_mut();
        if *old != options {
            preferences::save(crate::OPTIONS_STORAGE, &options);
            *old = options;
        }
    }
}

pub struct MusicManager {
    inner: RefCell<MusicManagerImpl>,
}

struct MusicManagerImpl {
    volume: f32,
    playing: Option<Music>,
}

impl MusicManager {
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(MusicManagerImpl {
                volume: 0.5,
                playing: None,
            }),
        }
    }

    pub fn current(&self) -> Option<MusicInfo> {
        self.inner
            .borrow()
            .playing
            .as_ref()
            .map(|music| music.meta.clone())
    }

    pub fn set_volume(&self, volume: f32) {
        let mut inner = self.inner.borrow_mut();
        inner.volume = volume;
        if let Some(music) = &mut inner.playing {
            music.set_volume(volume);
        }
    }

    pub fn set_speed(&self, speed: f32) {
        let mut inner = self.inner.borrow_mut();
        if let Some(music) = &mut inner.playing {
            if let Some(effect) = &mut music.effect {
                effect.set_speed(speed);
            }
        }
    }

    pub fn stop(&self) {
        let mut inner = self.inner.borrow_mut();
        if let Some(music) = &mut inner.playing {
            music.stop();
        }
    }

    pub fn is_playing(&self) -> Option<Id> {
        self.inner
            .borrow()
            .playing
            .as_ref()
            .filter(|music| music.effect.is_some())
            .map(|music| music.meta.id)
    }

    pub fn switch(&self, music: &CachedMusic) {
        if self
            .inner
            .borrow()
            .playing
            .as_ref()
            .map_or(true, |playing| {
                playing.effect.is_none() || playing.meta.id != music.meta.id
            })
        {
            self.play(music);
        }
    }

    // pub fn restart_from(&self, time: Duration) {
    //     let mut inner = self.inner.borrow_mut();
    //     if let Some(music) = &mut inner.playing {
    //         music.play_from(time);
    //     }
    // }

    pub fn play(&self, music: &CachedMusic) {
        self.play_from(music, Duration::from_secs_f64(0.0))
    }

    pub fn play_from(&self, music: &CachedMusic, time: Duration) {
        let mut inner = self.inner.borrow_mut();
        let mut music = Music::from_cache(music);
        music.set_volume(inner.volume);
        music.play_from(time);
        inner.playing = Some(music);
    }

    pub fn play_from_beat(&self, music: &CachedMusic, beat: Time) {
        let time = Duration::from_secs_f64((beat * music.meta.beat_time()).as_f32() as f64);
        self.play_from(music, time)
    }
}

pub struct Music {
    pub meta: MusicInfo,
    sound: Rc<geng::Sound>,
    effect: Option<geng::SoundEffect>,
    volume: f32,
    /// Stop the music after the timer runs out.
    pub timer: Time,
}

impl Drop for Music {
    fn drop(&mut self) {
        self.stop();
    }
}

impl Debug for Music {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Music")
            .field("meta", &self.meta)
            // .field("effect", &self.effect)
            .field("volume", &self.volume)
            .field("timer", &self.timer)
            .finish()
    }
}

impl Clone for Music {
    fn clone(&self) -> Self {
        Self::new(self.sound.clone(), self.meta.clone())
    }
}

impl Music {
    pub fn new(sound: Rc<geng::Sound>, meta: MusicInfo) -> Self {
        Self {
            meta,
            sound,
            volume: 0.5,
            effect: None,
            timer: Time::ZERO,
        }
    }

    pub fn from_cache(cached: &CachedMusic) -> Self {
        Self::new(Rc::clone(&cached.music), cached.meta.clone())
    }

    pub fn set_volume(&mut self, volume: f32) {
        let volume = volume.clamp(0.0, 1.0);
        self.volume = volume;
        if let Some(effect) = &mut self.effect {
            effect.set_volume(volume);
        }
    }

    pub fn stop(&mut self) {
        if let Some(mut effect) = self.effect.take() {
            effect.stop();
        }
        self.timer = Time::ZERO;
    }

    pub fn play_from(&mut self, time: time::Duration) {
        self.stop();
        let mut effect = self.sound.effect();
        effect.set_volume(self.volume);
        effect.play_from(time);
        self.effect = Some(effect);
    }
}
