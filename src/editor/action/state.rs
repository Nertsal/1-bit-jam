use super::*;

#[derive(Debug, Clone)]
pub enum EditorStateAction {
    Exit,
    ScrollTime(Time),
    Editor(EditorAction),
    StopTextEdit,
    UpdateTextEdit(String),
    CursorMove(vec2<f32>),
    WheelScroll(f32),
    StartPlaytest,
    TimelineScroll(Coord),
    TimelineZoom(Coord),
    EndDrag,
    StartDrag(DragTarget),
    ConfirmPopupAction,
}

impl From<EditorAction> for EditorStateAction {
    fn from(value: EditorAction) -> Self {
        Self::Editor(value)
    }
}

impl From<LevelAction> for EditorStateAction {
    fn from(value: LevelAction) -> Self {
        Self::Editor(EditorAction::Level(value))
    }
}

impl EditorState {
    pub fn execute(&mut self, action: EditorStateAction) {
        // log::debug!("action EditorStateAction::{:?}", action);
        match action {
            EditorStateAction::Exit => {
                self.transition = Some(geng::state::Transition::Pop);
            }
            EditorStateAction::ScrollTime(delta) => self.scroll_time(delta),
            EditorStateAction::Editor(action) => self.editor.execute(action),
            EditorStateAction::StopTextEdit => {
                self.ui_context.text_edit.stop();
            }
            EditorStateAction::UpdateTextEdit(text) => {
                self.ui_context.text_edit.text = text;
            }
            EditorStateAction::CursorMove(position) => {
                self.ui_context.cursor.cursor_move(position);
                if let Some(drag) = &mut self.drag {
                    drag.moved = true;
                }
            }
            EditorStateAction::WheelScroll(delta) => {
                self.ui_context.cursor.scroll += delta;
            }
            EditorStateAction::StartPlaytest => self.play_game(),
            EditorStateAction::TimelineScroll(scroll) => {
                if let Some(level_editor) = &self.editor.level_edit {
                    let timeline = &mut self.ui.edit.timeline;
                    let delta = -scroll.as_f32() * 30.0 / timeline.get_scale();
                    let delta = (delta * TIME_IN_FLOAT_TIME as f32).round() as Time;
                    let current = -timeline.get_scroll();
                    let delta = if delta > 0 {
                        delta.min(current)
                    } else {
                        -delta.abs().min(
                            level_editor.level.last_time() - timeline.visible_scroll() - current,
                        )
                    };
                    timeline.scroll(delta);
                }
            }
            EditorStateAction::TimelineZoom(scroll) => {
                let timeline = &mut self.ui.edit.timeline;
                let zoom = timeline.get_scale();
                let zoom = (zoom + scroll.as_f32() * 0.05).clamp(0.05, 0.75);
                timeline.rescale(zoom);
            }
            EditorStateAction::EndDrag => self.end_drag(),
            EditorStateAction::StartDrag(target) => self.start_drag(target),
            EditorStateAction::ConfirmPopupAction => self.editor.confirm_action(&mut self.ui),
        }
    }

    fn end_drag(&mut self) {
        let Some(level_editor) = &mut self.editor.level_edit else {
            return;
        };

        if let Some(drag) = self.drag.take() {
            if let DragTarget::Light { double, .. } = drag.target {
                if double
                    && drag.from_world == self.editor.cursor_world_pos_snapped
                    && level_editor.real_time - drag.from_real_time < r32(0.5)
                {
                    // See waypoints
                    level_editor.view_waypoints();
                }
            }

            level_editor.flush_changes();
        }
    }

    fn start_drag(&mut self, target: DragTarget) {
        self.end_drag();

        let Some(level_editor) = &mut self.editor.level_edit else {
            return;
        };

        self.drag = Some(Drag {
            moved: false,
            from_screen: self.ui_context.cursor.position,
            from_world: self.editor.cursor_world_pos_snapped,
            from_real_time: level_editor.real_time,
            from_beat: level_editor.current_time,
            target,
        });
    }

    fn scroll_time(&mut self, mut delta: Time) {
        let Some(level_editor) = &mut self.editor.level_edit else {
            return;
        };

        if let Some(waypoints) = &level_editor.level_state.waypoints {
            if let Some(waypoint) = waypoints.selected {
                // Move waypoint in time
                if let Some(event) = level_editor.level.events.get_mut(waypoints.light.event) {
                    if let Event::Light(light) = &mut event.event {
                        // Move temporaly
                        if let Some(beat) = light.movement.get_time(waypoint) {
                            let next_i = match waypoint {
                                WaypointId::Initial => 0,
                                WaypointId::Frame(i) => i + 1,
                            };
                            let next = WaypointId::Frame(next_i);
                            let next_time = light.movement.get_time(next);

                            let min_lerp = 50;
                            let max_delta = next_time
                                .map_or(50 * TIME_IN_FLOAT_TIME, |time| time - min_lerp - beat);

                            delta = delta.min(max_delta);
                            let snap = |time: Time| {
                                // TODO: customize snap
                                level_editor
                                    .level
                                    .timing
                                    .snap_to_beat(time, BeatTime::QUARTER)
                            };

                            match waypoint {
                                WaypointId::Initial => event.time = snap(event.time + delta),
                                WaypointId::Frame(i) => {
                                    if let Some(frame) = light.movement.key_frames.get_mut(i) {
                                        let target = (frame.lerp_time + delta).max(min_lerp);
                                        let target = snap(target);
                                        delta = target - frame.lerp_time;
                                        frame.lerp_time = target;
                                    }
                                }
                            }

                            if let Some(next) = light.movement.key_frames.get_mut(next_i) {
                                next.lerp_time -= delta;
                            }

                            level_editor.save_state(HistoryLabel::MoveWaypointTime(
                                waypoints.light,
                                waypoint,
                            ));
                        }
                    }
                }
                return;
            }
        }

        // Scroll current time
        level_editor.scroll_time(delta);
    }
}
