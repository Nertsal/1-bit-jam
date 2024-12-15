use super::*;

use crate::{
    leaderboard::Leaderboard,
    render::{ui::UiRender, THEME},
    ui::{layout::AreaOps, widget::*, UiContext},
};

pub struct MainMenu {
    context: Context,
    leaderboard: Leaderboard,
    transition: Option<geng::state::Transition>,

    dither: DitherRender,
    util_render: UtilRender,
    ui_render: UiRender,

    framebuffer_size: vec2<usize>,
    /// Cursor position in screen space.
    cursor_pos: vec2<f64>,
    /// Cursor clicked last frame.
    clicked: bool,
    active_touch: Option<u64>,
    cursor_world_pos: vec2<Coord>,
    camera: Camera2d,

    time: Time,
    play_button: HoverButton,
    player: Player,

    ui: MainUI,
    ui_context: UiContext,
}

struct MainUI {
    screen: WidgetState,
    join_community: TextWidget,
    join_discord: IconButtonWidget,
    profile: ProfileWidget,
}

impl MainMenu {
    pub fn new(context: Context, client: Option<&Arc<ctl_client::Nertboard>>) -> Self {
        let leaderboard = Leaderboard::new(&context.geng, client);

        Self {
            dither: DitherRender::new(&context.geng, &context.assets),
            util_render: UtilRender::new(&context.geng, &context.assets),
            ui_render: UiRender::new(context.clone()),
            leaderboard,

            framebuffer_size: vec2(1, 1),
            cursor_pos: vec2::ZERO,
            active_touch: None,
            cursor_world_pos: vec2::ZERO,
            clicked: false,
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },

            time: Time::ZERO,
            play_button: HoverButton::new(
                Collider {
                    position: vec2(0.0, 0.0).as_r32(),
                    rotation: Angle::ZERO,
                    shape: Shape::Circle { radius: r32(1.0) },
                },
                1.5,
            ),
            player: Player::new(
                Collider::new(vec2::ZERO, Shape::Circle { radius: r32(0.1) }),
                r32(0.0),
            ),

            ui: MainUI::new(context.clone()),
            ui_context: UiContext::new(context.clone()),

            context,
            transition: None,
        }
    }

    fn play(&mut self) {
        let context = self.context.clone();
        let state = LevelMenu::new(context, self.leaderboard.clone());
        self.transition = Some(geng::state::Transition::Push(Box::new(state)));
    }

    fn draw_ui(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let theme = self.context.get_options().theme;
        let ui = &self.ui;

        self.ui_render.draw_text(&ui.join_community, framebuffer);
        self.ui_render
            .draw_icon_button(&ui.join_discord, theme, framebuffer);
        self.ui_render.draw_profile(&ui.profile, framebuffer);
    }
}

