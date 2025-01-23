//use crate::{generated::image::Image, generated::renderer::Texture, ApplicationSettings};
use crate::ApplicationSettings;
use clay_layout::render_commands::RenderCommand;
use raw_window_handle::RawWindowHandle;
use std::collections::HashMap;
use crate::font::FontHandle;

pub enum SoftwareRenderFormat {
    RGBA16,
    RGB8,
}

pub enum ImageFormat {
    RGBA16,
    I16,
}

pub struct Image {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
}

pub struct SoftwareRenderData<'a> {
    pub buffer: &'a [u8],
    pub width: u32,
    pub height: u32,
}

pub trait FlowiRenderer {
    fn new(settings: &ApplicationSettings, window: Option<&RawWindowHandle>) -> Self
    where
        Self: Sized;

    /// Sets the text buffer with the given parameters.
    ///
    /// This is a generated text matching the text, font and size as input. It's expected that the
    /// backend will save away this data as it sees fit and use it later when rendering text.
    /// 
    /// # Parameters
    /// - `_text`: The text to be set.
    /// - `_font_size`: The size of the font.
    /// - `_font_id`: The handle to the font.
    /// - `_text_buffer`: The buffer containing the text data.
    fn set_text_buffer(&mut self, _text: &str, _font_size: i16, _font_id: FontHandle, _image: &Image) {}

    /// If the renderer returns this it's expected that it has filled this buffer.
    fn software_renderer_info<'a>(&'a self) -> Option<SoftwareRenderData<'a>> {
        None
    }

    fn render(&mut self, commands: &[RenderCommand]);
}

pub struct DummyRenderer {}

impl FlowiRenderer for DummyRenderer {
    fn new(_settings: &ApplicationSettings, _window: Option<&RawWindowHandle>) -> Self {
        Self {}
    }

    fn render(&mut self, _commands: &[RenderCommand]) {}

    /*
    fn get_texture(&mut self, _image: Image) -> Texture {
        Texture { handle: 0 }
    }
    */
}

#[allow(dead_code)]
pub(crate) struct RendererState {
    pub(crate) _image_texture_map: HashMap<u64, u64>,
}

#[allow(dead_code)]
impl RendererState {
    pub fn new() -> Self {
        Self {
            _image_texture_map: HashMap::new(),
        }
    }
}
