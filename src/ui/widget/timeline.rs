use super::*;

use crate::{
    assets::PixelTexture,
    editor::{Change, EditorAction, LevelAction, LevelEditor, LightId, ScrollSpeed},
    prelude::*,
    ui::{layout::AreaOps, UiState},
    util::{SecondOrderDynamics, SecondOrderState},
};

use std::collections::BTreeMap;

/// Pixels per unit
const PPU: usize = 2;
const LIGHT_LINE_WIDTH: f32 = 16.0;
const LIGHT_LINE_SPACE: f32 = 4.0;

pub struct TimelineWidget {
    context: UiContext,
    expansion: SecondOrderState<f32>,
    pub state: WidgetState,
    pub ceiling: WidgetState,
    pub extra_line: WidgetState,
    pub lights_line: WidgetState,
    pub main_line: WidgetState,
    pub highlight_line: WidgetState,
    highlight_bar: Option<HighlightBar>,
    dots: Vec<vec2<f32>>,
    ticks: Vec<(vec2<f32>, BeatTime)>,
    dragging_light: Option<(vec2<f32>, f32)>,
    dragging_waypoint: bool,

    /// Render scale in pixels per beat.
    scale: f32,
    /// The scrolloff in exact time.
    scroll: Time,
    raw_current_time: Time,
    level: Level, // TODO: reuse existing
    selected_light: Option<LightId>,
    selected_waypoint: Option<WaypointId>,
}

struct HighlightBar {
    from_time: Time,
    from: vec2<f32>,
    to_time: Time,
    to: vec2<f32>,
}

impl TimelineWidget {
    pub fn new(context: Context) -> Self {
        Self {
            context: UiContext {
                state: UiState::new(),
                geometry: crate::ui::geometry::GeometryContext::new(context.assets.clone()),
                font: context.geng.default_font().clone(),
                screen: Aabb2::ZERO.extend_positive(vec2(1.0, 1.0)),
                layout_size: 1.0,
                font_size: 1.0,
                can_focus: true.into(),
                cursor: CursorContext::new(),
                real_time: 0.0,
                delta_time: 0.1,
                mods: KeyModifiers::default(),
                text_edit: TextEdit::empty(),
                context,
            },
            expansion: SecondOrderState::new(SecondOrderDynamics::new(3.0, 1.0, 1.0, 0.0)),
            state: default(),
            ceiling: default(),
            extra_line: default(),
            lights_line: default(),
            main_line: default(),
            highlight_line: default(),
            highlight_bar: None,
            dots: Vec::new(),
            ticks: Vec::new(),
            dragging_light: None,
            dragging_waypoint: false,

            scale: 0.5,
            scroll: Time::ZERO,
            raw_current_time: Time::ZERO,
            level: Level::new(),
            selected_light: None,
            selected_waypoint: None,
        }
    }

    pub fn rescale(&mut self, new_scale: f32) {
        if new_scale.approx_eq(&0.0) {
            return;
        }

        self.scale = new_scale;
    }

    // pub fn auto_scale(&mut self, max_beat: Time) {
    //     let scale = self.state.position.width() / max_beat.as_f32().max(1.0);
    //     self.scale = scale;
    // }

    pub fn visible_scroll(&self) -> Time {
        (self.state.position.width() / self.scale) as Time
    }

    pub fn update_time(&mut self, current_beat: Time) {
        self.raw_current_time = current_beat;
        self.scroll = -current_beat;
    }

