use super::*;

impl EditorState {
    pub fn process_event(&self, event: geng::Event) -> Vec<EditorStateAction> {
        let mut actions = vec![];

        match &event {
            geng::Event::KeyPress { key } => {
                if self.ui_context.text_edit.any_active() {
                    if let geng::Key::Escape | geng::Key::Enter = key {
                        actions.push(EditorStateAction::StopTextEdit);
                    }
                    return actions;
                }
            }
            geng::Event::EditText(text) => {
                actions.push(EditorStateAction::UpdateTextEdit(text.clone()));
            }
            geng::Event::CursorMove { position } => {
                actions.push(EditorStateAction::CursorMove(position.as_f32()));
            }
            geng::Event::Wheel { delta } => {
                actions.push(EditorStateAction::WheelScroll(*delta as f32));
            }
            _ => (),
        }

        if self.ui_focused {
            if let geng::Event::Wheel { .. } | geng::Event::MousePress { .. } = &event {
                return actions;
            }
        }

        if self.editor.tab != EditorTab::Edit {
            return actions;
        }
        let Some(level_editor) = &self.editor.level_edit else {
            return actions;
        };

        let window = self.context.geng.window();
        let ctrl = window.is_key_pressed(geng::Key::ControlLeft);
        let shift = window.is_key_pressed(geng::Key::ShiftLeft);
        let alt = window.is_key_pressed(geng::Key::AltLeft);

        let scroll_speed = if shift {
            ScrollSpeed::Slow
        } else if alt {
            ScrollSpeed::Fast
        } else {
            ScrollSpeed::Normal
        };

        let rotate_by = Angle::from_degrees(r32(15.0));

        match event {
            geng::Event::KeyPress { key } => match key {
                geng::Key::ArrowLeft => {
                    actions.push(EditorAction::ScrollTimeBy(scroll_speed, -1).into());
                }
                geng::Key::ArrowRight => {
                    actions.push(EditorAction::ScrollTimeBy(scroll_speed, 1).into());
                }
                geng::Key::C if ctrl => {
                    actions.push(LevelAction::Copy.into());
                }
                geng::Key::V if ctrl => {
                    actions.push(LevelAction::Paste.into());
                }
                geng::Key::V if shift => {
                    if let Some((id, anchor)) = self.get_anchor() {
                        actions.push(LevelAction::FlipVertical(id, anchor).into());
                    }
                }
                geng::Key::H if shift => {
                    if let Some((id, anchor)) = self.get_anchor() {
                        actions.push(LevelAction::FlipHorizontal(id, anchor).into());
                    }
                }
                geng::Key::F => {
                    actions.push(EditorAction::ToggleDynamicVisual.into());
                }
                geng::Key::X => {
                    if let Some(level_editor) = &self.editor.level_edit {
                        if let Some(waypoints) = &level_editor.level_state.waypoints {
                            if let Some(waypoint) = waypoints.selected {
                                actions.push(
                                    LevelAction::DeleteWaypoint(waypoints.light, waypoint).into(),
                                );
                            }
                        } else if let Some(light) = level_editor.selected_light {
                            actions.push(LevelAction::DeleteLight(light).into());
                        }
                    }
                }
                geng::Key::S if ctrl => {
                    actions.push(EditorAction::Save.into());
                }
                geng::Key::Q => self.rotate(&mut actions, rotate_by),
                geng::Key::E => self.rotate(&mut actions, -rotate_by),
                geng::Key::Z if ctrl => {
                    if shift {
                        actions.push(LevelAction::Redo.into());
                    } else {
                        actions.push(LevelAction::Undo.into());
                    }
                }
                geng::Key::H => {
                    actions.push(EditorAction::ToggleUI.into());
                }
                geng::Key::D => {
                    if let Some(level_editor) = &self.editor.level_edit {
                        if let State::Place { .. } = level_editor.state {
                            actions.push(LevelAction::ToggleDangerPlacement.into());
                        } else if let Some(light) = level_editor.selected_light {
                            actions.push(LevelAction::ToggleDanger(light).into());
                        }
                    }
                }
                geng::Key::W => {
                    actions.push(LevelAction::ToggleWaypointsView.into());
                }
                geng::Key::Backquote => {
                    if ctrl {
                        actions.push(EditorAction::ToggleGrid.into());
                    } else {
                        actions.push(EditorAction::ToggleGridSnap.into());
                    }
                }
                geng::Key::Escape => {
                    actions.push(EditorStateAction::Cancel);
                }
                geng::Key::Space => {
                    if let State::Playing { .. } = &level_editor.state {
                        actions.push(LevelAction::StopPlaying.into());
                    } else {
                        actions.push(LevelAction::StartPlaying.into());
                    }
                }
                geng::Key::Digit1 => actions.extend(self.handle_digit(1)),
                geng::Key::Digit2 => actions.extend(self.handle_digit(2)),
                geng::Key::Digit3 => actions.extend(self.handle_digit(3)),
                geng::Key::Digit4 => actions.extend(self.handle_digit(4)),
                geng::Key::Digit5 => actions.extend(self.handle_digit(5)),
                geng::Key::Digit6 => actions.extend(self.handle_digit(6)),
                geng::Key::Digit7 => actions.extend(self.handle_digit(7)),
                geng::Key::Digit8 => actions.extend(self.handle_digit(8)),
                geng::Key::Digit9 => actions.extend(self.handle_digit(9)),
                geng::Key::Digit0 => actions.extend(self.handle_digit(0)),
                geng::Key::F1 => {
                    actions.push(EditorAction::ToggleUI.into());
                }
                geng::Key::F5 => {
                    actions.push(EditorStateAction::StartPlaytest);
                }
                geng::Key::F11 => window.toggle_fullscreen(),
                _ => {}
            },
            geng::Event::Wheel { delta } => {
                let delta = delta as f32;
                if !self.ui_focused && self.ui.game.hovered {
                    if ctrl {
                        // Zoom view in/out
                        let delta = delta.signum() * 0.25;
                        actions.push(EditorAction::SetViewZoom(Change::Add(delta)).into());
                    } else {
                        // Scroll time
                        let scroll = delta.signum() as i64;
                        actions.push(EditorAction::ScrollTimeBy(scroll_speed, scroll).into());
                    }
                }
            }
            geng::Event::MousePress { button } => {
                actions.extend(self.cursor_down(button));
            }
            geng::Event::MouseRelease {
                button: geng::MouseButton::Left,
            } => actions.extend(self.cursor_up()),
            _ => {}
        }
        actions
    }

