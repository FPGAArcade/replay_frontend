//use crate::image::Image;
//use flowi_core::imgui::FontAtlas;
use flowi_core::render::FlowiRenderer;
//use flowi_core::renderer::Texture as CoreTexture;
use flowi_core::ApplicationSettings;
use raw_window_handle::RawWindowHandle;

pub struct SwRenderer {
    _dummy: u32,
}

impl FlowiRenderer for SwRenderer {
    fn new(_settings: &ApplicationSettings, _window: Option<&RawWindowHandle>) -> Self {
        //let _font_atlas = FontAtlas::build_r8_texture();
        Self { _dummy: 0 }
    }

    fn render(&mut self) {}

    /*
    fn get_texture(&mut self, _image: Image) -> CoreTexture {
        CoreTexture { handle: 0 }
    }
    */
}

impl SwRenderer {}