    fn reload(&mut self, editor: &LevelEditor, actions: &mut Vec<EditorAction>) {
        let sprites = &self.context.context.assets.sprites.timeline;

        // from time to screen position
        let render_at = |center: vec2<f32>, time: Time| {
            let size = vec2::splat(18) * PPU;
            let pos = (time + self.scroll) as f32 * self.scale;
            let pos = center + vec2(pos, 0.0);
            geng_utils::pixel::pixel_perfect_aabb(
                pos,
                vec2(0.5, 0.5),
                size,
                &geng::PixelPerfectCamera,
                self.context.geometry.framebuffer_size.as_f32(),
            )
        };
        let render_time = |line: &WidgetState, time: Time| render_at(line.position.center(), time);
        let render_light = |time: Time, i: usize| {
            let pos = self.lights_line.position.align_pos(vec2(0.5, 0.0))
                + vec2(
                    0.0,
                    (LIGHT_LINE_WIDTH * 0.5 + ((LIGHT_LINE_WIDTH + LIGHT_LINE_SPACE) * i as f32))
                        * PPU as f32,
                );
            render_at(pos, time)
        };

        // from screen position to time
        let unrender_time = |pos: f32| {
            ((pos - self.main_line.position.center().x) / self.scale).round() as Time - self.scroll
        };

        // TODO: customize snap
        let snap = BeatTime::QUARTER;

        // Check highlight bounds
        self.highlight_bar = self
            .selected_light
            .and_then(|id| self.level.events.get(id.event))
            .and_then(|event| {
                if let Event::Light(light) = &event.event {
                    let from_time = event.time;
                    let from = render_time(&self.highlight_line, from_time).center();
                    let to_time = event.time + light.movement.total_duration();
                    let to = render_time(&self.highlight_line, to_time).center();
                    Some(HighlightBar {
                        from_time,
                        from,
                        to_time,
                        to,
                    })
                } else {
                    None
                }
            });

        // Render events on the timeline
        let mut occupied = BTreeMap::new();
        self.dots.clear();
        let focus = {
            let mut focus = self.context.can_focus.borrow_mut();
            let f = *focus;
            *focus = self.state.hovered;
            f
        };
        for (i, event) in self.level.events.iter().enumerate() {
            if let Event::Light(light_event) = &event.event {
                let light_id = LightId { event: i };
                let is_selected = Some(light_id) == self.selected_light;

                // Selected light's waypoints
                if is_selected {
                    if !self.context.can_focus() || !self.context.cursor.down {
                        match self.dragging_light.take() {
                            // TODO: unmagic constant - max click shake distance and duration
                            Some((from, from_time))
                                if (self.context.cursor.position - from).len_sqr() < 25.0
                                    && (self.context.real_time - from_time).abs() < 0.5 => {}
                            Some(_) => {
                                if is_selected {
                                    actions.push(LevelAction::DeselectLight.into());
                                }
                            }
                            None => {}
                        }
                    }
                    if self.dragging_light.is_some() {
                        let time = unrender_time(self.context.cursor.position.x);
                        let time = editor.level.timing.snap_to_beat(time, snap)
                            - light_event.movement.fade_in;
                        actions.push(
                            LevelAction::MoveLight(
                                light_id,
                                Change::Set(time),
                                Change::Add(vec2::ZERO),
                            )
                            .into(),
                        );
                    }

                    let from_time = event.time;
                    let from = render_time(&self.highlight_line, from_time).center();
                    let to_time = event.time + light_event.movement.total_duration();
                    let to = render_time(&self.highlight_line, to_time).center();

                    let size = vec2(4.0, 16.0) * PPU as f32;

                    // Fade in
                    let position = Aabb2::point(from).extend_symmetric(size / 2.0);
                    let tick = self.context.state.get_or(|| {
                        // TODO: somehow mask this with other stuff
                        IconButtonWidget::new(&sprites.tick_smol)
                            .highlight(HighlightMode::Color(ThemeColor::Highlight))
                    });
                    tick.update(position, &self.context);
                    if tick.state.pressed {
                        // Drag fade in
                        let target = unrender_time(self.context.cursor.position.x);
                        let target = editor.level.timing.snap_to_beat(target, snap);
                        let fade_in = event.time + light_event.movement.fade_in - target;
                        actions
                            .push(LevelAction::ChangeFadeIn(light_id, Change::Set(fade_in)).into());
                    }

                    // Fade out
                    let position = Aabb2::point(to).extend_symmetric(size / 2.0);
                    let tick = self.context.state.get_or(|| {
                        // TODO: somehow mask this with other stuff
                        IconButtonWidget::new(&sprites.tick_smol)
                            .highlight(HighlightMode::Color(ThemeColor::Highlight))
                    });
                    tick.update(position, &self.context);
                    if tick.state.pressed {
                        // Drag fade out
                        let target = unrender_time(self.context.cursor.position.x);
                        let target = editor.level.timing.snap_to_beat(target, snap);
                        let fade_out = target - to_time + light_event.movement.fade_out;
                        actions.push(
                            LevelAction::ChangeFadeOut(light_id, Change::Set(fade_out)).into(),
                        );
                    }

                    let mut last_dot_time = from_time;
                    let mut connect_dots = |time: Time| {
                        // TODO: variable timing within this segment
                        let timing = self.level.timing.get_timing(from_time);

                        let resolution = 4.0; // Ticks per beat
                        let step = timing.beat_time / r32(resolution);
                        let dots = ((time_to_seconds(time - last_dot_time) / step).as_f32() + 0.1)
                            .floor() as usize;
                        let dots = (0..=dots).map(|i| {
                            let time = last_dot_time + seconds_to_time(step * r32(i as f32));
                            render_light(time, 0).center()
                        });

                        self.dots.extend(dots);
                        last_dot_time = time;
                    };

                    let last_id =
                        WaypointId::Frame(light_event.movement.key_frames.len().saturating_sub(1));
                    for (waypoint_id, _, offset) in light_event.movement.timed_positions() {
                        let is_waypoint_selected = Some(waypoint_id) == self.selected_waypoint;
                        // connect_dots(time + offset);

                        // Icon
                        let position = render_light(event.time + offset, 0).center();
                        let position = Aabb2::point(position).extend_uniform(5.0 * PPU as f32);
                        let texture = &sprites.waypoint;
                        // TODO: somehow mask this with other stuff
                        let icon = self.context.state.get_or(|| IconButtonWidget::new(texture));
                        icon.color = if is_waypoint_selected {
                            ThemeColor::Highlight
                        } else {
                            ThemeColor::Light
                        };
                        icon.update(position, &self.context);

                        // Tick
                        let position =
                            render_time(&self.highlight_line, event.time + offset).center();
                        let position = Aabb2::point(position).extend_symmetric(size / 2.0);
                        let texture = match waypoint_id {
                            WaypointId::Initial => &sprites.tick_big,
                            WaypointId::Frame(_) if waypoint_id == last_id => &sprites.tick_mid,
                            WaypointId::Frame(_) => &sprites.tick_smol,
                        };
                        let tick = self.context.state.get_or(|| {
                            // TODO: somehow mask this with other stuff
                            IconButtonWidget::new(texture)
                                .highlight(HighlightMode::Color(ThemeColor::Highlight))
                        });
                        tick.update(position, &self.context);

                        // Waypoint drag
                        if icon.state.clicked || tick.state.clicked {
                            actions.extend([
                                LevelAction::SelectLight(light_id).into(),
                                LevelAction::SelectWaypoint(waypoint_id, false).into(),
                            ]);
                            self.dragging_waypoint = true;
                        } else if !self.context.can_focus() || !self.context.cursor.down {
                            self.dragging_waypoint = false;
                        }
                        if self.dragging_waypoint && is_waypoint_selected {
                            let time = unrender_time(self.context.cursor.position.x);
                            let time = editor.level.timing.snap_to_beat(time, snap);
                            let time = Change::Set(time);
                            actions.push(
                                LevelAction::MoveWaypointTime(light_id, waypoint_id, time).into(),
                            );
                        }
                    }

                    connect_dots(event.time + light_event.movement.total_duration());
                }

                // Light icon
                let light_time = event.time + light_event.movement.fade_in;
                let visible =
                    !is_selected && (light_time + self.scroll).abs() < self.visible_scroll() / 2;
                if visible {
                    let dots = if self.highlight_bar.as_ref().map_or(false, |bar| {
                        (bar.from_time..=bar.to_time).contains(&light_time)
                    }) {
                        1
                    } else {
                        *occupied
                            .entry(light_time)
                            .and_modify(|x| *x += 1)
                            .or_insert(0)
                    };

                    if dots as f32 <= self.expansion.current + 0.9 {
                        let light = render_light(light_time, dots);
                        let texture = match light_event.shape {
                            Shape::Circle { .. } => &sprites.circle,
                            Shape::Line { .. } => &sprites.square,
                            Shape::Rectangle { .. } => &sprites.square,
                        };
                        // TODO: somehow mask this with other stuff
                        let icon = self.context.state.get_or(|| IconButtonWidget::new(texture));
                        icon.update(light, &self.context);
                        icon.color = if is_selected {
                            ThemeColor::Highlight
                        } else if light_event.danger {
                            ThemeColor::Danger
                        } else {
                            ThemeColor::Light
                        };
                        icon.texture = texture.clone();
                        if icon.state.hovered {
                            actions.push(LevelAction::HoverLight(light_id).into());
                        }
                        if icon.state.clicked {
                            actions.push(LevelAction::SelectLight(light_id).into());
                            self.dragging_light =
                                Some((self.context.cursor.position, self.context.real_time));
                        }
                    } else {
                        // Dots to indicate there are more light in that position
                        let dots = render_time(&self.extra_line, light_time);
                        let texture = &sprites.dots;
                        // TODO: somehow mask this with other stuff
                        let icon = self.context.state.get_or(|| IconWidget::new(texture));
                        icon.update(dots, &self.context);
                    }
                }
            }
        }

        self.expansion.target = if self.state.hovered {
            occupied.into_values().max().unwrap_or(0) as f32
        } else {
            0.0
        };

        // Main line ticks
        self.ticks.clear();
        let points = &self.level.timing.points;
        for (timing, next) in points
            .iter()
            .zip(points.iter().skip(1).map(Some).chain([None]))
        {
            let from = timing.time;
            let until = next.map(|timing| timing.time);
            for i in 0.. {
                let offset = r32(i as f32) * timing.beat_time;
                let time = from + seconds_to_time(offset);

                if -(time + self.scroll) > self.visible_scroll() / 2 {
                    continue;
                }

                let mut tick = |offset: BeatTime, marker: BeatTime| {
                    let offset = (r32(i as f32) + offset.as_beats()) * timing.beat_time;
                    let time = from + seconds_to_time(offset);
                    if !until.map_or(false, |limit| time >= limit) {
                        self.ticks
                            .push((render_time(&self.main_line, time).center(), marker));
                    }
                };

                tick(BeatTime::HALF, BeatTime::HALF);
                tick(BeatTime::QUARTER, BeatTime::QUARTER);
                tick(BeatTime::QUARTER * 3, BeatTime::QUARTER);

                if until.map_or(false, |limit| time >= limit)
                    || time + self.scroll > self.visible_scroll() / 2
                {
                    break;
                }

                self.ticks
                    .push((render_time(&self.main_line, time).center(), BeatTime::WHOLE));
            }
        }

        *self.context.can_focus.borrow_mut() = focus;
    }

