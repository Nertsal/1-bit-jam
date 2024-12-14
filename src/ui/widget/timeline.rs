use super::*;

use crate::{
    editor::{LevelEditor, LightId},
    prelude::*,
};

use std::collections::BTreeMap;

pub struct TimelineWidget {
    context: UiContext,
    pub state: WidgetState,
    pub clickable: WidgetState,
    pub bar: WidgetState,

    pub current_beat: WidgetState,

    /// Start of the selection.
    pub left: WidgetState,
    /// End of the selection.
    pub right: WidgetState,
    /// Replay current position.
    pub replay: WidgetState,

    pub lights: BTreeMap<Time, Vec<(LightId, WidgetState)>>,
    pub selected: WidgetState,
    pub waypoints: Vec<(WaypointId, WidgetState)>,

    /// Render scale in pixels per beat.
    scale: f32,
    /// The scrolloff in beats.
    scroll: Time,
    raw_current_beat: Time,
    raw_left: Option<Time>,
    raw_right: Option<Time>,
    raw_replay: Option<Time>,
    level: Level, // TODO: reuse existing
    selected_light: Option<LightId>,
    selected_waypoint: Option<WaypointId>,
}

impl TimelineWidget {
    pub fn new(context: Context) -> Self {
        Self {
            context: UiContext {
                font: context.assets.fonts.default.clone(),
                screen: Aabb2::ZERO.extend_positive(vec2(1.0, 1.0)),
                layout_size: 1.0,
                font_size: 1.0,
                can_focus: true,
                cursor: CursorContext::new(),
                delta_time: 0.1,
                mods: KeyModifiers::default(),
                text_edit: TextEdit::empty(),
                context,
            },
            state: default(),
            clickable: default(),
            bar: default(),

            current_beat: default(),

            left: default(),
            right: default(),
            replay: default(),

            lights: BTreeMap::new(),
            selected: WidgetState::new(),
            waypoints: Vec::new(),

            scale: 15.0,
            scroll: Time::ZERO,
            raw_current_beat: Time::ZERO,
            raw_left: None,
            raw_right: None,
            raw_replay: None,
            level: Level::new(),
            selected_light: None,
            selected_waypoint: None,
        }
    }

    pub fn get_scale(&self) -> f32 {
        self.scale
    }

    pub fn rescale(&mut self, new_scale: f32) {
        if new_scale.approx_eq(&0.0) {
            return;
        }

        // scroll so that current beat stays in-place
        let min = self.state.position.min.x;
        let current = self.current_beat.position.center().x;
        self.scroll = r32(current - min) / r32(new_scale) - self.raw_current_beat;

        self.scale = new_scale;
        self.reload(None);
    }

    // pub fn auto_scale(&mut self, max_beat: Time) {
    //     let scale = self.state.position.width() / max_beat.as_f32().max(1.0);
    //     self.scale = scale;
    //     self.reload();
    // }

    pub fn visible_scroll(&self) -> Time {
        r32(self.state.position.width() / self.scale)
    }

    pub fn get_scroll(&self) -> Time {
        self.scroll
    }

    pub fn scroll(&mut self, delta: Time) {
        self.scroll += delta;
        self.reload(None);
    }

    pub fn update_time(&mut self, current_beat: Time, replay: Option<Time>) {
        self.raw_current_beat = current_beat;
        self.raw_replay = replay;
        self.reload(None);

        // Auto scroll if current beat goes off screen
        let margin = 50.0;
        let margin_beats = r32(margin / self.scale);

        let min = margin;
        let max = self.state.position.width() - margin;
        let current = self.current_beat.position.center().x - self.state.position.min.x;
        if current < min && self.raw_current_beat > margin_beats {
            self.scroll(r32((min - current) / self.scale));
        } else if current > max {
            self.scroll(r32((max - current) / self.scale));
        }
        self.reload(None);
    }

    pub fn start_selection(&mut self) {
        self.raw_left = Some(self.raw_current_beat);
        self.reload(None);
    }