    fn handle_digit(&self, digit: u8) -> Vec<EditorStateAction> {
        let mut actions = vec![];

        let Some(level_editor) = &self.editor.level_edit else {
            return actions;
        };

        match &level_editor.state {
            State::Idle | State::Place { .. } => {
                if let Some(&shape) = self
                    .editor
                    .config
                    .shapes
                    .get((digit as usize).saturating_sub(1))
                {
                    actions.push(LevelAction::NewLight(shape).into());
                }
            }
            State::Waypoints { .. } => {
                // TODO: better key
                actions.push(LevelAction::NewWaypoint.into());
            }
            _ => (),
        }
        actions
    }

    fn cursor_down(&self, button: geng::MouseButton) -> Vec<EditorStateAction> {
        let mut actions = vec![EditorStateAction::CloseContextMenu];

        if self
            .ui
            .game
            .position
            .contains(self.ui_context.cursor.position)
            || self.editor.render_options.hide_ui
        {
            actions.extend(self.game_cursor_down(button));
        }

        actions
    }

    fn cursor_up(&self) -> Vec<EditorStateAction> {
        vec![EditorStateAction::EndDrag]
    }

    pub(super) fn update_drag(&mut self) -> Vec<EditorStateAction> {
        let mut actions = vec![];

        let Some(level_editor) = &self.editor.level_edit else {
            return actions;
        };

        let Some(drag) = &mut self.drag else {
            return actions;
        };
        match drag.target {
            DragTarget::Light {
                light,
                initial_time,
                initial_translation,
                ..
            } => {
                actions.push(
                    LevelAction::MoveLight(
                        light,
                        Change::Set(
                            level_editor.current_time.target - drag.from_beat + initial_time,
                        ),
                        Change::Set(
                            initial_translation + self.editor.cursor_world_pos_snapped
                                - drag.from_world,
                        ),
                    )
                    .into(),
                );
            }
            DragTarget::Waypoint {
                light,
                waypoint,
                initial_translation,
            } => {
                actions.push(
                    LevelAction::MoveWaypoint(
                        light,
                        waypoint,
                        // Change::Set(level_editor.current_beat - drag.from_beat + initial_time),
                        Change::Set(
                            initial_translation + self.editor.cursor_world_pos_snapped
                                - drag.from_world,
                        ),
                    )
                    .into(),
                );
            }
        }

        actions
    }