    pub fn get_cursor_time(&self) -> Time {
        self.get_time_at(self.context.cursor.position.x)
    }

    fn get_time_at(&self, pos: f32) -> Time {
        ((pos - self.state.position.center().x) / self.scale) as Time - self.scroll
    }

    pub fn update(
        &mut self,
        position: Aabb2<f32>,
        context: &UiContext,
        state: &LevelEditor,
        actions: &mut Vec<EditorAction>,
    ) {
        self.context = context.clone();
        self.expansion.update(context.delta_time);
        self.level = state.level.clone();
        self.selected_light = state.selected_light;
        self.selected_waypoint = state
            .level_state
            .waypoints
            .as_ref()
            .and_then(|waypoints| waypoints.selected);

        let pixel = PPU as f32;

        let expansion = self.expansion.current * pixel * (LIGHT_LINE_WIDTH + LIGHT_LINE_SPACE);
        let mut position = position.extend_up(expansion);
        let state_top = position.max.y;

        let ceiling = position.cut_top(pixel * 3.0);
        self.ceiling.update(ceiling, context);

        let extra = position.cut_top(pixel * 3.0);
        self.extra_line.update(extra, context);
        position.cut_top(pixel * 2.0);

        let lights = position.cut_top(pixel * LIGHT_LINE_WIDTH + expansion);
        self.lights_line.update(lights, context);
        position.cut_top(pixel * 4.0);

        let main = position.cut_top(pixel * 16.0);
        self.main_line.update(main, context);
        position.cut_top(pixel * 4.0);

        let highlight = position.cut_top(pixel * 16.0);
        self.highlight_line.update(highlight, context);

        // TODO: unduplicate code from handle_event
        let scroll_speed = if context.mods.shift {
            ScrollSpeed::Slow
        } else if context.mods.alt {
            ScrollSpeed::Fast
        } else {
            ScrollSpeed::Normal
        };

        let state_full = Aabb2 {
            min: vec2(position.min.x, position.max.y),
            max: vec2(position.max.x, state_top),
        };
        self.state.update(state_full, context);
        if self.state.hovered {
            let delta = context.cursor.scroll_dir();
            if delta != 0 {
                if context.mods.ctrl {
                    // Zoom on the timeline
                    let delta = delta as f32;
                    actions.push(LevelAction::TimelineZoom(Change::Add(delta)).into());
                } else {
                    // Scroll on the timeline
                    actions.push(EditorAction::ScrollTimeBy(scroll_speed, delta));
                }
            }
        }
        if self.state.right_clicked {
            // TODO: maybe more specific to actual timeline actions
            actions.push(LevelAction::Cancel.into());
        }

        if self.main_line.clicked {
            let time = self.get_cursor_time();
            actions.push(LevelAction::ScrollTime(time - state.current_time.target).into());
        }

        self.reload(state, actions);

        context.update_focus(self.state.hovered); // Take focus
    }
}

