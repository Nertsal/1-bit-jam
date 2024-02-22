mod button;
mod checkbox;
mod explore;
mod group;
mod icon;
mod input;
mod leaderboard;
mod level;
mod level_config;
mod options;
mod profile;
mod slider;
mod text;
mod timeline;
mod value;

pub use self::{
    button::*, checkbox::*, explore::*, group::*, icon::*, input::*, leaderboard::*, level::*,
    level_config::*, options::*, profile::*, slider::*, text::*, timeline::*, value::*,
};

use super::{context::*, window::*};

use geng::prelude::*;

pub trait Widget {
    /// Update position and related properties.
    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext);
    /// Apply a function to all states contained in the widget.
    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState));
    /// Make the widget visible.
    fn show(&mut self) {
        self.walk_states_mut(&WidgetState::show)
    }
    /// Hide the widget and disable interactions.
    fn hide(&mut self) {
        self.walk_states_mut(&WidgetState::hide)
    }
}

pub trait StatefulWidget {
    /// The external state that the widget operates on and can modify.
    type State;

    /// Update position and related properties.
    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext, state: &mut Self::State);
    /// Apply a function to all states contained in the widget.
    fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState));
    /// Make the widget visible.
    fn show(&mut self) {
        self.walk_states_mut(&WidgetState::show)
    }
    /// Hide the widget and disable interactions.
    fn hide(&mut self) {
        self.walk_states_mut(&WidgetState::hide)
    }
}

// impl<T: Widget> StatefulWidget for T {
//     type State = ();

//     fn update(&mut self, position: Aabb2<f32>, context: &UiContext, state: &mut Self::State) {
//         self.update(position, context)
//     }

//     fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
//         self.walk_states_mut(f)
//     }
// }

#[derive(Debug, Clone)]
pub struct WidgetState {
    pub position: Aabb2<f32>,
    /// Whether to show the widget.
    pub visible: bool,
    pub hovered: bool,
    /// Whether user has clicked on the widget since last frame.
    pub clicked: bool,
    /// Whether user is holding the mouse button down on the widget.
    pub pressed: bool,
}

impl WidgetState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.position = position;
        if self.visible && context.can_focus {
            self.hovered = self.position.contains(context.cursor.position);
            let was_pressed = self.pressed;
            // TODO: check for mouse being pressed and then dragged onto the widget
            self.pressed =
                context.cursor.down && (was_pressed || self.hovered && !context.cursor.was_down);
            self.clicked = !was_pressed && self.pressed;
        } else {
            self.hovered = false;
            self.pressed = false;
            self.clicked = false;
        }
    }

    /// For compatibility with [Widget::walk_states_mut].
    pub fn walk_states_mut(&mut self, f: &dyn Fn(&mut WidgetState)) {
        f(self)
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.hovered = false;
        self.pressed = false;
        self.clicked = false;
    }
}

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            position: Aabb2::ZERO.extend_uniform(1.0),
            visible: true,
            hovered: false,
            clicked: false,
            pressed: false,
        }
    }
}
