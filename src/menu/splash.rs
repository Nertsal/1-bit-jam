use super::*;

const TRANSITION_TIME: f32 = 5.0;

pub struct SplashScreen {
    context: Context,
    client: Option<Arc<ctl_client::Nertboard>>,
    transition: Option<geng::state::Transition>,

    util: UtilRender,

    time: Time,
}

impl SplashScreen {
    pub fn new(context: Context, client: Option<&Arc<ctl_client::Nertboard>>) -> Self {
        Self {
            util: UtilRender::new(&context.geng, &context.assets),

            time: Time::ZERO,

            context,
            client: client.cloned(),
            transition: None,
        }
    }
}

impl geng::State for SplashScreen {
    fn transition(&mut self) -> Option<geng::state::Transition> {
        self.transition.take()
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let theme = self.context.get_options().theme;

        ugli::clear(framebuffer, Some(theme.dark), None, None);

        let camera = &Camera2d {
            center: vec2::ZERO,
            rotation: Angle::ZERO,
            fov: 12.0,
        };

        let alpha = (TRANSITION_TIME - self.time.as_f32()).clamp(0.0, 1.0);
        let alpha = crate::util::smoothstep(alpha);
        let color = crate::util::with_alpha(theme.light, alpha);

        self.util.draw_text(
            "PHOTOSENSITIVITY WARNING",
            vec2(0.0, 0.7),
            TextRenderOptions::new(1.3)
                .align(vec2(0.5, 0.0))
                .color(color),
            camera,
            framebuffer,
        );
        let warning = "
This game contains flashing lights which might

trigger seizures for people with photosensitive epilepsy
            ";
        self.util.draw_text(
            warning,
            vec2(0.0, 0.0),
            TextRenderOptions::new(0.8)
                .align(vec2(0.5, 1.0))
                .color(color),
            camera,
            framebuffer,
        );
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as f32);
        self.time += delta_time;

        if self.time.as_f32() > TRANSITION_TIME {
            self.transition = Some(geng::state::Transition::Switch(Box::new(MainMenu::new(
                self.context.clone(),
                self.client.as_ref(),
            ))));
        }
    }

    fn handle_event(&mut self, event: geng::Event) {
        if geng_utils::key::is_event_press(&event, [geng::Key::Space]) {
            self.time = r32(TRANSITION_TIME - 1.0);
        }
    }
}