impl Widget for TimelineWidget {
    fn draw(&self, context: &UiContext) -> Geometry {
        let pixel_scale = PPU;
        let pixel = pixel_scale as f32;
        let theme = context.theme();
        let sprites = &context.context.assets.sprites.timeline;

        let mut geometry = Geometry::new();

        for &dot in &self.dots {
            let dot = geng_utils::pixel::pixel_perfect_aabb(
                dot,
                vec2(0.5, 0.5),
                vec2::splat(pixel_scale * 2),
                &geng::PixelPerfectCamera,
                context.geometry.framebuffer_size.as_f32(),
            );
            geometry.merge(context.geometry.quad(dot, theme.light));
        }

        for &(pos, beat) in &self.ticks {
            let (color, texture) = if beat == BeatTime::WHOLE {
                (theme.light, &sprites.tick_big)
            } else if beat == BeatTime::HALF {
                (theme.danger, &sprites.tick_mid)
            } else if beat == BeatTime::QUARTER {
                (theme.highlight, &sprites.tick_smol)
            } else if beat == BeatTime::EIGHTH {
                (
                    Color::lerp(theme.highlight, theme.danger, 0.5),
                    &sprites.tick_tiny,
                )
            } else {
                // Unknown beat separation
                (theme.danger, &sprites.tick_smol)
            };
            geometry.merge(
                context
                    .geometry
                    .texture_pp_at(pos, color, pixel_scale, texture),
            );
        }

        {
            let texture = &sprites.current_arrow;
            let size = texture.size() * pixel_scale;
            let position = geng_utils::pixel::pixel_perfect_aabb(
                self.ceiling.position.align_pos(vec2(0.5, 1.0)),
                vec2(0.5, 1.0),
                size,
                &geng::PixelPerfectCamera,
                context.geometry.framebuffer_size.as_f32(),
            );

            geometry.merge(context.geometry.texture(
                position,
                mat3::identity(),
                theme.highlight,
                texture,
            ));
        }

        let main_bar = self.main_line.position;
        let main_bar = main_bar.align_aabb(vec2(main_bar.width(), pixel * 4.0), vec2(0.5, 0.5));
        geometry.merge(context.geometry.quad(main_bar, theme.light));

        if let Some(bar) = &self.highlight_bar {
            let highlight_bar = Aabb2::from_corners(bar.from, bar.to);
            let highlight_bar =
                highlight_bar.align_aabb(vec2(highlight_bar.width(), pixel * 4.0), vec2(0.5, 0.5));
            geometry.merge(context.geometry.quad(highlight_bar, theme.highlight));
        }

        let position = self.state.position;
        geometry = context.geometry.masked(position, geometry);
        let width = pixel * 2.0;
        geometry.merge(context.geometry.quad_outline(
            position.extend_uniform(width),
            width,
            theme.light,
        ));
        geometry.merge(
            context
                .geometry
                .quad_fill(position.extend_uniform(width * 3.0), theme.dark),
        );

        geometry.change_z_index(-100);
        geometry
    }
}

