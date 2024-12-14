use crate::render::util::TextRenderOptions;
use crate::render::{dither::DitherRender, util::UtilRender};

use crate::prelude::*;

pub struct MediaState {
    context: Context,
    util_render: UtilRender,
    dither: DitherRender,
    text: String,
    theme: Theme,
    time: Time,
    camera: Camera2d,
}

impl MediaState {
    pub fn new(context: Context) -> Self {
        Self {
            util_render: UtilRender::new(&context.geng, &context.assets),
            dither: DitherRender::new(&context.geng, &context.assets),
            text: String::new(),
            theme: Theme::default(),
            time: Time::ZERO,
            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            context,
        }
    }

    pub fn with_text(self, text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..self
        }
    }
}

impl geng::State for MediaState {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(self.theme.dark), None, None);

        let mut dither_buffer = self.dither.start();

        self.util_render.draw_text(
            &self.text,
            vec2(0.0, 0.0),
            TextRenderOptions::new(1.0).color(crate::render::THEME.light),
            &self.camera,
            &mut dither_buffer,
        );

        self.dither.finish(self.time, &self.theme);

        let aabb = Aabb2::ZERO.extend_positive(framebuffer.size().as_f32());
        geng_utils::texture::DrawTexture::new(self.dither.get_buffer())
            .fit(aabb, vec2(0.5, 0.5))
            .draw(&geng::PixelPerfectCamera, &self.context.geng, framebuffer);
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as f32);
        self.time += delta_time;
    }
}
