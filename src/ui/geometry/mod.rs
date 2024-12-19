use crate::{
    assets::{Assets, PixelTexture},
    render::util::TextRenderOptions,
};

use ctl_client::core::{prelude::Color, types::Name};
use geng::prelude::*;
use geng_utils::conversions::Vec2RealConversions;

#[derive(Clone)]
pub struct GeometryContext {
    pub assets: Rc<Assets>,
    pub framebuffer_size: vec2<usize>,
    pub pixel_scale: f32,
    z_index: RefCell<i32>,
    // TODO: texture atlas i guess
}

#[derive(Default, Debug)]
pub struct Geometry {
    pub triangles: Vec<GeometryTriangleVertex>,
    // TODO: texture atlas and move to triangles
    pub textures: Vec<GeometryTexture>,
    pub text: Vec<GeometryText>,
    pub masked: Vec<MaskedGeometry>,
}

#[derive(Debug)]
pub struct MaskedGeometry {
    pub z_index: f32,
    pub clip_rect: Aabb2<f32>,
    pub geometry: Geometry,
}

#[derive(ugli::Vertex, Debug, Clone, Copy, PartialEq)]
pub struct GeometryTriangleVertex {
    // /// Vertex z-index
    pub a_z: f32,
    /// Vertex position
    pub a_pos: vec2<f32>,
    /// Vertex color
    pub a_color: Color,
    /// Texture coordinates (when using a texture)
    pub a_vt: vec2<f32>,
}

#[derive(Debug)]
pub struct GeometryTexture {
    pub texture: PixelTexture,
    pub triangles: Vec<GeometryTriangleVertex>,
}

#[derive(Debug)]
pub struct GeometryText {
    pub z_index: f32,
    pub text: Name,
    pub position: vec2<f32>,
    pub options: TextRenderOptions,
}

impl Geometry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge(&mut self, other: Self) {
        self.triangles.extend(other.triangles);
        self.textures.extend(other.textures);
        self.text.extend(other.text);
        self.masked.extend(other.masked);
    }

    pub fn change_z_index(&mut self, delta: usize) {
        let delta = -(delta as f32) * 1e-5;
        for masked in &mut self.masked {
            masked.z_index += delta;
        }
        for v in &mut self.triangles {
            v.a_z += delta;
        }
        for texture in &mut self.textures {
            for v in &mut texture.triangles {
                v.a_z += delta;
            }
        }
        for text in &mut self.text {
            text.z_index += delta;
        }
    }

    fn triangles(triangles: Vec<GeometryTriangleVertex>) -> Self {
        Self {
            triangles,
            ..default()
        }
    }

    fn texture(texture: GeometryTexture) -> Self {
        Self {
            textures: vec![texture],
            ..default()
        }
    }

    fn text(text: GeometryText) -> Self {
        Self {
            text: vec![text],
            ..default()
        }
    }
}

impl GeometryContext {
    pub fn new(assets: Rc<Assets>) -> Self {
        Self {
            assets,
            framebuffer_size: vec2(1, 1),
            pixel_scale: 1.0,
            z_index: 0.into(),
        }
    }

    pub fn update(&mut self, framebuffer_size: vec2<usize>) {
        self.framebuffer_size = framebuffer_size;
        self.pixel_scale = crate::render::ui::pixel_scale(self.framebuffer_size);
        *self.z_index.get_mut() = 0;
    }

    fn next_z_index(&self) -> f32 {
        let mut index = self.z_index.borrow_mut();
        let current = *index;
        *index += 1;
        (current as f32) * 1e-5
    }

    #[must_use]
    pub fn masked(&self, clip_rect: Aabb2<f32>, geometry: Geometry) -> Geometry {
        Geometry {
            masked: vec![MaskedGeometry {
                z_index: self.next_z_index(),
                clip_rect,
                geometry,
            }],
            ..default()
        }
    }

    #[must_use]
    pub fn text(
        &self,
        text: Arc<str>,
        position: vec2<f32>,
        options: TextRenderOptions,
    ) -> Geometry {
        Geometry::text(GeometryText {
            z_index: self.next_z_index(),
            text,
            position,
            options,
        })
    }

