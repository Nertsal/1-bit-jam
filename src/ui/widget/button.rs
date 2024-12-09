use super::*;

use crate::{model::ThemeColor, ui::layout::AreaOps};

use ctl_client::core::types::Name;

#[derive(Clone)]
pub struct ButtonWidget {
    pub text: TextWidget,
    pub bg_color: ThemeColor,
}

impl ButtonWidget {
    pub fn new(text: impl Into<Name>) -> Self {
        Self {
            text: TextWidget::new(text),
            bg_color: ThemeColor::Light,
        }
    }

    pub fn color(mut self, bg_color: ThemeColor) -> Self {
        self.bg_color = bg_color;
        self
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.text.update(position, context);
        self.text.options.color = context.theme().dark;
    }
}

impl WidgetOld for ButtonWidget {
    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.text.update(position, context);
    }

    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.text.state
    }
}

impl Widget for ButtonWidget {
    fn draw(&self, context: &UiContext) -> Geometry {
        let theme = context.theme();
        let state = &self.text.state;
        let width = self.text.options.size * 0.2;

        let position = state.position;
        let bg_color = theme.get_color(self.bg_color);
        let mut geometry = if state.pressed {
            context
                .geometry
                .quad_fill(position.extend_uniform(-width), bg_color)
        } else if state.hovered {
            context
                .geometry
                .quad_fill(position.extend_uniform(-width * 0.5), bg_color)
        } else {
            context.geometry.quad_fill(position, bg_color)
        };

        geometry.merge(self.text.draw(context));
        geometry
    }
}

#[derive(Clone)]
pub struct IconButtonWidget {
    pub state: WidgetState,
    pub icon: IconWidget,
    pub light_color: ThemeColor,
}

impl IconButtonWidget {
    pub fn new(
        texture: &Rc<ugli::Texture>,
        light_color: ThemeColor,
        bg_kind: IconBackgroundKind,
    ) -> Self {
        let mut icon = IconWidget::new(texture);
        icon.background = Some(IconBackground {
            color: ThemeColor::Dark,
            kind: bg_kind,
        });
        Self {
            state: WidgetState::new(),
            icon,
            light_color,
        }
    }

    pub fn new_normal(texture: &Rc<ugli::Texture>) -> Self {
        Self::new(texture, ThemeColor::Light, IconBackgroundKind::NineSlice)
    }

    pub fn new_danger(texture: &Rc<ugli::Texture>) -> Self {
        Self::new(texture, ThemeColor::Danger, IconBackgroundKind::NineSlice)
    }

    pub fn new_close_button(texture: &Rc<ugli::Texture>) -> Self {
        Self::new(texture, ThemeColor::Danger, IconBackgroundKind::Circle)
    }
}

impl WidgetOld for IconButtonWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);
        self.icon.update(position, context);

        let mut light = self.light_color;
        let mut dark = ThemeColor::Dark;
        if self.state.hovered {
            std::mem::swap(&mut dark, &mut light);
        }

        self.icon.color = light;
        if let Some(bg) = &mut self.icon.background {
            bg.color = dark;
        }
    }
}

#[derive(Clone, Default)]
pub struct ToggleButtonWidget {
    pub text: TextWidget,
    pub selected: bool,
    pub can_deselect: bool,
}

impl ToggleButtonWidget {
    pub fn new(text: impl Into<Name>) -> Self {
        Self {
            text: TextWidget::new(text),
            selected: false,
            can_deselect: false,
        }
    }

    pub fn new_deselectable(text: impl Into<Name>) -> Self {
        Self {
            text: TextWidget::new(text),
            selected: false,
            can_deselect: true,
        }
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        self.text.update(position, context);
        if self.text.state.clicked {
            if self.can_deselect {
                self.selected = !self.selected;
            } else {
                self.selected = true;
            }
        }
    }
}

impl WidgetOld for ToggleButtonWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.text.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.text.update(position, context);
        if self.text.state.clicked {
            if self.can_deselect {
                self.selected = !self.selected;
            } else {
                self.selected = true;
            }
        }
    }
}

impl Widget for ToggleButtonWidget {
    fn draw(&self, context: &UiContext) -> Geometry {
        let theme = context.theme();
        let width = self.text.options.size * 0.2;
        let (bg_color, fg_color) = if self.selected {
            (theme.light, theme.dark)
        } else {
            (theme.dark, theme.light)
        };

        let mut geometry = context
            .geometry
            .quad_fill(self.text.state.position, bg_color);
        geometry.merge(self.text.draw_colored(context, fg_color));
        geometry.merge(
            context
                .geometry
                .quad_outline(self.text.state.position, width, theme.light),
        );
        geometry
    }
}

pub struct ToggleWidget {
    pub state: WidgetState,
    pub text: TextWidget,
    pub tick: WidgetState,
    pub checked: bool,
    pub checked_color: ThemeColor,
}

impl ToggleWidget {
    pub fn new(text: impl Into<Name>) -> Self {
        Self {
            state: WidgetState::new(),
            text: TextWidget::new(text).aligned(vec2(0.0, 0.5)),
            tick: WidgetState::new(),
            checked: false,
            checked_color: ThemeColor::Highlight,
        }
    }

    pub fn color(mut self, select_color: ThemeColor) -> Self {
        self.checked_color = select_color;
        self
    }

    pub fn update(&mut self, position: Aabb2<f32>, context: &UiContext) {
        let mut main = position;
        self.state.update(main, context);
        let size = main.height();
        let tick = main.cut_right(size).extend_uniform(-size / 5.0);
        self.tick.update(tick, context);
        self.text.update(main, context);
    }
}

impl Widget for ToggleWidget {
    fn draw(&self, context: &UiContext) -> Geometry {
        let theme = context.theme();
        let width = self.text.options.size * 0.1;
        let mut geometry = self.text.draw(context);
        if self.checked {
            geometry.merge(context.geometry.quad_fill(
                self.tick.position.extend_uniform(-width),
                theme.get_color(self.checked_color),
            ));
        }
        geometry.merge(
            context
                .geometry
                .quad_outline(self.tick.position, width, theme.light),
        );
        geometry
    }
}
