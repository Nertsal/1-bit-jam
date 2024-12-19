use super::*;

use crate::ui::geometry::Geometry;

#[derive(Debug, Clone)]
pub struct TooltipWidget {
    pub state: WidgetState,
    pub title: TextWidget,
    pub text: TextWidget,
}

impl TooltipWidget {
    pub fn new() -> Self {
        let mut state = WidgetState::new();
        state.hide();
        Self {
            state,
            title: TextWidget::new("shortcut"),
            text: TextWidget::new("tip").aligned(vec2(0.5, 0.0)),
        }
    }

    pub fn update(&mut self, anchor: &WidgetState, tip: impl Into<Name>, context: &UiContext) {
        if !anchor.hovered {
            return;
        }
        self.state.show();
        let mut position = Aabb2::point(anchor.position.top_right())
            .extend_positive(vec2::splat(context.font_size * 1.5));
        if position.max.x >= context.screen.max.x {
            position = position.translate(vec2(-anchor.position.width() - position.width(), 0.0));
        }
        self.state.update(position, context);

        let position = position.extend_uniform(-context.font_size * 0.2);

        let title = position.clone().cut_top(context.font_size * 0.3);
        self.title.update(title, &context.scale_font(0.7));

        self.text.text = tip.into();
        self.text.update(position, &context.scale_font(0.9));
    }
}

impl Widget for TooltipWidget {
    fn draw(&self, context: &UiContext) -> Geometry {
        if self.state.visible {
            return Geometry::new();
        }

        let position = self.state.position;
        let theme = context.theme();
        let width = context.font_size * 0.1;
        let mut geometry = context
            .geometry
            .quad_fill(position.extend_uniform(width / 2.0), theme.dark);
        geometry.merge(self.title.draw(context));
        geometry.merge(self.text.draw(context));
        geometry.merge(context.geometry.quad_outline(position, width, theme.light));
        geometry
    }
}

pub struct EditorEditUi {}

impl EditorEditUi {
    pub fn new() -> Self {
        Self {}
    }
}

