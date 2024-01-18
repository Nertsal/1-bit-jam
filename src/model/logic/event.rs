use super::*;

impl Model {
    pub fn handle_event(&mut self, event: GameEvent) {
        match event {
            GameEvent::Rhythm { perfect } => {
                // Collect rhythm
                if let Some((event, light)) = self.player.closest_light.and_then(|id| {
                    self.level_state
                        .lights
                        .iter()
                        .find(|light| light.event_id == Some(id))
                        .map(|light| (id, light))
                }) {
                    if perfect {
                        self.last_rhythm = (event, light.closest_waypoint.1);
                    }
                }

                self.rhythms.push(Rhythm {
                    position: self.player.collider.position,
                    time: Bounded::new_zero(self.level.music.beat_time()),
                    perfect,
                });
            }
        }
    }
}