#[derive(Clone)]
struct IconWidget {
    state: WidgetState,
    texture: PixelTexture,
    color: ThemeColor,
}

impl IconWidget {
    pub fn new(texture: &PixelTexture) -> Self {
        Self {
            state: default(),
            texture: texture.clone(),
            color: ThemeColor::Light,
        }
    }
    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
    }
}

impl Widget for IconWidget {
    fn draw(&self, context: &UiContext) -> Geometry {
        let pixel_scale = PPU;
        let theme = context.theme();
        let mut geometry = Geometry::new();

        let fg_color = theme.get_color(self.color);

        geometry.merge(context.geometry.texture_pp_at(
            self.state.position.center(),
            fg_color,
            pixel_scale,
            &self.texture,
        ));

        geometry
    }
}

#[derive(Debug, Clone, Copy)]
enum HighlightMode {
    SwapColors,
    Color(ThemeColor),
}

#[derive(Clone)]
struct IconButtonWidget {
    state: WidgetState,
    texture: PixelTexture,
    color: ThemeColor,
    highlight: HighlightMode,
}

impl IconButtonWidget {
    pub fn new(texture: &PixelTexture) -> Self {
        Self {
            state: WidgetState::new(),
            texture: texture.clone(),
            color: ThemeColor::Light,
            highlight: HighlightMode::SwapColors,
        }
    }