impl EditorEditUi {
    pub fn layout(
        &mut self,
        position: Aabb2<f32>,
        game_position: Aabb2<f32>,
        context: &UiContext,
        editor: &Editor,
        actions: &mut Vec<EditorStateAction>,
    ) {
        let Some(level_editor) = &editor.level_edit else {
            let size = vec2(15.0, 1.0) * context.font_size;
            let warn = position
                .align_aabb(size, vec2(0.5, 1.0))
                .translate(vec2(0.0, -3.0 * size.y));

            let text = context
                .state
                .get_or(|| TextWidget::new("Select or create a difficulty in the Config tab"));
            text.update(warn, context);
            return;
        };

        let mut main = position;
        let font_size = context.font_size;
        let layout_size = context.layout_size;

        let bottom_bar = main.cut_bottom(game_position.min.y - 6.0 - main.min.y);
        let mut bottom_bar = bottom_bar.extend_symmetric(-vec2(5.0, 0.0) * layout_size);

        let mut main = main
            .extend_symmetric(-vec2(1.0, 2.0) * layout_size)
            .extend_up(-layout_size);
        let mut left_bar = main.cut_left(layout_size * 7.0);
        let mut right_bar = main.cut_right(layout_size * 7.0);

        let spacing = layout_size * 0.25;
        let title_size = font_size * 1.3;
        let button_height = font_size * 1.2;
        let delete_width = font_size * 3.5;
        let value_height = font_size * 1.2;

        let tooltip = context.state.get_or(TooltipWidget::new);

        // Event
        {
            let mut bar = left_bar;

            let event = bar.cut_top(title_size);
            let text = context
                .state
                .get_or(|| TextWidget::new("Event").aligned(vec2(0.0, 0.5)));
            text.update(event, context);
            text.options.size = title_size;

            if level_editor.level_state.waypoints.is_some() {
                let waypoint = bar.cut_top(button_height);
                bar.cut_top(spacing);
                let button = context.state.get_or(|| ButtonWidget::new("Add waypoint"));
                button.update(waypoint, context);
                if button.text.state.clicked {
                    actions.push(LevelAction::NewWaypoint.into());
                }

                tooltip.update(&button.text.state, "1", context);

                bar.cut_top(button_height);
                bar.cut_top(spacing);
            } else {
                // let palette = bar.cut_top(button_height);
                // bar.cut_top(spacing);
                // update!(self.new_palette, palette);
                // if self.new_palette.text.state.clicked {
                //     level_editor.palette_swap();
                // }

                let new_light_width = font_size * 3.0;

                let circle = bar.cut_top(button_height).cut_left(new_light_width);
                bar.cut_top(spacing);
                let button = context.state.get_or(|| ButtonWidget::new("Circle"));
                button.update(circle, context);
                if button.text.state.clicked {
                    actions.push(LevelAction::NewLight(Shape::circle(r32(1.0))).into());
                }
                tooltip.update(&button.text.state, "1", context);

                let line = bar.cut_top(button_height).cut_left(new_light_width);
                bar.cut_top(spacing);
                let button = context.state.get_or(|| ButtonWidget::new("Line"));
                button.update(line, context);
                if button.text.state.clicked {
                    actions.push(LevelAction::NewLight(Shape::line(r32(1.0))).into());
                }
                tooltip.update(&button.text.state, "2", context);
            }

            bar.cut_top(layout_size * 1.5);
            left_bar = bar;
        }

        // View
        {
            let mut bar = left_bar;

            let view = bar.cut_top(title_size);
            bar.cut_top(spacing);
            let text = context
                .state
                .get_or(|| TextWidget::new("View").aligned(vec2(0.0, 0.5)));
            text.update(view, context);
            text.options.size = title_size;

            let selected = bar.cut_top(font_size);
            bar.cut_top(spacing);
            let toggle = context.state.get_or(|| ToggleWidget::new("Only selected"));
            toggle.update(selected, context);
            if toggle.state.clicked {
                actions.push(EditorAction::ToggleShowOnlySelected.into());
            }
            toggle.checked = editor.show_only_selected;

            let dynamic = bar.cut_top(font_size);
            bar.cut_top(spacing);
            let toggle = context.state.get_or(|| ToggleWidget::new("Dynamic"));
            toggle.update(dynamic, context);
            if toggle.state.clicked {
                actions.push(EditorAction::ToggleDynamicVisual.into());
            }
            toggle.checked = editor.visualize_beat;
            tooltip.update(&toggle.state, "F", context);

            let grid = bar.cut_top(font_size);
            bar.cut_top(spacing);
            let toggle = context.state.get_or(|| ToggleWidget::new("Show grid"));
            toggle.update(grid, context);
            if toggle.state.clicked {
                actions.push(EditorAction::ToggleGrid.into());
            }
            toggle.checked = editor.render_options.show_grid;
            tooltip.update(&toggle.state, "C-~", context);

            // let waypoints = bar.cut_top(button_height);
            // bar.cut_top(spacing);
            // update!(self.view_waypoints, waypoints);
            // if self.view_waypoints.text.state.clicked {
            //     editor.view_waypoints();
            // }

            let zoom = bar.cut_top(value_height);
            bar.cut_top(spacing);
            let slider = context
                .state
                .get_or(|| ValueWidget::new_range("Zoom", editor.view_zoom, 0.5..=2.0, 0.25));
            {
                let mut view_zoom = editor.view_zoom;
                slider.update(zoom, context, &mut view_zoom);
                actions.push(EditorAction::SetViewZoom(view_zoom).into());
            }
            context.update_focus(slider.state.hovered);

            bar.cut_top(layout_size * 1.5);
            left_bar = bar;
        }

        // Placement
        {
            let mut bar = left_bar;

            let placement = bar.cut_top(title_size);
            let text = context
                .state
                .get_or(|| TextWidget::new("Placement").aligned(vec2(0.0, 0.5)));
            text.update(placement, context);
            text.options.size = title_size;

            let grid_snap = bar.cut_top(font_size);
            bar.cut_top(spacing);
            let button = context.state.get_or(|| ToggleWidget::new("Snap to grid"));
            button.update(grid_snap, context);
            if button.state.clicked {
                actions.push(EditorAction::ToggleGridSnap.into());
            }
            button.checked = editor.snap_to_grid;
            tooltip.update(&button.state, "~", context);

            let grid_size = bar.cut_top(value_height);
            bar.cut_top(spacing);
            {
                let mut value = 10.0 / editor.grid_size.as_f32();
                let slider = context
                    .state
                    .get_or(|| ValueWidget::new_range("Grid size", value, 2.0..=32.0, 1.0));
                slider.update(grid_size, context, &mut value);
                actions.push(EditorAction::SetGridSize(r32(10.0 / value)).into());
                context.update_focus(slider.state.hovered);
            }

            bar.cut_top(layout_size * 1.5);
            left_bar = bar;
        }

        // Light
        {
            let selected = level_editor
                .selected_light
                .and_then(|i| level_editor.level.events.get(i.event))
                .filter(|event| matches!(event.event, Event::Light(_)));

            if let Some(event) = selected {
                let light_id = level_editor
                    .selected_light
                    .expect("light selected without id 0_0");
                if let Event::Light(light) = &event.event {
                    let mut bar = right_bar;

                    let light_pos = bar.cut_top(title_size);
                    let text = context
                        .state
                        .get_or(|| TextWidget::new("Light").aligned(vec2(0.0, 0.5)));
                    text.update(light_pos, context);
                    text.options.size = title_size;

                    let delete = bar.cut_top(button_height).cut_left(delete_width);
                    let button = context
                        .state
                        .get_or(|| ButtonWidget::new("Delete").color(ThemeColor::Danger));
                    button.update(delete, context);
                    tooltip.update(&button.text.state, "X", context);
                    if button.text.state.clicked {
                        actions.push(LevelAction::DeleteLight(light_id).into());
                    }

                    let danger_pos = bar.cut_top(button_height);
                    bar.cut_top(spacing);
                    let button = context
                        .state
                        .get_or(|| ToggleWidget::new("Danger").color(ThemeColor::Danger));
                    button.update(danger_pos, context);
                    if button.state.clicked {
                        actions.push(LevelAction::ToggleDanger(light_id).into());
                    }
                    button.checked = light.danger;
                    tooltip.update(&button.state, "D", context);

                    {
                        let fade_in = bar.cut_top(value_height);
                        bar.cut_top(spacing);
                        let mut fade = time_to_seconds(light.movement.fade_in);
                        let slider = context.state.get_or(|| {
                            ValueWidget::new_range("Fade in", fade, r32(0.25)..=r32(10.0), r32(0.1))
                        });
                        slider.update(fade_in, context, &mut fade);
                        context.update_focus(slider.state.hovered);
                        actions.push(
                            LevelAction::ChangeFadeIn(light_id, Change::Set(seconds_to_time(fade)))
                                .into(),
                        );
                    }

                    {
                        let fade_out = bar.cut_top(value_height);
                        bar.cut_top(spacing);
                        let mut fade = time_to_seconds(light.movement.fade_out);
                        let slider = context.state.get_or(|| {
                            ValueWidget::new_range(
                                "Fade out",
                                fade,
                                r32(0.25)..=r32(10.0),
                                r32(0.1),
                            )
                        });
                        slider.update(fade_out, context, &mut fade);
                        context.update_focus(slider.state.hovered);
                        actions.push(
                            LevelAction::ChangeFadeOut(
                                light_id,
                                Change::Set(seconds_to_time(fade)),
                            )
                            .into(),
                        );
                    }

                    bar.cut_top(layout_size * 1.5);

                    let waypoints = bar.cut_top(title_size);
                    let button = context.state.get_or(|| ToggleWidget::new("Waypoints"));
                    button.update(waypoints, context);
                    button.text.options.size = title_size;
                    button.checked = matches!(level_editor.state, State::Waypoints { .. });
                    if button.state.clicked {
                        actions.push(LevelAction::ToggleWaypointsView.into());
                    }

                    bar.cut_top(spacing);
                    right_bar = bar;
                }
            }
        }

        if let Some(waypoints) = &level_editor.level_state.waypoints {
            if let Some(selected) = waypoints.selected {
                if let Some(event) = level_editor.level.events.get(waypoints.light.event) {
                    if let Event::Light(light) = &event.event {
                        let frames = light.movement.key_frames.len();
                        if let Some(frame) = light.movement.get_frame(selected) {
                            // Waypoint
                            let mut bar = right_bar;

                            let mut current = bar.cut_top(font_size);
                            let mut current = current.cut_left(font_size * 3.0);

                            // Previous waypoint
                            let prev = current
                                .cut_left(current.height() * 0.6)
                                .zero_size(vec2(0.5, 0.5))
                                .extend_uniform(font_size * 0.7);
                            if let WaypointId::Initial = selected {
                                let button = context.state.get_or(|| {
                                    IconWidget::new(
                                        &context.context.assets.sprites.button_prev_hollow,
                                    )
                                });
                                button.update(prev, context);
                            } else {
                                let button = context.state.get_or(|| {
                                    IconButtonWidget::new_normal(
                                        &context.context.assets.sprites.button_prev,
                                    )
                                });
                                button.update(prev, context);
                                if button.state.clicked {
                                    if let Some(id) = selected.prev() {
                                        actions.push(LevelAction::SelectWaypoint(id).into());
                                    }
                                }
                            };

                            let i = match selected {
                                WaypointId::Initial => 0,
                                WaypointId::Frame(i) => i + 1,
                            };

                            // Next waypoint
                            let next = current
                                .cut_right(current.height() * 0.6)
                                .zero_size(vec2(0.5, 0.5))
                                .extend_uniform(font_size * 0.7);
                            if i >= frames {
                                let button = context.state.get_or(|| {
                                    IconWidget::new(
                                        &context.context.assets.sprites.button_next_hollow,
                                    )
                                });
                                button.update(next, context);
                            } else {
                                let button = context.state.get_or(|| {
                                    IconButtonWidget::new_normal(
                                        &context.context.assets.sprites.button_next,
                                    )
                                });
                                button.update(next, context);
                                if button.state.clicked {
                                    actions
                                        .push(LevelAction::SelectWaypoint(selected.next()).into());
                                }
                            }

                            // Current waypoint
                            let text = context.state.get_or(|| TextWidget::new("0"));
                            text.update(current, context);
                            text.text = (i + 1).to_string().into();

                            // Delete
                            let delete = bar.cut_top(button_height).cut_left(delete_width);
                            let button = context
                                .state
                                .get_or(|| ButtonWidget::new("Delete").color(ThemeColor::Danger));
                            button.update(delete, context);
                            if button.text.state.clicked {
                                actions.push(
                                    LevelAction::DeleteWaypoint(waypoints.light, selected).into(),
                                );
                            }
                            tooltip.update(&button.text.state, "X", context);

                            let mut new_frame = frame;

                            let scale = bar.cut_top(value_height);
                            bar.cut_top(spacing);
                            let mut value = frame.scale.as_f32();
                            let slider = context.state.get_or(|| {
                                ValueWidget::new_range("Scale", value, 0.25..=10.0, 0.25)
                            });
                            slider.update(scale, context, &mut value);
                            new_frame.scale = r32(value);
                            context.update_focus(slider.state.hovered);

                            let angle = bar.cut_top(value_height);
                            bar.cut_top(spacing);
                            let mut value = frame.rotation.as_degrees().as_f32();
                            let slider = context
                                .state
                                .get_or(|| ValueWidget::new_circle("Angle", value, 360.0, 1.0));
                            slider.update(angle, context, &mut value);
                            new_frame.rotation = Angle::from_degrees(r32(value.round()));
                            context.update_focus(slider.state.hovered);
                            tooltip.update(&slider.state, "Q/E", context);

                            actions.push(
                                LevelAction::SetWaypointFrame(waypoints.light, selected, new_frame)
                                    .into(),
                            );

                            // Interpolation
                            let curve = bar.cut_top(button_height);
                            bar.cut_top(spacing);
                            let interpolation = bar.cut_top(button_height);
                            bar.cut_top(spacing);

                            if let Some((mut move_interpolation, mut curve_interpolation)) =
                                light.movement.get_interpolation(selected)
                            {
                                let waypoint_curve = context.state.get_or(|| {
                                    DropdownWidget::new(
                                        "Curve",
                                        0,
                                        [
                                            ("Continue", None),
                                            ("Linear", Some(TrajectoryInterpolation::Linear)),
                                            (
                                                "Spline",
                                                Some(TrajectoryInterpolation::Spline {
                                                    tension: r32(0.1),
                                                }),
                                            ),
                                            ("Bezier", Some(TrajectoryInterpolation::Bezier)),
                                        ],
                                    )
                                });

                                let waypoint_interpolation = context.state.get_or(|| {
                                    DropdownWidget::new(
                                        "Interpolation",
                                        0,
                                        [
                                            ("Linear", MoveInterpolation::Linear),
                                            ("Smoothstep", MoveInterpolation::Smoothstep),
                                            ("EaseIn", MoveInterpolation::EaseIn),
                                            ("EaseOut", MoveInterpolation::EaseOut),
                                        ],
                                    )
                                });

                                waypoint_curve.update(curve, context, &mut curve_interpolation);
                                actions.push(
                                    LevelAction::SetWaypointCurve(
                                        waypoints.light,
                                        selected,
                                        curve_interpolation,
                                    )
                                    .into(),
                                );

                                waypoint_interpolation.update(
                                    interpolation,
                                    context,
                                    &mut move_interpolation,
                                );
                                actions.push(
                                    LevelAction::SetWaypointInterpolation(
                                        waypoints.light,
                                        selected,
                                        move_interpolation,
                                    )
                                    .into(),
                                );
                            }

                            bar.cut_top(spacing);
                            right_bar = bar;
                        }
                    }
                }
            }
        }

        {
            let timeline = bottom_bar.cut_top(font_size * 1.0);
            let linetime = context
                .state
                .get_or(|| TimelineWidget::new(context.context.clone()));
            let was_pressed = linetime.state.pressed;

            {
                let mut timeline_actions = vec![];
                linetime.update(timeline, context, level_editor, &mut timeline_actions);
                actions.extend(timeline_actions.into_iter().map(Into::into));
            }

            if linetime.main_line.pressed {
                let time = linetime.get_cursor_time();
                actions.push(EditorStateAction::ScrollTime(
                    time - level_editor.current_time,
                ));
            }
            linetime.update_time(level_editor.current_time);

            // self.timeline.auto_scale(level_editor.level.last_beat());
        }

        let _ = left_bar;
        let _ = right_bar;
    }
}