    fn game_cursor_down(&self, button: geng::MouseButton) -> Vec<EditorStateAction> {
        let mut actions = vec![];

        let Some(level_editor) = &self.editor.level_edit else {
            return actions;
        };

        match &level_editor.state {
            State::Idle => {
                // Select a light
                if let Some(event) = level_editor.level_state.hovered_event() {
                    let light_id = LightId { event };
                    actions.push(LevelAction::SelectLight(light_id).into());

                    let double = level_editor.selected_light == Some(light_id);
                    if let Some(e) = level_editor.level.events.get(event) {
                        if let Event::Light(light) = &e.event {
                            match button {
                                geng::MouseButton::Left => {
                                    let target = DragTarget::Light {
                                        double,
                                        light: LightId { event },
                                        initial_time: e.time,
                                        initial_translation: light.movement.initial.translation,
                                    };
                                    actions.push(EditorStateAction::StartDrag(target));
                                }
                                geng::MouseButton::Middle => {}
                                geng::MouseButton::Right => {
                                    let anchor = light
                                        .movement
                                        .get(level_editor.current_time.target - e.time)
                                        .translation;
                                    actions.push(EditorStateAction::ContextMenu(
                                        self.ui_context.cursor.position,
                                        vec![
                                            (
                                                "Copy light".into(),
                                                LevelAction::CopyLight(light_id).into(),
                                            ),
                                            ("Paste".into(), LevelAction::Paste.into()),
                                            (
                                                "Flip horizontally".into(),
                                                LevelAction::FlipHorizontal(light_id, anchor)
                                                    .into(),
                                            ),
                                            (
                                                "Flip vertically".into(),
                                                LevelAction::FlipVertical(light_id, anchor).into(),
                                            ),
                                            (
                                                "Delete".into(),
                                                LevelAction::DeleteLight(light_id).into(),
                                            ),
                                        ],
                                    ));
                                }
                            }
                        }
                    }
                } else {
                    // Deselect
                    actions.push(LevelAction::DeselectLight.into());
                    if let geng::MouseButton::Right = button {
                        actions.push(EditorStateAction::ContextMenu(
                            self.ui_context.cursor.position,
                            vec![("Paste".into(), LevelAction::Paste.into())],
                        ));
                    }
                }
            }
            State::Place { .. } => {
                if let geng::MouseButton::Left = button {
                    actions
                        .push(LevelAction::PlaceLight(self.editor.cursor_world_pos_snapped).into());
                }
            }
            State::Playing { .. } => {}
            State::Waypoints { state, .. } => match state {
                WaypointsState::Idle => {
                    if let Some(waypoints) = &level_editor.level_state.waypoints {
                        if let Some(hovered) =
                            waypoints.hovered.and_then(|i| waypoints.points.get(i))
                        {
                            if let Some(waypoint) = hovered.original {
                                if let Some(event) =
                                    level_editor.level.events.get(waypoints.light.event)
                                {
                                    if let Event::Light(event) = &event.event {
                                        if let Some(frame) = event.movement.get_frame(waypoint) {
                                            match button {
                                                geng::MouseButton::Left => {
                                                    actions.push(
                                                        LevelAction::SelectWaypoint(
                                                            waypoint, false,
                                                        )
                                                        .into(),
                                                    );
                                                    let initial_translation = frame.translation;
                                                    actions.push(EditorStateAction::StartDrag(
                                                        DragTarget::Waypoint {
                                                            light: waypoints.light,
                                                            waypoint,
                                                            initial_translation,
                                                        },
                                                    ));
                                                }
                                                geng::MouseButton::Middle => {}
                                                geng::MouseButton::Right => {
                                                    // TODO: context menu
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            // Deselect
                            actions.push(LevelAction::DeselectWaypoint.into());
                        }
                    }
                }
                WaypointsState::New => {
                    if let geng::MouseButton::Left = button {
                        actions.push(
                            LevelAction::PlaceWaypoint(self.editor.cursor_world_pos_snapped).into(),
                        );
                    }
                }
            },
        }
        actions
    }

    fn rotate(&self, actions: &mut Vec<EditorStateAction>, rotate_by: Angle<Coord>) {
        let Some(level_editor) = &self.editor.level_edit else {
            return;
        };

        if let State::Place { .. }
        | State::Waypoints {
            state: WaypointsState::New,
            ..
        } = &level_editor.state
        {
            actions.push(LevelAction::RotatePlacement(rotate_by).into());
            return;
        }

        if let Some(waypoints) = &level_editor.level_state.waypoints {
            if let Some(selected) = waypoints.selected {
                actions.push(
                    LevelAction::RotateWaypoint(waypoints.light, selected, Change::Add(rotate_by))
                        .into(),
                );
                return;
            }
        }

        if let Some(selected) = level_editor.selected_light {
            if let Some(event) = level_editor.level.events.get(selected.event) {
                if let Event::Light(light) = &event.event {
                    let time = level_editor.current_time.target - event.time;
                    let transform = light.movement.get(time);
                    actions.push(
                        LevelAction::RotateLightAround(selected, transform.translation, rotate_by)
                            .into(),
                    );
                }
            }
        }
    }

    fn get_anchor(&self) -> Option<(LightId, vec2<Coord>)> {
        let level_editor = self.editor.level_edit.as_ref()?;
        let id = level_editor.selected_light?;
        let event = level_editor.level.events.get(id.event)?;
        let Event::Light(light) = &event.event else {
            return None;
        };
        let anchor = light
            .movement
            .get(level_editor.current_time.target - event.time)
            .translation;
        Some((id, anchor))
    }
}
