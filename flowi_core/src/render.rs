use std::collections::HashMap;
use raw_window_handle::RawWindowHandle;
use crate::{generated::image::Image, ApplicationSettings, generated::renderer::Texture};

pub trait FlowiRenderer {
    fn new(settings: &ApplicationSettings, window: Option<&RawWindowHandle>) -> Self 
        where Self: Sized;
    fn render(&mut self);
    fn get_texture(&mut self, image: Image) -> Texture;
}

pub struct DummyRenderer {}

impl FlowiRenderer for DummyRenderer {
    fn new(_settings: &ApplicationSettings, _window: Option<&RawWindowHandle>) -> Self {
        Self {}
    }

    fn render(&mut self) { }

    fn get_texture(&mut self, _image: Image) -> Texture {
        Texture { handle: 0 }
    }
}

pub(crate) struct RendererState {
    pub(crate) image_texture_map: HashMap<u64, u64>, 
}  

impl RendererState {
    pub fn new() -> Self {
        Self {
            image_texture_map: HashMap::new(),
        }
    }
}

struct WrapState<'a> {
    s: &'a mut crate::InternalState,
}

#[no_mangle]
pub extern "C" fn fl_renderer_get_texture_impl(data: *mut core::ffi::c_void, image: Image) -> Texture {
    let state = &mut unsafe { &mut *(data as *mut WrapState) }.s;

    // TODO: How to handle reload
    if let Some(texture_id) = state.renderer_state.image_texture_map.get(&image.handle) {
        return Texture { handle: *texture_id };
    }

    // Check if image has been loaded yet
    if !state.io_handler.is_loaded(image.handle) {
        return Texture { handle: 0 };
    }

    state.renderer.get_texture(image)
}