// TODO: move to layout (we have scroll in context)
// if shift && self.ui.edit.timeline.state.hovered {
//     actions.push(EditorStateAction::TimelineScroll(scroll));
// } else if ctrl {
//     if self.ui.edit.timeline.state.hovered {
//         // Zoom on the timeline
//         actions.push(EditorStateAction::TimelineZoom(scroll));
//     } else if let State::Place { .. }
//     | State::Waypoints {
//         state: WaypointsState::New,
//         ..
//     } = level_editor.state
//     {
//         // Scale light or waypoint placement
//         let delta = scroll * r32(0.1);
//         actions.push(LevelAction::ScalePlacement(delta).into());
//     } else if let Some(waypoints) = &level_editor.level_state.waypoints {
//         if let Some(selected) = waypoints.selected {
//             let delta = scroll * r32(0.1);
//             actions.push(
//                 LevelAction::ScaleWaypoint(waypoints.light, selected, delta)
//                     .into(),
//             );
//         }
//     } else if let Some(id) = level_editor.selected_light {
//         // Control fade time
//         let scroll = scroll.as_f32() as Time;
//         let change = scroll * self.editor.config.scroll_slow.as_time(beat_time);
//         let action = if shift {
//             LevelAction::ChangeFadeOut(id, Change::Add(change))
//         } else {
//             LevelAction::ChangeFadeIn(id, Change::Add(change))
//         };
//         actions.push(action.into());
//     }
