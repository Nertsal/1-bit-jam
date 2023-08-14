mod collider;
mod config;
mod level;
mod light;
mod logic;
mod movement;

pub use self::{collider::*, config::*, level::*, light::*, movement::*};

use std::collections::VecDeque;

use geng::prelude::*;
use geng_utils::{bounded::Bounded, conversions::Vec2RealConversions};

pub type Time = R32;
pub type Coord = R32;
pub type Lifetime = Bounded<Time>;

#[derive(Debug)]
pub struct QueuedEvent {
    /// Delay until the event should happen (in seconds).
    pub delay: Time,
    pub event: Event,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub target_position: vec2<Coord>,
    pub shake: vec2<Coord>,
    pub collider: Collider,
    pub fear_meter: Bounded<Time>,
}

pub struct Model {
    pub config: Config,
    pub level: Level,
    pub current_beat: usize,
    pub camera: Camera2d,
    /// The time until the next music beat.
    pub beat_timer: Time,
    pub queued_events: Vec<QueuedEvent>,
    pub player: Player,
    pub telegraphs: Vec<LightTelegraph>,
    pub lights: Vec<Light>,
}

impl Model {
    pub fn new(config: Config, level: Level) -> Self {
        Self {
            config,
            level,
            current_beat: 0,
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            beat_timer: Time::ZERO,
            queued_events: Vec::new(),
            player: Player {
                target_position: vec2::ZERO,
                shake: vec2::ZERO,
                collider: Collider::new(vec2::ZERO, Shape::Circle { radius: r32(0.2) }),
                fear_meter: Bounded::new_max(r32(1.0)),
            },
            telegraphs: vec![],
            lights: vec![],
        }
    }
}
