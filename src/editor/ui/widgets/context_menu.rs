use super::*;

pub struct ContextMenuWidget {
    pub extension: SecondOrderState<f32>,
    pub state: WidgetState,
    pub options: Vec<OptionWidget>,
}

pub struct OptionWidget {
    pub state: WidgetState,
    pub text: TextWidget,
    pub action: EditorStateAction,
}

impl ContextMenuWidget {
    pub fn empty() -> Self {
        Self {
            extension: SecondOrderState::new(SecondOrderDynamics::new(3.0, 1.0, 0.0, 0.0)),
            state: WidgetState::new(),
            options: vec![],
        }
    }

    pub fn new(
        position: vec2<f32>,
        options: impl IntoIterator<Item = (impl Into<Name>, EditorStateAction)>,
    ) -> Self {
        let mut extension = SecondOrderState::new(SecondOrderDynamics::new(3.0, 1.0, 0.0, 0.0));
        extension.target = 1.0;

        let mut state = WidgetState::new();
        state.position = Aabb2::point(position);

        Self {
            extension,
            state,
            options: options
                .into_iter()
                .map(|(text, action)| OptionWidget::new(text, action))
                .collect(),
        }
    }

    // pub fn open(&mut self) {
    //     self.extension.target = 1.0;
    // }

    pub fn is_open(&self) -> bool {
        self.extension.target > 0.0
    }

    pub fn close(&mut self) {
        self.extension.target = 0.0;
    }

    pub fn update(&mut self, actions: &mut Vec<EditorStateAction>, context: &UiContext) {
        let size = vec2(7.0, -1.6 * self.options.len() as f32) * context.font_size;
        let position = self.state.position.top_left();
        let position = Aabb2::from_corners(position, position + size);
        self.state.update(position, context);

        self.extension.update(context.delta_time);

        let position = position.extend_uniform(-context.font_size * 0.2);
        let rows = position.split_rows(self.options.len());
        for (row, widget) in rows.into_iter().zip(&mut self.options) {
            widget.update(row, actions, context);
        }

        context.update_focus(self.state.hovered);
    }
}

impl OptionWidget {
    pub fn new(text: impl Into<Name>, action: EditorStateAction) -> Self {
        Self {
            state: WidgetState::new(),
            text: TextWidget::new(text).aligned(vec2(0.0, 0.5)),
            action,
        }
    }

    pub fn update(
        &mut self,
        position: Aabb2<f32>,
        actions: &mut Vec<EditorStateAction>,
        context: &UiContext,
    ) {
        self.state.update(position, context);
        self.text
            .update(position.extend_uniform(-0.3 * context.font_size), context);
        if self.state.clicked {
            actions.extend([self.action.clone(), EditorStateAction::CloseContextMenu]);
        }
    }
}

impl Widget for ContextMenuWidget {
    fn draw(&self, context: &UiContext) -> Geometry {
        let mut position = self.state.position;
        let position = position.cut_top(self.extension.current * position.height());

        const MIN_SIZE: f32 = 5.0;
        if position.height() < MIN_SIZE || position.width() < MIN_SIZE {
            return Geometry::new();
        }

        let theme = context.theme();
        let width = context.font_size * 0.2;

        let mut geometry = Geometry::new();
        for option in &self.options {
            geometry.merge(option.draw(context));
        }

        let mut geometry = context.geometry.masked(position, geometry);
        geometry.merge(context.geometry.quad_outline(position, width, theme.light));
        geometry.merge(
            context
                .geometry
                .quad_fill(position.extend_uniform(width), theme.dark),
        );
        geometry.change_z_index(1000);
        geometry
    }
}

impl Widget for OptionWidget {
    fn draw(&self, context: &UiContext) -> Geometry {
        let mut geometry = Geometry::new();
        let width = self.text.options.size * 0.1;
        let theme = context.theme();
        let mut fg_color = theme.light;
        let mut bg_color = theme.dark;

        if self.state.hovered {
            std::mem::swap(&mut fg_color, &mut bg_color);
        }

        geometry.merge(self.text.draw_colored(context, fg_color));

        let position = self.state.position;
        geometry.merge(if self.state.pressed {
            context
                .geometry
                .quad_fill(position.extend_uniform(-width * 0.5), bg_color)
        } else {
            context.geometry.quad_fill(position, bg_color)
        });

        geometry
    }
}
