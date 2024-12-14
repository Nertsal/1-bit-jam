use geng::prelude::*;
use geng_utils::bounded::Bounded;

use crate::assets::Font;

pub struct Lerp<T> {
    pub time: Bounded<f32>,
    pub smoothstep: bool,
    pub from: T,
    pub to: T,
}

impl<T> Lerp<T> {
    pub fn new(time: f32, from: T, to: T) -> Self {
        Self {
            time: Bounded::new_zero(time),
            smoothstep: false,
            from,
            to,
        }
    }

    pub fn new_smooth(time: f32, from: T, to: T) -> Self {
        Self {
            smoothstep: true,
            ..Self::new(time, from, to)
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.time.change(delta_time);
    }
}

impl<T: Float> Lerp<T> {
    pub fn stop(&mut self) {
        self.to = self.current();
    }

    pub fn current(&self) -> T {
        let mut t = self.time.get_ratio();
        if self.smoothstep {
            t = smoothstep(t);
        };
        self.from + (self.to - self.from) * T::from_f32(t)
    }

    pub fn change_target(&mut self, to: T) {
        if self.to == to {
            return;
        }
        self.from = self.current();
        self.to = to;
        self.time.set_ratio(0.0);
    }
}

pub fn smoothstep<T: Float>(t: T) -> T {
    T::from_f32(3.0) * t * t - T::from_f32(2.0) * t * t * t
}

/// Returns the given color with the multiplied alpha.
pub fn with_alpha(mut color: Rgba<f32>, alpha: f32) -> Rgba<f32> {
    color.a *= alpha;
    color
}

pub fn wrap_text(font: &Font, text: &str, target_width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for source_line in text.lines() {
        let mut line = String::new();
        for word in source_line.split_whitespace() {
            if line.is_empty() {
                line += word;
                continue;
            }
            if font.measure(&(line.clone() + " " + word), 1.0).width() > target_width {
                lines.push(line);
                line = word.to_string();
            } else {
                line += " ";
                line += word;
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
    }
    lines
}
