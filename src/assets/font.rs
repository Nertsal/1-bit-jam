use super::*;

use crate::render::util::TextRenderOptions;

use geng_utils::conversions::Vec2RealConversions;

#[derive(ugli::Vertex, Debug, Clone, Copy)]
struct Vertex {
    a_pos: vec2<f32>,
    a_vt: vec2<f32>,
}

pub struct Font {
    font: rusttype::Font<'static>,
    cache: RefCell<rusttype::gpu_cache::Cache<'static>>,
    cache_texture: RefCell<ugli::Texture>,
    geometry: RefCell<ugli::VertexBuffer<Vertex>>,
    program: ugli::Program,
    descent: f32,
}

const CACHE_SIZE: usize = 4096;

impl Font {
    pub fn new(manager: &geng::asset::Manager, data: Vec<u8>) -> anyhow::Result<Font> {
        let font = rusttype::Font::try_from_vec(data)
            .ok_or_else(|| anyhow::Error::msg("Failed to read font"))?;
        let metrics = font.v_metrics(rusttype::Scale { x: 1.0, y: 1.0 });
        Ok(Font {
            font,
            cache: RefCell::new(
                rusttype::gpu_cache::Cache::builder()
                    .dimensions(CACHE_SIZE as u32, CACHE_SIZE as u32)
                    .scale_tolerance(0.1)
                    .position_tolerance(0.1)
                    .build(),
            ),
            cache_texture: RefCell::new({
                let mut texture = ugli::Texture2d::new_uninitialized(
                    manager.ugli(),
                    vec2(CACHE_SIZE, CACHE_SIZE),
                );
                texture.set_filter(ugli::Filter::Nearest);
                texture
            }),
            geometry: RefCell::new(ugli::VertexBuffer::new_dynamic(manager.ugli(), Vec::new())),
            program: manager
                .shader_lib()
                .compile(include_str!("font.glsl"))
                .unwrap(),
            descent: metrics.descent,
        })
    }
    pub fn measure_at(&self, text: &str, mut pos: vec2<f32>, size: f32) -> Aabb2<f32> {
        pos.y -= self.descent * size;
        let scale = rusttype::Scale { x: size, y: size };
        let pos = rusttype::Point {
            x: pos.x,
            y: -pos.y,
        };
        let mut result: Option<Aabb2<f32>> = None;
        for glyph in self.font.layout(text, scale, pos) {
            if let Some(bb) = glyph.pixel_bounding_box() {
                if let Some(cur) = result {
                    result = Some(Aabb2::from_corners(
                        vec2(
                            partial_min(bb.min.x as f32, cur.min.x),
                            partial_min(bb.min.y as f32, cur.min.y),
                        ),
                        vec2(
                            partial_max(bb.max.x as f32, cur.max.x),
                            partial_max(bb.max.y as f32, cur.max.y),
                        ),
                    ));
                } else {
                    result = Some(Aabb2::from_corners(
                        vec2(bb.min.x as f32, bb.min.y as f32),
                        vec2(bb.max.x as f32, bb.max.y as f32),
                    ));
                }
            }
        }
        let mut result = result.unwrap_or(Aabb2::ZERO);
        let (bottom, top) = (-result.max.y, -result.min.y);
        result.min.y = bottom;
        result.max.y = top;
        result
    }
    pub fn measure(&self, text: &str, size: f32) -> Aabb2<f32> {
        self.measure_at(text, vec2(0.0, 0.0), size)
    }
    pub fn draw_with(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        text: impl AsRef<str>,
        size: f32,
        color: Rgba<f32>,
        transform: mat3<f32>,
        params: ugli::DrawParameters,
    ) {
        let text = text.as_ref();

        let mut pos = vec2::ZERO;
        pos.y -= self.descent * size;
        let scale = rusttype::Scale { x: size, y: size };
        let pos = rusttype::Point {
            x: pos.x,
            y: framebuffer.size().y as f32 - pos.y,
        };

        let mut cache = self.cache.borrow_mut();
        let mut cache_texture = self.cache_texture.borrow_mut();

        let glyphs: Vec<_> = text
            .chars()
            .map(|c| self.font.glyph(c))
            .scan((None, 0.0), |(last, x), g| {
                let g = g.scaled(scale);
                if let Some(last) = last {
                    *x += self.font.pair_kerning(scale, *last, g.id());
                }
                let w = g.h_metrics().advance_width;
                let next = g.positioned(pos + rusttype::vector(*x, 0.0));
                *last = Some(next.id());
                *x += w;
                Some(next)
            })
            .collect();
        for glyph in &glyphs {
            cache.queue_glyph(0, glyph.clone());
        }

        cache
            .cache_queued(|rect, data| {
                let x = rect.min.x as usize;
                let y = rect.min.y as usize;
                let width = rect.width() as usize;
                let height = rect.height() as usize;

                let mut rgba_data = vec![0xff; data.len() * 4];
                for i in 0..data.len() {
                    rgba_data[i * 4 + 3] = data[i];
                }

                cache_texture.sub_image(vec2(x, y), vec2(width, height), &rgba_data);
            })
            .unwrap();

        let mut geometry = self.geometry.borrow_mut();
        geometry.clear();
        for glyph in &glyphs {
            if let Some((texture_rect, rect)) = cache.rect_for(0, glyph).unwrap() {
                let x1 = rect.min.x as f32;
                let y1 = rect.min.y as f32;
                let x2 = rect.max.x as f32;
                let y2 = rect.max.y as f32;
                let u1 = texture_rect.min.x;
                let u2 = texture_rect.max.x;
                let v1 = texture_rect.min.y;
                let v2 = texture_rect.max.y;

                let a = Vertex {
                    a_pos: vec2(x1, y1),
                    a_vt: vec2(u1, v1),
                };
                let b = Vertex {
                    a_pos: vec2(x2, y1),
                    a_vt: vec2(u2, v1),
                };
                let c = Vertex {
                    a_pos: vec2(x2, y2),
                    a_vt: vec2(u2, v2),
                };
                let d = Vertex {
                    a_pos: vec2(x1, y2),
                    a_vt: vec2(u1, v2),
                };
                geometry.extend([a, b, c, a, c, d]);
            }
        }

        let framebuffer_size = framebuffer.size();

        ugli::draw(
            framebuffer,
            &self.program,
            ugli::DrawMode::Triangles,
            &*geometry,
            ugli::uniforms! {
                u_color: color,
                u_cache_texture: &*cache_texture,
                u_framebuffer_size: framebuffer_size,
                u_model_matrix: transform
            },
            params,
        );
    }
    pub fn draw(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        camera: &impl geng::AbstractCamera2d,
        text: impl AsRef<str>,
        position: vec2<impl Float>,
        mut options: TextRenderOptions,
    ) {
        let text = text.as_ref();
        let framebuffer_size = framebuffer.size().as_f32();

        let position = position.map(Float::as_f32);
        let position = world_to_screen(camera, framebuffer_size, position);

        let scale = world_to_screen(
            camera,
            framebuffer_size,
            vec2::splat(std::f32::consts::FRAC_1_SQRT_2),
        ) - world_to_screen(camera, framebuffer_size, vec2::ZERO);
        options.size *= scale.len() * 0.6; // TODO: could rescale all dependent code but whatever

        let measure = self.measure(text, options.size);
        let size = measure.size();
        let align = size * (options.align - vec2::splat(0.5)); // Centered by default
        let descent = -self.descent * options.size;
        let align = vec2(
            measure.center().x + align.x,
            descent + (measure.max.y - descent) * options.align.y,
        );

        let transform =
            mat3::translate(position) * mat3::rotate(options.rotation) * mat3::translate(-align);

        self.draw_with(
            framebuffer,
            text,
            options.size,
            options.color,
            transform,
            ugli::DrawParameters {
                depth_func: None,
                blend_mode: Some(ugli::BlendMode::straight_alpha()),
                ..default()
            },
        );
    }
}

impl geng::asset::Load for Font {
    type Options = ();

    fn load(
        manager: &geng::asset::Manager,
        path: &std::path::Path,
        &(): &Self::Options,
    ) -> geng::asset::Future<Self> {
        let manager = manager.clone();
        let path = path.to_owned();
        async move {
            let data = file::load_bytes(path).await?;
            let font = Self::new(&manager, data)?;
            Ok(font)
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some("ttf");
}

fn world_to_screen(
    camera: &impl geng::AbstractCamera2d,
    framebuffer_size: vec2<f32>,
    pos: vec2<f32>,
) -> vec2<f32> {
    let pos = (camera.projection_matrix(framebuffer_size) * camera.view_matrix()) * pos.extend(1.0);
    let pos = pos.xy() / pos.z;
    vec2(
        (pos.x + 1.0) / 2.0 * framebuffer_size.x,
        (pos.y + 1.0) / 2.0 * framebuffer_size.y,
    )
}