    pub fn highlight(mut self, mode: HighlightMode) -> Self {
        self.highlight = mode;
        self
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.state.update(position, context);
    }
}

impl Widget for IconButtonWidget {
    fn draw(&self, context: &UiContext) -> Geometry {
        let pixel_scale = PPU;
        let theme = context.theme();
        let outline_width = pixel_scale as f32 * 3.0;
        let mut geometry = Geometry::new();

        let mut fg_color = theme.get_color(self.color);
        let mut bg_color = theme.dark;

        match self.highlight {
            HighlightMode::SwapColors => {
                if self.state.hovered {
                    std::mem::swap(&mut fg_color, &mut bg_color);
                }
                geometry.merge(context.geometry.texture_pp_at(
                    self.state.position.center(),
                    fg_color,
                    pixel_scale,
                    &self.texture,
                ));
                geometry.merge(
                    context
                        .geometry
                        .quad_fill(self.state.position.extend_uniform(outline_width), bg_color),
                );
            }
            HighlightMode::Color(highlight) => {
                if self.state.hovered {
                    fg_color = theme.get_color(highlight);
                }
                geometry.merge(context.geometry.texture_pp_at(
                    self.state.position.center(),
                    fg_color,
                    pixel_scale,
                    &self.texture,
                ));
            }
        }

        geometry
    }
}
