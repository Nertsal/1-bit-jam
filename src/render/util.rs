use super::*;

use crate::ui::UiContext;

pub struct UtilRender {
    geng: Geng,
    assets: Rc<Assets>,
    pub unit_quad: ugli::VertexBuffer<draw2d::TexturedVertex>,
}

#[derive(Debug, Clone, Copy)]
pub struct TextRenderOptions {
    pub size: f32,
    pub align: vec2<f32>,
    pub color: Color,
    pub hover_color: Color,
    pub press_color: Color,
    pub rotation: Angle,
}

#[derive(Debug, Clone, Copy)]
pub struct DashRenderOptions {
    pub width: f32,
    pub color: Color,
    pub dash_length: f32,
    pub space_length: f32,
}

impl TextRenderOptions {
    pub fn new(size: f32) -> Self {
        Self { size, ..default() }
    }

    // pub fn size(self, size: f32) -> Self {
    //     Self { size, ..self }
    // }

    pub fn align(self, align: vec2<f32>) -> Self {
        Self { align, ..self }
    }

    pub fn color(self, color: Color) -> Self {
        Self { color, ..self }
    }

    pub fn update(&mut self, context: &UiContext) {
        self.size = context.font_size;
        self.color = context.theme().light;
        self.hover_color = self.color.map_rgb(|x| x * 0.7);
        self.press_color = self.color.map_rgb(|x| x * 0.5);
    }
}

impl Default for TextRenderOptions {
    fn default() -> Self {
        Self {
            size: 1.0,
            align: vec2::splat(0.5),
            color: Color::WHITE,
            hover_color: Color {
                r: 0.7,
                g: 0.7,
                b: 0.7,
                a: 1.0,
            },
            press_color: Color {
                r: 0.5,
                g: 0.5,
                b: 0.5,
                a: 1.0,
            },
            rotation: Angle::ZERO,
        }
    }
}