    /// Finishes the selection and returns the left and right boundaries in ascending order.
    pub fn end_selection(&mut self) -> (Time, Time) {
        let right = self.raw_current_beat;
        self.raw_right = Some(right);
        self.reload(None);

        let left = self.raw_left.unwrap_or(right);
        if left < right {
            (left, right)
        } else {
            (right, left)
        }
    }

    pub fn clear_selection(&mut self) {
        self.raw_left = None;
        self.raw_right = None;
        self.reload(None);
    }

    fn reload(&mut self, mut editor: Option<&mut LevelEditor>) {
        let render_time = |time: Time| {
            let pos = (time + self.scroll).as_f32() * self.scale;
            let pos = vec2(
                self.state.position.min.x + pos,
                self.state.position.center().y,
            );
            Aabb2::point(pos).extend_symmetric(vec2(0.1, 0.5) * self.context.font_size / 2.0)
        };
        self.current_beat
            .update(render_time(self.raw_current_beat), &self.context);

        self.lights.clear();
        self.waypoints.clear();
        self.selected.hide();
        let height = self.context.font_size * 0.4;
        for (i, event) in self.level.events.iter().enumerate() {
            if let Event::Light(light) = &event.event {
                let time = event.beat + light.telegraph.precede_time;
                let light_id = LightId { event: i };
                if Some(light_id) == self.selected_light {
                    let from = render_time(time).center();
                    let to = render_time(time + light.light.movement.total_duration())
                        .center()
                        .x;
                    let selected = Aabb2::point(from)
                        .extend_right(to - from.x)
                        .extend_symmetric(vec2(0.0, 0.1) * self.context.font_size / 2.0);
                    self.selected.show();
                    self.selected.update(selected, &self.context);

                    let size = vec2(0.25, 0.5) * self.context.font_size;
                    for (waypoint_id, _, offset) in light.light.movement.timed_positions() {
                        let mut state = WidgetState::new();
                        let position = render_time(time + offset).center();
                        let position = Aabb2::point(position).extend_symmetric(size / 2.0);
                        state.update(position, &self.context);
                        if state.clicked {
                            if let Some(editor) = &mut editor {
                                editor.select_waypoint(light_id, waypoint_id);
                            }
                        }
                        self.waypoints.push((waypoint_id, state));
                    }
                }

                let lights = self.lights.entry(time).or_default();

                let mut state = WidgetState::new();
                let position = render_time(time).center();
                let position = Aabb2::point(position)
                    .extend_symmetric(vec2(height, 0.0) / 2.0)
                    .extend_down(height)
                    .translate(-vec2(0.0, height * (lights.len() as f32 + 0.2)));
                state.update(position, &self.context);
                if state.clicked {
                    if let Some(editor) = &mut editor {
                        editor.selected_light = Some(light_id);
                        editor.level_state.waypoints = None;
                    }
                }

                lights.push((light_id, state));
            }
        }

        let render_option = |widget: &mut WidgetState, time: Option<Time>| match time {
            Some(time) => {
                widget.show();
                widget.update(render_time(time), &self.context);
            }
            None => widget.hide(),
        };
        render_option(&mut self.left, self.raw_left);
        render_option(&mut self.right, self.raw_right);
        render_option(&mut self.replay, self.raw_replay);
    }

    pub fn get_cursor_time(&self) -> Time {
        r32((self.context.cursor.position.x - self.state.position.min.x) / self.scale) - self.scroll
    }
}

impl StatefulWidget for TimelineWidget {
    type State = LevelEditor;

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext, state: &mut Self::State) {
        self.state.update(position, context);
        let clickable =
            position.extend_symmetric(vec2(0.0, context.font_size * 0.2 - position.height()) / 2.0);
        self.clickable.update(clickable, context);
        let bar = Aabb2::point(clickable.center())
            .extend_symmetric(vec2(clickable.width(), context.font_size * 0.1) / 2.0);
        self.bar.update(bar, context);

        self.context = context.clone();
        self.level = state.level.clone();
        self.selected_light = state.selected_light;
        self.selected_waypoint = state
            .level_state
            .waypoints
            .as_ref()
            .and_then(|waypoints| waypoints.selected);
        self.reload(Some(state));
    }
}