    #[must_use]
    pub fn nine_slice(&self, pos: Aabb2<f32>, color: Color, texture: &PixelTexture) -> Geometry {
        let z_index = self.next_z_index();
        let texture = texture.clone();
        let whole = Aabb2::ZERO.extend_positive(vec2::splat(1.0));

        // TODO: configurable
        let mid = Aabb2 {
            min: vec2(0.3, 0.3),
            max: vec2(0.7, 0.7),
        };

        let size = mid.min * texture.size().as_f32() * self.pixel_scale;
        let size = vec2(size.x.min(pos.width()), size.y.min(pos.height()));

        let tl = Aabb2::from_corners(mid.top_left(), whole.top_left());
        let tm = Aabb2::from_corners(mid.top_left(), vec2(mid.max.x, whole.max.y));
        let tr = Aabb2::from_corners(mid.top_right(), whole.top_right());
        let rm = Aabb2::from_corners(mid.top_right(), vec2(whole.max.x, mid.min.y));
        let br = Aabb2::from_corners(mid.bottom_right(), whole.bottom_right());
        let bm = Aabb2::from_corners(mid.bottom_right(), vec2(mid.min.x, whole.min.y));
        let bl = Aabb2::from_corners(mid.bottom_left(), whole.bottom_left());
        let lm = Aabb2::from_corners(mid.bottom_left(), vec2(whole.min.x, mid.max.y));

        let triangles: Vec<GeometryTriangleVertex> = [tl, tm, tr, rm, br, bm, bl, lm, mid]
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
                    GeometryTriangleVertex {
                        a_z: z_index,
                        a_pos,
                        a_color: color,
                        a_vt,
                    }
                });
                [a, b, c, a, c, d]
            })
            .collect();

        Geometry::texture(GeometryTexture { texture, triangles })
    }

    #[must_use]
    pub fn quad(&self, position: Aabb2<f32>, color: Color) -> Geometry {
        let z_index = self.next_z_index();

        let [a, b, c, d] = position.corners();
        let a = (a, vec2(0.0, 0.0));
        let b = (b, vec2(1.0, 0.0));
        let c = (c, vec2(1.0, 1.0));
        let d = (d, vec2(0.0, 1.0));
        let triangles = [a, b, c, a, c, d]
            .into_iter()
            .map(|(a_pos, a_vt)| GeometryTriangleVertex {
                a_z: z_index,
                a_pos,
                a_color: color,
                a_vt,
            })
            .collect();

        Geometry::triangles(triangles)
    }

    #[must_use]
    pub fn quad_fill(&self, position: Aabb2<f32>, color: Color) -> Geometry {
        let size = position.size();
        let size = size.x.min(size.y);
        let texture = if size < 48.0 * self.pixel_scale {
            &self.assets.sprites.fill_thin
        } else {
            &self.assets.sprites.fill
        };
        self.nine_slice(position, color, texture)
    }

    #[must_use]
    pub fn quad_outline(&self, position: Aabb2<f32>, width: f32, color: Color) -> Geometry {
        let (texture, real_width) = if width < 2.0 * self.pixel_scale {
            (&self.assets.sprites.border_thinner, 1.0 * self.pixel_scale)
        } else if width < 4.0 * self.pixel_scale {
            (&self.assets.sprites.border_thin, 2.0 * self.pixel_scale)
        } else {
            (&self.assets.sprites.border, 4.0 * self.pixel_scale)
        };
        self.nine_slice(position.extend_uniform(real_width - width), color, texture)
    }

    #[must_use]
    pub fn texture(
        &self,
        position: Aabb2<f32>,
        transform: mat3<f32>,
        color: Color,
        texture: &PixelTexture,
    ) -> Geometry {
        let z_index = self.next_z_index();
        let texture = texture.clone();

        let [a, b, c, d] = position.corners();
        let a = (a, vec2(0.0, 0.0));
        let b = (b, vec2(1.0, 0.0));
        let c = (c, vec2(1.0, 1.0));
        let d = (d, vec2(0.0, 1.0));
        let triangles = [a, b, c, a, c, d]
            .into_iter()
            .map(|(a_pos, a_vt)| GeometryTriangleVertex {
                a_z: z_index,
                a_pos: (transform * a_pos.extend(1.0)).into_2d(),
                a_color: color,
                a_vt,
            })
            .collect();

        Geometry::texture(GeometryTexture { texture, triangles })
    }

    /// Pixel perfect texture
    #[must_use]
    pub fn texture_pp(
        &self,
        center: vec2<f32>,
        color: Color,
        scale: f32,
        texture: &PixelTexture,
    ) -> Geometry {
        let size = texture.size() * (self.pixel_scale * scale).round() as usize;
        let position = geng_utils::pixel::pixel_perfect_aabb(
            center,
            vec2(0.5, 0.5),
            size,
            &geng::PixelPerfectCamera,
            self.framebuffer_size.as_f32(),
        );

        self.texture(position, mat3::identity(), color, texture)
    }
}
