use super::*;

use crate::ui::layout::AreaOps;

use ctl_client::core::types::Name;

pub struct InputWidget {
    pub state: WidgetState,
    pub name: TextWidget,
    pub text: TextWidget,
    pub edit_id: Option<usize>,
    pub raw: String,
    pub editing: bool,

    pub hide_input: bool,
    pub format: InputFormat,
    pub layout_vertical: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum InputFormat {
    Any,
    // Integer,
    Float,
}

impl InputFormat {
    pub fn fix(&self, s: &str) -> String {
        match self {
            InputFormat::Any => s.to_owned(),
            // InputFormat::Integer => s.replace(|c: char| !c.is_ascii_digit(), ""),
            InputFormat::Float => {
                let (s, negative) = match s.strip_prefix("-") {
                    Some(s) => (s, true),
                    None => (s, false),
                };
                let mut s = s.replace(|c: char| c != '.' && !c.is_ascii_digit(), "");
                if let Some((a, b)) = s.split_once('.') {
                    s = a.to_owned() + "." + &b.replace('.', "");
                }
                if negative {
                    s = "-".to_string() + &s;
                }
                s
            }
        }
    }
}

impl InputWidget {
    pub fn new(name: impl Into<Name>) -> Self {
        Self {
            state: WidgetState::new(),
            name: TextWidget::new(name).aligned(vec2(0.5, 0.5)),
            text: TextWidget::new("").aligned(vec2(0.5, 0.5)),
            edit_id: None,
            raw: String::new(),
            editing: false,

            hide_input: false,
            format: InputFormat::Any,
            layout_vertical: false,
        }
    }

    pub fn format(self, format: InputFormat) -> Self {
        Self { format, ..self }
    }

    pub fn vertical(self) -> Self {
        Self {
            layout_vertical: true,
            ..self
        }
    }

    // pub fn hide_input(self) -> Self {
    //     Self {
    //         hide_input: true,
    //         ..self
    //     }
    // }

    pub fn sync(&mut self, text: &str, context: &mut UiContext) {
        if self.raw == text {
            return;
        }

        text.clone_into(&mut self.raw);
        self.text.text = self.raw.clone().into();

        self.editing = self
            .edit_id
            .map_or(false, |id| context.text_edit.is_active(id));
        if self.editing {
            self.edit_id = Some(context.text_edit.edit(&self.raw));
        }
    }
}

impl Widget for InputWidget {
    fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn update(&mut self, position: Aabb2<f32>, context: &mut UiContext) {
        self.state.update(position, context);

        if self.state.clicked {
            self.edit_id = Some(context.text_edit.edit(&self.text.text));
        }

        self.editing = if self
            .edit_id
            .map_or(false, |id| context.text_edit.is_active(id))
        {
            if self.raw != context.text_edit.text {
                self.raw.clone_from(&context.text_edit.text);
                self.raw = self.format.fix(&self.raw);
                self.edit_id = Some(context.text_edit.edit(&self.raw));

                self.text.text = if self.hide_input {
                    "*".repeat(self.raw.len()).into()
                } else {
                    self.raw.clone().into()
                };
            }
            true
        } else {
            false
        };

        let mut main = position;

        if self.layout_vertical {
            if !self.name.text.is_empty() {
                let name = main.split_top(0.5);
                self.name.update(name, context);
            }
            self.text.align(vec2(0.5, 0.5));
            self.text.update(main, context);
        } else {
            if !self.name.text.is_empty() {
                let name_width = (context.layout_size * 5.0).min(main.width() / 2.0);
                let name = main.cut_left(name_width);
                self.name.update(name, context);
                self.text.align(vec2(1.0, 0.5));
            } else {
                self.text.align(vec2(0.5, 0.5));
            }
            self.text.update(main, context);
        }
    }
}
