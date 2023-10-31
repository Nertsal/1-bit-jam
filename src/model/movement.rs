use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Movement {
    /// Time (in beats) to spend fading into the initial position.
    pub fade_in: Time,
    /// Time (in beats) to spend fading out of the last keyframe.
    pub fade_out: Time,
    pub initial: Transform,
    pub key_frames: VecDeque<MoveFrame>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MoveFrame {
    /// How long (in beats) should the interpolation from the last frame to that frame last.
    pub lerp_time: Time,
    pub transform: Transform,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Transform {
    pub translation: vec2<Coord>,
    pub rotation: Angle<Coord>,
    pub scale: Coord,
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            translation: vec2::ZERO,
            rotation: Angle::ZERO,
            scale: Coord::ONE,
        }
    }

    pub fn lerp(&self, target: &Self, t: Time) -> Self {
        Self {
            translation: self.translation + (target.translation - self.translation) * t,
            rotation: self.rotation + self.rotation.angle_to(target.rotation) * t,
            scale: self.scale + (target.scale - self.scale) * t,
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            fade_in: r32(1.0),
            fade_out: r32(1.0),
            initial: default(),
            key_frames: default(),
        }
    }
}

impl Movement {
    /// Iterate over frames with corrected (accumulated) transforms.
    pub fn frames_iter(&self) -> impl Iterator<Item = MoveFrame> + '_ {
        self.key_frames.iter().scan(self.initial, |trans, frame| {
            *trans = Transform {
                translation: trans.translation + frame.transform.translation,
                rotation: trans.rotation + frame.transform.rotation,
                scale: frame.transform.scale,
            };
            Some(MoveFrame {
                transform: *trans,
                ..*frame
            })
        })
    }

    /// Iterate over all key transformations (including initial).
    pub fn positions(&self) -> impl Iterator<Item = Transform> + '_ {
        std::iter::once(self.initial).chain(self.frames_iter().map(|frame| frame.transform))
    }

    /// Get the transform at the given time.
    pub fn get(&self, mut time: Time) -> Transform {
        let mut from = self.initial;

        let lerp = |from: Transform, to, time, duration| {
            let t = if duration > Time::ZERO {
                time / duration
            } else {
                Time::ONE
            };
            let t = crate::util::smoothstep(t);
            from.lerp(&to, t)
        };

        // Fade in
        if time <= self.fade_in {
            return lerp(
                Transform {
                    scale: Coord::ZERO,
                    ..from
                },
                from,
                time,
                self.fade_in,
            );
        }
        time -= self.fade_in;

        for frame in self.frames_iter() {
            if time <= frame.lerp_time {
                return lerp(from, frame.transform, time, frame.lerp_time);
            }
            time -= frame.lerp_time;
            from = frame.transform;
        }

        // Fade out
        let target = Transform {
            scale: Coord::ZERO,
            ..from
        };
        if time <= self.fade_out {
            lerp(from, target, time, self.fade_out)
        } else {
            target
        }
    }

    /// Get the transform at the end of the movement.
    pub fn get_finish(&self) -> Transform {
        self.frames_iter()
            .last()
            .map_or(self.initial, |frame| frame.transform)
    }

    /// Returns the total duration of the movement.
    pub fn duration(&self) -> Time {
        self.fade_in
            + self
                .key_frames
                .iter()
                .map(|frame| frame.lerp_time)
                .fold(Time::ZERO, Time::add)
            + self.fade_out
    }
}
