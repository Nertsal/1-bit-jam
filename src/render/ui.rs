use super::{mask::MaskedRender, util::UtilRender, *};

use crate::ui::{layout::AreaOps, widget::*};

pub fn pixel_scale(framebuffer: &ugli::Framebuffer) -> f32 {
    const TARGET_SIZE: vec2<usize> = vec2(640, 360);
    let size = framebuffer.size().as_f32();
    let ratio = size / TARGET_SIZE.as_f32();
    ratio.x.min(ratio.y)
}

pub struct UiRender {
    context: Context,
    pub util: UtilRender,
}

impl UiRender {
    pub fn new(context: Context) -> Self {
        Self {
            util: UtilRender::new(&context.geng, &context.assets),
            context,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_window(
        &self,
        masked: &mut MaskedRender,
        main: Aabb2<f32>,
        head: Option<Aabb2<f32>>,
        outline_width: f32,
        theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
        inner: impl FnOnce(&mut ugli::Framebuffer),
    ) {
        // let size = main.size().map(|x| x.abs().ceil() as usize);
        // let mut texture = ugli::Texture::new_with(self.geng.ugli(), size, |_| theme.dark);

        let mut mask = masked.start();

        // Fill
        if let Some(head) = head {
            self.draw_quad(head, theme.dark, framebuffer);
            mask.mask_quad(head);
        }
        mask.mask_quad(main);
        self.draw_quad(main.extend_uniform(-outline_width), theme.dark, framebuffer);

        inner(&mut mask.color);
        masked.draw(draw_parameters(), framebuffer);

        // Outline
        if let Some(head) = head {
            let delta = main.center() - head.center(); // TODO: more precise
            let dir = if delta.x.abs() > delta.y.abs() {
                vec2(1.0, 0.0)
            } else {
                vec2(0.0, 1.0)
            };
            let mut low = 0.0;
            let mut high = outline_width;
            if vec2::dot(dir, delta) < 0.0 {
                std::mem::swap(&mut low, &mut high);
            }
            self.draw_outline(
                head.extend_left(dir.x * low)
                    .extend_right(dir.x * high)
                    .extend_down(dir.y * low)
                    .extend_up(dir.y * high),
                outline_width,
                theme.light,
                framebuffer,
            );
        }
        self.draw_outline(main, outline_width, theme.light, framebuffer);
        if let Some(head) = head {
            let delta = main.center() - head.center(); // TODO: more precise
            let dir = if delta.x.abs() > delta.y.abs() {
                vec2(1.0, 0.0)
            } else {
                vec2(0.0, 1.0)
            };
            let mut low = -1.0 * outline_width;
            let mut high = 3.0 * outline_width;
            if vec2::dot(dir, delta) < 0.0 {
                std::mem::swap(&mut low, &mut high);
            }
            self.draw_quad(
                head.extend_uniform(-outline_width)
                    .extend_left(dir.x * low)
                    .extend_right(dir.x * high)
                    .extend_down(dir.y * low)
                    .extend_up(dir.y * high),
                theme.dark,
                framebuffer,
            );
        }
    }

    pub fn draw_texture(
        &self,
        quad: Aabb2<f32>,
        texture: &ugli::Texture,
        color: Color,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let size = texture.size() * pixel_scale(framebuffer).round() as usize;
        // let pos = crate::ui::layout::align_aabb(size, quad, vec2(0.5, 0.5));
        let pos = geng_utils::pixel::pixel_perfect_aabb(
            quad.center(),
            vec2(0.5, 0.5),
            size,
            &geng::PixelPerfectCamera,
            framebuffer.size().as_f32(),
        );
        self.context.geng.draw2d().textured_quad(
            framebuffer,
            &geng::PixelPerfectCamera,
            pos,
            texture,
            color,
        );
    }

    pub fn draw_outline(
        &self,
        quad: Aabb2<f32>,
        width: f32,
        color: Color,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let scale = pixel_scale(framebuffer);
        let (texture, real_width) = if width < 2.0 * scale {
            (&self.context.assets.sprites.border_thinner, 1.0 * scale)
        } else if width < 4.0 * scale {
            (&self.context.assets.sprites.border_thin, 2.0 * scale)
        } else {
            (&self.context.assets.sprites.border, 4.0 * scale)
        };
        self.util.draw_nine_slice(
            quad.extend_uniform(real_width - width),
            color,
            texture,
            scale,
            &geng::PixelPerfectCamera,
            framebuffer,
        );
    }

    pub fn draw_quad(
        &self,
        quad: Aabb2<f32>,
        color: Rgba<f32>,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.context.geng.draw2d().draw2d(
            framebuffer,
            &geng::PixelPerfectCamera,
            &draw2d::Quad::new(quad, color),
        );
    }

    pub fn draw_icon_button(
        &self,
        icon: &IconButtonWidget,
        theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if !icon.state.visible {
            return;
        }
        self.draw_icon(&icon.icon, theme, framebuffer);
    }

    pub fn draw_icon(&self, icon: &IconWidget, theme: Theme, framebuffer: &mut ugli::Framebuffer) {
        if !icon.state.visible {
            return;
        }

        if let Some(bg) = &icon.background {
            match bg.kind {
                IconBackgroundKind::NineSlice => {
                    let texture = //if width < 5.0 {
                        &self.context.assets.sprites.fill_thin;
                    // } else {
                    //     &self.assets.sprites.fill
                    // };
                    self.util.draw_nine_slice(
                        icon.state.position,
                        theme.get_color(bg.color),
                        texture,
                        pixel_scale(framebuffer),
                        &geng::PixelPerfectCamera,
                        framebuffer,
                    );
                }
                IconBackgroundKind::Circle => {
                    self.draw_texture(
                        icon.state.position,
                        &self.context.assets.sprites.circle,
                        theme.get_color(bg.color),
                        framebuffer,
                    );
                }
            }
        }
        self.draw_texture(
            icon.state.position,
            &icon.texture,
            theme.get_color(icon.color),
            framebuffer,
        );
    }

    pub fn draw_checkbox(
        &self,
        widget: &CheckboxWidget,
        theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if !widget.state.visible {
            return;
        }

        let camera = &geng::PixelPerfectCamera;
        let size = widget.state.position.height();

        let checkbox = widget.check.position;
        if widget.checked {
            let checkbox = checkbox.extend_uniform(-size * 0.05);
            for (a, b) in [
                (checkbox.bottom_left(), checkbox.top_right()),
                (checkbox.top_left(), checkbox.bottom_right()),
            ] {
                self.context.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Segment::new(Segment(a, b), size * 0.07, theme.light),
                );
            }
        }
        self.util.draw_outline(
            &Collider::aabb(checkbox.map(r32)),
            size * 0.1,
            theme.light,
            camera,
            framebuffer,
        );
        self.draw_text(&widget.text, framebuffer);
    }

    // TODO: as text render option
    pub fn draw_text_wrapped(&self, widget: &TextWidget, framebuffer: &mut ugli::Framebuffer) {
        if !widget.state.visible {
            return;
        }

        let main = widget.state.position;
        let lines = crate::util::wrap_text(
            &self.context.assets.fonts.pixel,
            &widget.text,
            main.width() / widget.options.size / 0.6, // Magic constant from the util renderer that scales everything by 0.6 idk why
        );
        let row = main.align_aabb(vec2(main.width(), widget.options.size), vec2(0.5, 1.0));
        let rows = row.stack(vec2(0.0, -row.height()), lines.len());

        for (line, position) in lines.into_iter().zip(rows) {
            self.util.draw_text(
                line,
                position.align_pos(widget.options.align),
                widget.options,
                &geng::PixelPerfectCamera,
                framebuffer,
            );
        }
    }

    pub fn draw_text(&self, widget: &TextWidget, framebuffer: &mut ugli::Framebuffer) {
        self.draw_text_colored(widget, widget.options.color, framebuffer)
    }

    pub fn draw_text_colored(
        &self,
        widget: &TextWidget,
        color: Color,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if !widget.state.visible {
            return;
        }

        // Fit to area
        let mut widget = widget.clone();

        let font = &self.context.assets.fonts.pixel;
        let measure = font
            .measure(&widget.text, vec2::splat(geng::TextAlign::CENTER))
            .unwrap_or(Aabb2::ZERO.extend_positive(vec2(1.0, 1.0)));

        let size = widget.state.position.size();
        let right = vec2(size.x, 0.0).rotate(widget.options.rotation).x;
        let left = vec2(0.0, size.y).rotate(widget.options.rotation).x;
        let width = if left.signum() != right.signum() {
            left.abs() + right.abs()
        } else {
            left.abs().max(right.abs())
        };

        let max_width = width * 0.9; // Leave some space TODO: move into a parameter or smth
        let max_size = max_width / measure.width() / 0.6; // Magic constant from the util renderer that scales everything by 0.6 idk why
        let size = widget.options.size.min(max_size);

        widget.options.size = size;

        self.util.draw_text(
            &widget.text,
            geng_utils::layout::aabb_pos(widget.state.position, widget.options.align),
            widget.options.color(color),
            &geng::PixelPerfectCamera,
            framebuffer,
        );
    }

    pub fn draw_slider(
        &self,
        slider: &SliderWidget,
        mut theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if slider.state.hovered {
            std::mem::swap(&mut theme.dark, &mut theme.light);
            self.fill_quad(slider.state.position, theme.dark, framebuffer);
        }

        self.draw_text_colored(&slider.text, theme.light, framebuffer);
        self.draw_text_colored(&slider.value, theme.light, framebuffer);

        if slider.bar.visible {
            self.context.geng.draw2d().quad(
                framebuffer,
                &geng::PixelPerfectCamera,
                slider.bar.position,
                theme.light,
            );
        }

        if slider.head.visible {
            let color = theme.light;
            self.context.geng.draw2d().quad(
                framebuffer,
                &geng::PixelPerfectCamera,
                slider.head.position,
                color,
            );
        }
    }

    pub fn draw_button(
        &self,
        widget: &ButtonWidget,
        theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let state = &widget.text.state;
        if !state.visible {
            return;
        }

        let options = &widget.text.options;
        let position = widget.text.state.position;
        let width = options.size * 0.2;

        let mut text_color = theme.light;
        if state.pressed {
            self.fill_quad(position, theme.light, framebuffer);
            text_color = theme.dark;
        } else if state.hovered {
            self.fill_quad(position.extend_uniform(width), theme.light, framebuffer);
            text_color = theme.dark;
        } else {
            self.draw_outline(position, width, theme.light, framebuffer);
        };

        self.draw_text_colored(&widget.text, text_color, framebuffer);
    }

    pub fn draw_input(&self, widget: &InputWidget, framebuffer: &mut ugli::Framebuffer) {
        if !widget.state.visible {
            return;
        }
        let theme = self.context.get_options().theme;

        self.draw_text(&widget.name, framebuffer);
        let color = if widget.editing {
            theme.highlight
        } else {
            theme.light
        };
        self.draw_text_colored(&widget.text, color, framebuffer);

        if !widget.editing && widget.state.hovered {
            // Underline
            let mut pos = widget.text.state.position;
            let underline = pos.cut_bottom(pos.height() * 0.05);
            self.draw_quad(underline, theme.highlight, framebuffer);
        }
    }

    pub fn draw_notification(
        &self,
        notification: &NotificationWidget,
        outline_width: f32,
        theme: Theme,
        masked: &mut MaskedRender,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let window = notification.state.position;
        self.draw_window(
            masked,
            window,
            None,
            outline_width,
            theme,
            framebuffer,
            |framebuffer| {
                self.draw_text_wrapped(&notification.text, framebuffer);
                self.draw_icon(&notification.confirm.icon, theme, framebuffer);
            },
        );
    }

    pub fn draw_confirm(
        &self,
        confirm: &ConfirmWidget,
        outline_width: f32,
        theme: Theme,
        masked: &mut MaskedRender,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let t = crate::util::smoothstep(confirm.window.show.time.get_ratio());

        let window = confirm.state.position;
        let min_height = outline_width * 10.0;
        let height = (t * window.height()).max(min_height);

        let window = window.with_height(height, 1.0);
        self.draw_window(
            masked,
            window,
            None,
            outline_width,
            theme,
            framebuffer,
            |framebuffer| {
                let title = confirm.title.state.position;
                self.fill_quad(title, theme.light, framebuffer);
                self.draw_text_colored(&confirm.title, theme.dark, framebuffer);
                self.draw_text(&confirm.message, framebuffer);
                self.draw_icon(&confirm.confirm.icon, theme, framebuffer);
                self.draw_icon(&confirm.discard.icon, theme, framebuffer);
            },
        );
    }

    pub fn draw_leaderboard(
        &self,
        leaderboard: &LeaderboardWidget,
        theme: Theme,
        outline_width: f32,
        masked: &mut MaskedRender,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let camera = &geng::PixelPerfectCamera;

        self.context.geng.draw2d().draw2d(
            framebuffer,
            camera,
            &draw2d::Quad::new(leaderboard.state.position, theme.dark),
        );
        // self.draw_icon(&leaderboard.close.icon, framebuffer);
        if leaderboard.reload.state.visible {
            self.draw_icon(&leaderboard.reload.icon, theme, framebuffer);
        }
        self.draw_text(&leaderboard.title, framebuffer);
        self.draw_text(&leaderboard.subtitle, framebuffer);
        self.draw_text(&leaderboard.status, framebuffer);

        self.draw_quad(
            leaderboard.separator_title.position,
            theme.light,
            framebuffer,
        );

        let mut buffer = masked.start();

        buffer.mask_quad(leaderboard.rows_state.position);

        for row in &leaderboard.rows {
            self.draw_text(&row.rank, &mut buffer.color);
            self.draw_text(&row.player, &mut buffer.color);
            self.draw_text(&row.score, &mut buffer.color);
            self.draw_outline(
                row.state.position,
                outline_width,
                theme.light,
                &mut buffer.color,
            );
            for icon in &row.modifiers {
                self.draw_icon(icon, theme, &mut buffer.color);
            }
        }

        masked.draw(draw_parameters(), framebuffer);

        self.draw_quad(
            leaderboard.separator_highscore.position,
            theme.light,
            framebuffer,
        );

        if leaderboard.highscore.state.visible {
            self.draw_text(&leaderboard.highscore.rank, framebuffer);
            self.draw_text(&leaderboard.highscore.player, framebuffer);
            self.draw_text(&leaderboard.highscore.score, framebuffer);
            for icon in &leaderboard.highscore.modifiers {
                self.draw_icon(icon, theme, framebuffer);
            }
        }
    }

    pub fn draw_toggle_button(
        &self,
        text: &TextWidget,
        selected: bool,
        can_deselect: bool,
        theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let state = &text.state;
        if !state.visible {
            return;
        }

        let (bg_color, fg_color) = if selected {
            (theme.light, theme.dark)
        } else {
            (theme.dark, theme.light)
        };

        let width = text.options.size * 0.2;
        let shrink = if can_deselect && state.hovered && selected {
            width
        } else {
            0.0
        };
        let pos = state.position.extend_uniform(-shrink);
        self.draw_quad(pos.extend_uniform(-width), bg_color, framebuffer);
        if state.hovered || selected {
            self.draw_outline(pos, width, theme.light, framebuffer);
        }
        self.util.draw_text(
            &text.text,
            geng_utils::layout::aabb_pos(state.position, text.options.align),
            text.options.color(fg_color),
            &geng::PixelPerfectCamera,
            framebuffer,
        );
    }

    pub fn draw_toggle_widget(
        &self,
        toggle: &ToggleWidget,
        theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.draw_toggle_button(
            &toggle.text,
            toggle.selected,
            toggle.can_deselect,
            theme,
            framebuffer,
        );
    }

    // TODO: more general name
    pub fn draw_toggle(
        &self,
        text: &TextWidget,
        width: f32,
        theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.draw_toggle_slide(
            &text.state,
            &[text],
            width,
            text.state.hovered,
            theme,
            framebuffer,
        )
    }

    // TODO: more general name
    pub fn draw_toggle_slide(
        &self,
        state: &WidgetState,
        texts: &[&TextWidget],
        width: f32,
        selected: bool,
        theme: Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if !state.visible {
            return;
        }

        let (bg_color, fg_color) = if selected {
            (theme.light, theme.dark)
        } else {
            (theme.dark, theme.light)
        };

        self.draw_quad(state.position.extend_uniform(-width), bg_color, framebuffer);

        for text in texts {
            self.draw_text_colored(text, fg_color, framebuffer);
            self.draw_text_colored(text, fg_color, framebuffer);
        }

        self.draw_outline(state.position, width, theme.light, framebuffer);
    }

    pub fn draw_value<T>(&self, value: &ValueWidget<T>, framebuffer: &mut ugli::Framebuffer) {
        if !value.state.visible {
            return;
        }

        self.draw_text(&value.text, framebuffer);
        self.draw_text(&value.value_text, framebuffer);
    }

    pub fn fill_quad(
        &self,
        position: Aabb2<f32>,
        color: Color,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let size = position.size();
        let size = size.x.min(size.y);

        let scale = ui::pixel_scale(framebuffer);

        let texture = if size < 48.0 * scale {
            &self.context.assets.sprites.fill_thin
        } else {
            &self.context.assets.sprites.fill
        };
        self.util.draw_nine_slice(
            position,
            color,
            texture,
            scale,
            &geng::PixelPerfectCamera,
            framebuffer,
        );
    }

    pub fn draw_profile(&self, ui: &ProfileWidget, framebuffer: &mut ugli::Framebuffer) {
        let theme = self.context.get_options().theme;
        self.draw_text(&ui.offline, framebuffer);

        let register = &ui.register;
        if register.state.visible {
            // self.draw_input(&register.username, framebuffer);
            // self.draw_input(&register.password, framebuffer);
            // self.draw_button(&register.login, framebuffer);
            // self.draw_button(&register.register, framebuffer);
            self.draw_text(&register.login_with, framebuffer);
            self.draw_icon(&register.discord.icon, theme, framebuffer);
        }

        let logged = &ui.logged;
        if logged.state.visible {
            self.draw_text(&logged.username, framebuffer);
            self.draw_toggle_button(&logged.logout, false, false, theme, framebuffer);
        }
    }
}
