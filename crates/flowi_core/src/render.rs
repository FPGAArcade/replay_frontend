//use crate::{generated::image::Image, generated::renderer::Texture, ApplicationSettings};
use crate::ApplicationSettings;
use raw_window_handle::RawWindowHandle;
use std::collections::HashMap;

pub trait FlowiRenderer {
    fn new(settings: &ApplicationSettings, window: Option<&RawWindowHandle>) -> Self
    where
        Self: Sized;
    fn render(&mut self);
    //fn get_texture(&mut self, image: Image) -> Texture;
}

pub struct DummyRenderer {}

impl FlowiRenderer for DummyRenderer {
    fn new(_settings: &ApplicationSettings, _window: Option<&RawWindowHandle>) -> Self {
        Self {}
    }

    fn render(&mut self) {}

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
