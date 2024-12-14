mod font;

pub use self::font::Font;

use ctl_client::core::types::MusicInfo;
use geng::prelude::*;

use crate::prelude::Modifier;

#[derive(geng::asset::Load)]
pub struct MusicAssets {
    #[load(postprocess = "looping", ext = "mp3")]
    pub music: geng::Sound,
    pub meta: MusicInfo,
}

#[derive(geng::asset::Load)]
pub struct LoadingAssets {
    #[load(path = "sprites/title.png", options(filter = "ugli::Filter::Nearest"))]
    pub title: ugli::Texture,
    #[load(path = "fonts/pixel.ttf")]
    pub font: Font,
}

#[derive(geng::asset::Load)]
pub struct Assets {
    pub sprites: Sprites,
    pub dither: DitherAssets,
    pub shaders: Shaders,
    pub fonts: Fonts,
}

#[derive(geng::asset::Load)]
pub struct Fonts {
    pub default: Rc<Font>,
    pub pixel: Rc<Font>,
}

#[derive(geng::asset::Load)]
pub struct Sprites {
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub title: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub linear_gradient: ugli::Texture,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub radial_gradient: ugli::Texture,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub button_next: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub button_prev: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub button_close: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub border: ugli::Texture,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub border_thin: ugli::Texture,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub border_thinner: ugli::Texture,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub fill: ugli::Texture,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub fill_thin: ugli::Texture,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub circle: ugli::Texture,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub help: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub reset: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub head: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub edit: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub download: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub goto: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub trash: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub settings: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub discor: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub star: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub local: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub dotdotdot: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub arrow_up: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub arrow_down: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub arrow_left: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub arrow_right: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub pause: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub confirm: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub discard: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub loading: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub mod_nofail: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub mod_sudden: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub mod_hidden: Rc<ugli::Texture>,
}

#[derive(geng::asset::Load)]
pub struct Shaders {
    pub solid: ugli::Program,
    pub light: ugli::Program,
    pub masked: ugli::Program,
    pub texture: ugli::Program,
}

#[derive(geng::asset::Load)]
pub struct DitherAssets {
    /// Dither postprocess shader
    pub dither_shader: ugli::Program,
    #[load(postprocess = "dither_pattern")]
    pub dither1: ugli::Texture,
    #[load(postprocess = "dither_pattern")]
    pub dither2: ugli::Texture,
    #[load(postprocess = "dither_pattern")]
    pub dither3: ugli::Texture,
}

fn dither_pattern(texture: &mut ugli::Texture) {
    texture.set_wrap_mode(ugli::WrapMode::Repeat);
    texture.set_filter(ugli::Filter::Nearest);
}

fn looping(sfx: &mut geng::Sound) {
    sfx.looped = true;
}

impl Assets {
    pub async fn load(manager: &geng::asset::Manager) -> anyhow::Result<Self> {
        geng::asset::Load::load(manager, &run_dir().join("assets"), &())
            .await
            .context("failed to load assets")
    }

    pub fn get_modifier(&self, modifier: Modifier) -> &Rc<ugli::Texture> {
        match modifier {
            Modifier::NoFail => &self.sprites.mod_nofail,
            Modifier::Sudden => &self.sprites.mod_sudden,
            Modifier::Hidden => &self.sprites.mod_hidden,
        }
    }
}
