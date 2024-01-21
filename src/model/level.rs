mod config;
mod state;

pub use self::{config::*, state::*};

use super::*;

type Name = Rc<str>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupMeta {
    pub name: Name,
    /// Music info
    pub music: MusicMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MusicMeta {
    /// Beats per minute.
    pub bpm: R32,
    pub author: Name,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LevelMeta {
    #[serde(default)] // 0 id for new levels not yet uploaded to the server.
    pub id: Id,
    pub name: Name,
    pub author: Name,
}

impl MusicMeta {
    /// Returns the duration (in seconds) of a single beat.
    pub fn beat_time(&self) -> Time {
        r32(60.0) / self.bpm
    }
}
