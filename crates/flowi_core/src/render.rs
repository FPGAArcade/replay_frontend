//use crate::{generated::image::Image, generated::renderer::Texture, ApplicationSettings};
use crate::ApplicationSettings;
use raw_window_handle::RawWindowHandle;
use std::collections::HashMap;
use clay_layout::render_commands::RenderCommand;

/// Used to tell the backend what kinda of rendering is going on (i.e hardware or not)
/// In the case of a software renderer the backend will request an output bufffer as well
/// after the rendering is complete
pub enum RenderType {
    Hardware,
    Software,
}

pub trait FlowiRenderer {
    fn new(settings: &ApplicationSettings, window: Option<&RawWindowHandle>) -> Self
    where
        Self: Sized;

    fn rendertype(&self) -> RenderType {
        RenderType::Hardware
    }

    fn render<'a>(&mut self, commands: impl Iterator<Item = RenderCommand<'a>>);
    //fn get_texture(&mut self, image: Image) -> Texture;
}

pub struct DummyRenderer {}

impl FlowiRenderer for DummyRenderer {
    fn new(_settings: &ApplicationSettings, _window: Option<&RawWindowHandle>) -> Self {
        Self {}
    }

    fn render<'a>(&mut self, _commands: impl Iterator<Item = RenderCommand<'a>>) {}

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
