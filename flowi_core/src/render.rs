use crate::{generated::image::Image, generated::renderer::Texture, ApplicationSettings};
use raw_window_handle::{HasRawWindowHandle, HasRawDisplayHandle};
use std::collections::HashMap;

pub trait Window: HasRawDisplayHandle + HasRawWindowHandle {
    fn new(settings: &ApplicationSettings) -> Self where Self: Sized;
    fn update(&mut self);
    fn should_close(&mut self) -> bool;
    fn is_focused(&self) -> bool;
}

pub struct WindowWrapper {
    pub w: Box<dyn Window>,
}

unsafe impl HasRawWindowHandle for WindowWrapper {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        self.w.raw_window_handle()
    }
} 

unsafe impl HasRawDisplayHandle for WindowWrapper {
    fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
        self.w.raw_display_handle()
    }
}

impl WindowWrapper {
    pub fn new(window: Box<dyn Window>) -> Self {
        Self {
            w: window,
        }
    }
}

pub trait FlowiRenderer {
    fn new(settings: &ApplicationSettings, window: &WindowWrapper) -> Self
    where
        Self: Sized;
    fn render(&mut self);
    fn get_texture(&mut self, image: Image) -> Texture;
}

pub struct DummyRenderer {}

impl FlowiRenderer for DummyRenderer {
    fn new(_settings: &ApplicationSettings, _window: &WindowWrapper) -> Self {
        Self {}
    }

    fn render(&mut self) {}

    fn get_texture(&mut self, _image: Image) -> Texture {
        Texture { handle: 0 }
    }
}

pub(crate) struct RendererState {
    pub(crate) _image_texture_map: HashMap<u64, u64>,
}

impl RendererState {
    pub fn new() -> Self {
        Self {
            _image_texture_map: HashMap::new(),
        }
    }
}

struct WrapState<'a> {
    s: &'a mut crate::InternalState,
}

#[no_mangle]
pub extern "C" fn fl_renderer_get_texture_impl(
    data: *mut core::ffi::c_void,
    image: Image,
) -> Texture {
    let state = &mut unsafe { &mut *(data as *mut WrapState) }.s;

    // TODO: How to handle reload
    /*
    if let Some(texture_id) = state.renderer_state.image_texture_map.get(&image.handle) {
        return Texture { handle: *texture_id };
    }
    */

    // Check if image has been loaded yet
    /*
    if !state.io_handler.is_loaded(image.handle) {
        return Texture { handle: 0 };
    }
    */

    state.renderer.get_texture(image)
}