impl geng::State for MainMenu {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as f32);
        self.time += delta_time;

        self.context
            .geng
            .window()
            .set_cursor_type(geng::CursorType::None);

        self.ui_context
            .update(self.context.geng.window(), delta_time.as_f32());

        self.context.music.stop(); // TODO: menu music

        self.leaderboard.poll();

        let pos = self.cursor_pos.as_f32();
        let game_pos = geng_utils::layout::fit_aabb(
            self.dither.get_render_size().as_f32(),
            Aabb2::ZERO.extend_positive(self.framebuffer_size.as_f32()),
            vec2(0.5, 0.5),
        );
        let pos = pos - game_pos.bottom_left();
        self.cursor_world_pos = self.camera.screen_to_world(game_pos.size(), pos).as_r32();

        self.player.collider.position = self.cursor_world_pos;
        self.player.reset_distance();

        let hovering = self.player.collider.check(&self.play_button.base_collider);
        if hovering && self.clicked {
            self.play_button.clicked = true;
        }
        self.play_button.update(hovering, delta_time);
        self.player
            .update_distance_simple(&self.play_button.base_collider);
        if self.play_button.hover_time.is_max() {
            self.play_button.hover_time.set_ratio(Time::ZERO);
            self.play();
        }

        self.clicked = false;
    }

    fn fixed_update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as _);
        self.player.update_tail(delta_time);
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::KeyPress {
                key: geng::Key::F11,
            } => self.context.geng.window().toggle_fullscreen(),
            geng::Event::Wheel { delta } => {
                self.ui_context.cursor.scroll += delta as f32;
            }
            geng::Event::CursorMove { position } => {
                self.cursor_pos = position;
                self.ui_context.cursor.cursor_move(position.as_f32());
            }
            geng::Event::MousePress {
                button: geng::MouseButton::Left,
            } => self.clicked = true,
            geng::Event::TouchStart(touch) if self.active_touch.is_none() => {
                self.active_touch = Some(touch.id);
            }
            geng::Event::TouchMove(touch) if Some(touch.id) == self.active_touch => {
                self.cursor_pos = touch.position;
            }
            geng::Event::TouchEnd(touch) if Some(touch.id) == self.active_touch => {
                self.active_touch = None;
            }
            _ => {}
        }
    }

    fn draw(&mut self, screen_buffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = screen_buffer.size();
        let theme = self.context.get_options().theme;
        ugli::clear(screen_buffer, Some(theme.dark), None, None);

        let mut framebuffer = self.dither.start();

        let button = crate::render::smooth_button(&self.play_button, self.time + r32(0.5));
        self.util_render
            .draw_button(&button, "START", &THEME, &self.camera, &mut framebuffer);

        self.util_render.draw_text(
            "made in rust btw",
            vec2(0.0, -4.0),
            TextRenderOptions::new(0.5).color(THEME.dark),
            &self.camera,
            &mut framebuffer,
        );

        let fading = self.play_button.is_fading();
        if !fading {
            if let Some(pos) = self
                .camera
                .world_to_screen(framebuffer.size().as_f32(), vec2(0.0, 3.5))
            {
                self.ui_render.draw_texture(
                    Aabb2::point(pos).extend_symmetric(vec2(0.0, 1.2) / 2.0),
                    &self.context.assets.sprites.title,
                    THEME.light,
                    &mut framebuffer,
                );
            }

            self.util_render
                .draw_player(&self.player, &self.camera, &mut framebuffer);
        }

        self.dither.finish(self.time, &theme);

        let aabb = Aabb2::ZERO.extend_positive(screen_buffer.size().as_f32());
        geng_utils::texture::DrawTexture::new(self.dither.get_buffer())
            .fit(aabb, vec2(0.5, 0.5))
            .draw(&geng::PixelPerfectCamera, &self.context.geng, screen_buffer);

        if !fading {
            self.ui.layout(
                Aabb2::ZERO.extend_positive(screen_buffer.size().as_f32()),
                &mut self.ui_context,
                &mut self.leaderboard,
            );

            self.draw_ui(screen_buffer);
        }

        self.ui_context.frame_end();
    }
}

impl MainUI {
    pub fn new(context: Context) -> Self {
        Self {
            screen: WidgetState::new(),
            join_community: TextWidget::new("Join our community!"),
            join_discord: IconButtonWidget::new_normal(&context.assets.sprites.discor),
            profile: ProfileWidget::new(&context.assets),
        }
    }

    pub fn layout(
        &mut self,
        screen: Aabb2<f32>,
        context: &mut UiContext,
        leaderboard: &mut Leaderboard,
    ) {
        // Fix aspect
        let screen = screen.fit_aabb(vec2(16.0, 9.0), vec2::splat(0.5));

        let layout_size = screen.height() * 0.03;
        let font_size = screen.height() * 0.06;

        context.screen = screen;
        context.layout_size = layout_size;
        context.font_size = font_size;

        self.screen.update(screen, context);

        let join = vec2(6.0, 3.0) * font_size;
        let mut join = screen
            .align_aabb(join, vec2(0.0, 0.0))
            .translate(vec2(1.0, 1.0) * layout_size);
        let text = join.cut_top(font_size * 1.5);
        self.join_community.update(text, context);
        self.join_discord.update(join, context);
        if self.join_discord.state.clicked {
            let _ = webbrowser::open(crate::DISCORD_SERVER_URL);
        }

        let profile = vec2(6.0, 3.0) * font_size;
        let profile = screen
            .align_aabb(profile, vec2(1.0, 0.0))
            .translate(vec2(-1.0, 1.0) * layout_size);
        self.profile.update(profile, context, leaderboard);
    }
}
