use super::*;

use geng_utils::conversions::Vec2RealConversions;

impl Model {
    pub fn update(&mut self, player_target: vec2<Coord>, delta_time: Time) {
        // TODO: interpolation or smth
        self.player.position = player_target;

        self.beat_timer -= delta_time;
        while self.beat_timer < Time::ZERO {
            self.beat_timer += Time::ONE; // TODO bpm

            // Spawn a light at a random pos
            let mut rng = thread_rng();
            let position = vec2(rng.gen_range(-5.0..=5.0), rng.gen_range(-5.0..=5.0)).as_r32();
            let rotation = Angle::from_degrees(rng.gen_range(0.0..=360.0));

            let shape_max = if rng.gen_bool(0.5) {
                Shape::Circle {
                    radius: r32(rng.gen_range(0.5..=1.0)),
                }
            } else {
                Shape::Line {
                    width: r32(rng.gen_range(0.2..=0.6)),
                }
            };

            self.lights.push(Light {
                position,
                rotation,
                shape_max,
                shape: Shape::Circle {
                    radius: Coord::ZERO,
                },
                lifetime: Lifetime::new_max(r32(1.5)),
            });
        }

        // Update lights
        for light in &mut self.lights {
            light.lifetime.change(-delta_time);

            let t = 1.0 - light.lifetime.get_ratio().as_f32(); // 0 to 1
            let t = 1.0 - (t - 0.5).abs() * 2.0; // 0 to 1 to 0
            let t = 3.0 * t * t - 2.0 * t * t * t; // Smoothstep
                                                   // let t = t.sqrt(); // Square root for linear area change
            let t = r32(t);
            light.shape = match light.shape_max {
                Shape::Circle { radius } => Shape::Circle { radius: radius * t },
                Shape::Line { width } => Shape::Line { width: width * t },
            };
        }
        self.lights.retain(|light| light.lifetime.is_above_min());
    }
}