impl UtilRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            unit_quad: geng_utils::geometry::unit_quad_geometry(geng.ugli()),
        }
    }

    pub fn draw_nine_slice(
        &self,
        pos: Aabb2<f32>,
        color: Color,
        texture: &ugli::Texture,
        scale: f32,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let whole = Aabb2::ZERO.extend_positive(vec2::splat(1.0));

        // TODO: configurable
        let mid = Aabb2 {
            min: vec2(0.3, 0.3),
            max: vec2(0.7, 0.7),
        };

        let size = mid.min * texture.size().as_f32() * scale;
        let size = vec2(size.x.min(pos.width()), size.y.min(pos.height()));

        let tl = Aabb2::from_corners(mid.top_left(), whole.top_left());
        let tm = Aabb2::from_corners(mid.top_left(), vec2(mid.max.x, whole.max.y));
        let tr = Aabb2::from_corners(mid.top_right(), whole.top_right());
        let rm = Aabb2::from_corners(mid.top_right(), vec2(whole.max.x, mid.min.y));
        let br = Aabb2::from_corners(mid.bottom_right(), whole.bottom_right());
        let bm = Aabb2::from_corners(mid.bottom_right(), vec2(mid.min.x, whole.min.y));
        let bl = Aabb2::from_corners(mid.bottom_left(), whole.bottom_left());
        let lm = Aabb2::from_corners(mid.bottom_left(), vec2(whole.min.x, mid.max.y));

        let slices: Vec<draw2d::TexturedVertex> = [tl, tm, tr, rm, br, bm, bl, lm, mid]
            .into_iter()
            .flat_map(|slice| {
                let [a, b, c, d] = slice.corners().map(|a_vt| {
                    let a_pos = vec2(
                        if a_vt.x == mid.min.x {
                            pos.min.x + size.x
                        } else if a_vt.x == mid.max.x {
                            pos.max.x - size.x
                        } else {
                            pos.min.x + pos.width() * a_vt.x
                        },
                        if a_vt.y == mid.min.y {
                            pos.min.y + size.y
                        } else if a_vt.y == mid.max.y {
                            pos.max.y - size.y
                        } else {
                            pos.min.y + pos.height() * a_vt.y
                        },
                    );
                    draw2d::TexturedVertex {
                        a_pos,
                        a_color: Color::WHITE,
                        a_vt,
                    }
                });
                [a, b, c, a, c, d]
            })
            .collect();
        let slices = ugli::VertexBuffer::new_dynamic(self.geng.ugli(), slices);

        ugli::draw(
            framebuffer,
            &self.assets.shaders.texture,
            ugli::DrawMode::Triangles,
            &slices,
            (
                ugli::uniforms! {
                    u_model_matrix: mat3::identity(),
                    u_color: color,
                    u_texture: texture,
                },
                camera.uniforms(framebuffer.size().as_f32()),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                ..default()
            },
        );

        // self.geng
        //     .draw2d()
        //     .textured_quad(framebuffer, camera, pos, texture, color);
    }

    pub fn draw_text(
        &self,
        text: impl AsRef<str>,
        position: vec2<impl Float>,
        options: TextRenderOptions,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let text = text.as_ref();
        let font = &self.assets.fonts.pixel;

        let measure = font
            .measure(text, vec2::splat(geng::TextAlign::CENTER))
            .unwrap_or(Aabb2::ZERO.extend_positive(vec2::splat(1.0)));
        let size = measure.size();
        let align = size * (options.align - vec2::splat(0.5)); // Centered by default
        let align = vec2(measure.center().x, 0.0) + align;

        let position = position.map(Float::as_f32);
        let transform = mat3::translate(position.map(Float::as_f32))
            * mat3::scale_uniform(options.size * 0.6) // TODO: figure out what that 0.6 is lmao
            * mat3::translate(-align.rotate(options.rotation))
            * mat3::rotate_around(vec2(measure.center().x, 0.0), options.rotation);

        let framebuffer_size = framebuffer.size();

        font.draw_with(
            text,
            vec2::splat(geng::TextAlign::CENTER),
            |glyphs, texture| {
                ugli::draw(
                    framebuffer,
                    &self.assets.shaders.sdf,
                    ugli::DrawMode::TriangleFan,
                    ugli::instanced(
                        &self.unit_quad,
                        &ugli::VertexBuffer::new_dynamic(self.geng.ugli(), glyphs.to_vec()),
                    ),
                    (
                        ugli::uniforms! {
                            u_texture: texture,
                            u_model_matrix: transform,
                            u_color: options.color,
                            u_smooth: 0.0,
                            u_outline_dist: 0.0 / font.max_distance(),
                            u_outline_color: Color::TRANSPARENT_BLACK,
                        },
                        camera.uniforms(framebuffer_size.map(|x| x as f32)),
                    ),
                    ugli::DrawParameters {
                        blend_mode: Some(ugli::BlendMode::straight_alpha()),
                        depth_func: None,
                        ..Default::default()
                    },
                );
            },
        );
    }

    pub fn draw_light(
        &self,
        light: &Light,
        color: Color,
        dark_color: Color,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.draw_light_gradient(&light.collider, color, camera, framebuffer);

        // Waypoint visual
        let radius_max = 0.2;
        let width = 0.05;
        let fade_in = 0.25;
        let fade_out = 0.5;

        let t = light.closest_waypoint.0.as_f32();
        let t = (t / fade_in + 1.0).min(1.0 - t / fade_out).max(0.0);
        let radius = r32(t * radius_max);

        if radius.as_f32() < width {
            return;
        }

        let shape = match light.collider.shape {
            Shape::Circle { .. } => Shape::circle(radius),
            Shape::Line { .. } => Shape::line(radius / r32(2.0)),
            Shape::Rectangle { .. } => Shape::rectangle(vec2::splat(radius)),
        };
        let waypoint = Collider {
            shape,
            ..light.collider
        };
        self.draw_outline(&waypoint, width, dark_color, camera, framebuffer);
    }

    pub fn draw_light_gradient(
        &self,
        collider: &Collider,
        color: Color,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let (texture, transform) = match collider.shape {
            Shape::Circle { radius } => (
                &self.assets.sprites.radial_gradient,
                mat3::scale_uniform(radius.as_f32()),
            ),
            Shape::Line { width } => {
                let inf = 999.0;
                (
                    &self.assets.sprites.linear_gradient,
                    mat3::scale(vec2(inf, width.as_f32()) / 2.0),
                )
            }
            Shape::Rectangle { width, height } => (
                &self.assets.sprites.linear_gradient,
                mat3::scale(vec2(width.as_f32(), height.as_f32()) / 2.0),
            ),
        };
        let transform = mat3::translate(collider.position.as_f32())
            * mat3::rotate(collider.rotation.map(Coord::as_f32))
            * transform;

        let framebuffer_size = framebuffer.size();
        ugli::draw(
            framebuffer,
            &self.assets.shaders.light,
            ugli::DrawMode::TriangleFan,
            &self.unit_quad,
            (
                ugli::uniforms! {
                    u_model_matrix: transform,
                    u_color: color,
                    u_texture: texture,
                },
                camera.uniforms(framebuffer_size.as_f32()),
            ),
            ugli::DrawParameters {
                blend_mode: Some(additive()),
                ..default()
            },
        );
    }

    pub fn draw_outline(
        &self,
        collider: &Collider,
        outline_width: f32,
        color: Rgba<f32>,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        match collider.shape {
            Shape::Circle { radius } => {
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Ellipse::circle_with_cut(
                        collider.position.as_f32(),
                        radius.as_f32() - outline_width,
                        radius.as_f32(),
                        color,
                    ),
                );
            }
            Shape::Line { width } => {
                let inf = 1e3; // camera.fov;
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Segment::new(
                        Segment(
                            vec2(-inf * 2.0, (width.as_f32() - outline_width) / 2.0),
                            vec2(inf * 2.0, (width.as_f32() - outline_width) / 2.0),
                        ),
                        outline_width,
                        color,
                    )
                    .rotate(collider.rotation.map(Coord::as_f32))
                    .translate(collider.position.as_f32()),
                );
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Segment::new(
                        Segment(
                            vec2(-inf * 2.0, -(width.as_f32() - outline_width) / 2.0),
                            vec2(inf * 2.0, -(width.as_f32() - outline_width) / 2.0),
                        ),
                        outline_width,
                        color,
                    )
                    .rotate(collider.rotation.map(Coord::as_f32))
                    .translate(collider.position.as_f32()),
                );
            }
            Shape::Rectangle { width, height } => {
                let [a, b, c, d] = Aabb2::ZERO
                    .extend_symmetric(vec2(width.as_f32(), height.as_f32()) / 2.0)
                    .extend_uniform(-outline_width / 2.0)
                    .corners();
                let m = (a + b) / 2.0;
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Chain::new(
                        Chain::new(vec![m, b, c, d, a, m]),
                        outline_width,
                        color,
                        1,
                    )
                    .rotate(collider.rotation.map(Coord::as_f32))
                    .translate(collider.position.as_f32()),
                );
            }
        }
    }

    pub fn draw_button(
        &self,
        button: &HoverButton,
        text: impl AsRef<str>,
        theme: &Theme,
        camera: &Camera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let t = button.hover_time.get_ratio();
        let scale = button.animation.get(t).scale;
        let collider = button
            .base_collider
            .transformed(Transform { scale, ..default() });
        self.draw_light_gradient(&collider, theme.light, camera, framebuffer);

        if t.as_f32() < 0.5 {
            self.draw_text(
                text,
                collider.position,
                TextRenderOptions::new(1.0).color(theme.dark),
                camera,
                framebuffer,
            );
        }
    }

    pub fn draw_dashed_chain(
        &self,
        chain: &Chain<f32>,
        options: &DashRenderOptions,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let mut dash_full_left = 0.0;
        for segment in chain.segments() {
            dash_full_left =
                self.draw_dashed_segment(segment, options, dash_full_left, camera, framebuffer);
        }
    }

    /// Draws a dashed segment.
    /// Returns the unrendered length of the last dash.
    pub fn draw_dashed_segment(
        &self,
        mut segment: Segment<f32>,
        options: &DashRenderOptions,
        dash_full_left: f32,
        camera: &impl geng::AbstractCamera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) -> f32 {
        let delta = segment.1 - segment.0;
        let delta_len = delta.len();
        let direction_norm = if delta.len().approx_eq(&0.0) {
            return dash_full_left;
        } else {
            delta / delta_len
        };

        if dash_full_left > 0.0 {
            // Finish drawing the previous dash and offset current segment
            let dash_full_length = dash_full_left.min(delta_len);
            let dash_length = dash_full_left - options.space_length;
            if dash_length > 0.0 {
                // Finish dash
                let dash_length = dash_length.min(dash_full_length);
                let dash_end = segment.0 + direction_norm * dash_length;
                self.geng.draw2d().draw2d(
                    framebuffer,
                    camera,
                    &draw2d::Chain::new(
                        Chain::new(vec![segment.0, dash_end]),
                        options.width,
                        options.color,
                        1,
                    ),
                );
            }

            // Finish space
            let dash_left = dash_full_left - dash_full_length;
            if dash_left > 0.0 {
                return dash_left;
            }

            // Offset
            segment.0 += dash_full_length * direction_norm
        }

        let full_length = options.dash_length + options.space_length;

        // Recalculate delta
        let delta_len = (segment.1 - segment.0).len();
        let dashes = (delta_len / full_length).floor() as usize;
        for i in 0..dashes {
            let dash_start = segment.0 + direction_norm * i as f32 * full_length;
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Chain::new(
                    Chain::new(vec![
                        dash_start,
                        dash_start + direction_norm * options.dash_length,
                    ]),
                    options.width,
                    options.color,
                    1,
                ),
            );
        }

        let last_start = segment.0 + direction_norm * dashes as f32 * full_length;
        let last_len = (segment.1 - last_start).len();
        let dash_len = last_len.min(options.dash_length);
        let last_end = last_start + direction_norm * dash_len;
        self.geng.draw2d().draw2d(
            framebuffer,
            camera,
            &draw2d::Chain::new(
                Chain::new(vec![last_start, last_end]),
                options.width,
                options.color,
                1,
            ),
        );
        full_length - last_len
    }

    pub fn draw_player(
        &self,
        player: &Player,
        camera: &Camera2d,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        // Player tail
        for tail in &player.tail {
            let radius = r32(0.1) * tail.lifetime.get_ratio();
            let collider = Collider::new(tail.pos, Shape::Circle { radius });
            let (in_color, out_color) = match tail.state {
                LitState::Dark => (THEME.danger, THEME.dark),
                LitState::Light => (THEME.dark, THEME.light),
                LitState::Danger => (THEME.light, THEME.danger),
            };
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Ellipse::circle(tail.pos.as_f32(), radius.as_f32(), in_color),
            );
            self.draw_outline(&collider, 0.05, out_color, camera, framebuffer);
        }

        // Player
        self.draw_outline(&player.collider, 0.05, THEME.light, camera, framebuffer);
    }

    pub fn draw_health(
        &self,
        health: &Lifetime,
        state: LitState,
        // theme: &Theme,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let theme = THEME;

        let camera = &geng::PixelPerfectCamera;
        let screen = Aabb2::ZERO.extend_positive(framebuffer.size().as_f32());
        let font_size = screen.height() * 0.05;

        let aabb = Aabb2::point(
            geng_utils::layout::aabb_pos(screen, vec2(0.5, 1.0)) - vec2(0.0, 1.0) * font_size,
        )
        .extend_symmetric(vec2(14.0, 0.0) * font_size / 2.0)
        .extend_up(font_size);

        // Outline
        // self.draw_outline(
        //     &Collider::aabb(aabb.map(r32)),
        //     font_size * 0.2,
        //     theme.light,
        //     camera,
        //     framebuffer,
        // );
        self.geng.draw2d().draw2d(
            framebuffer,
            camera,
            &draw2d::Quad::new(aabb.extend_uniform(font_size * 0.1), theme.light),
        );

        // Dark fill
        self.geng
            .draw2d()
            .draw2d(framebuffer, camera, &draw2d::Quad::new(aabb, theme.dark));

        // Health fill
        let color = match state {
            LitState::Light => crate::util::with_alpha(theme.light, 1.0),
            LitState::Dark => crate::util::with_alpha(theme.light, 0.7),
            LitState::Danger => crate::util::with_alpha(theme.danger, 0.7),
        };
        // self.geng.draw2d().draw2d(
        //     framebuffer,
        //     camera,
        //     &draw2d::Quad::new(
        //         aabb.extend_symmetric(
        //             vec2((health.get_ratio().as_f32() - 1.0) * aabb.width(), 0.0) / 2.0,
        //         ),
        //         color,
        //     ),
        // );
        let framebuffer_size = framebuffer.size();
        let aabb = aabb
            .extend_symmetric(vec2((health.get_ratio().as_f32() - 1.0) * aabb.width(), 0.0) / 2.0);
        let transform = mat3::translate(aabb.center()) * mat3::scale(aabb.size() / 2.0);
        ugli::draw(
            framebuffer,
            &self.assets.shaders.solid,
            ugli::DrawMode::TriangleFan,
            &self.unit_quad,
            (
                ugli::uniforms! {
                    u_model_matrix: transform,
                    u_color: color,
                },
                camera.uniforms(framebuffer_size.as_f32()),
            ),
            ugli::DrawParameters {
                blend_mode: None,
                ..default()
            },
        );
    }
}

pub fn additive() -> ugli::BlendMode {
    ugli::BlendMode::combined(ugli::ChannelBlendMode {
        src_factor: ugli::BlendFactor::One,
        dst_factor: ugli::BlendFactor::One,
        equation: ugli::BlendEquation::Add,
    })
}
