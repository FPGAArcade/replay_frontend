
pub mod generated;
pub mod image_api;
pub mod imgui;
pub use manual::Result;
pub use generated::*;
pub mod render;
mod manual;
mod tests;

use core::ffi::c_void;
use fileorama::Fileorama;
use image_api::ImageHandler;
pub use crate::render::{FlowiRenderer, DummyRenderer};
use crate::render::RendererState;

#[repr(C)]
pub struct InternalState {
    pub renderer: Box<dyn FlowiRenderer>,
    pub(crate) renderer_state: RendererState,
    pub(crate) vfs: Fileorama,
    pub(crate) image_handler: ImageHandler,
}

#[repr(C)]
pub struct Instance {
    c_data: *const c_void,
    pub state: Box<InternalState>,
}

extern "C" {
    fn c_create(
        settings: *const ApplicationSettings,
        rust_state: *const c_void,
    ) -> *const c_void;
    fn c_pre_update(data: *const c_void);
    fn c_post_update(data: *const c_void);
}

impl Instance {
    pub fn new(settings: &ApplicationSettings) -> Self {
        let vfs = Fileorama::new(2);
        let image_handler = ImageHandler::new(&vfs);
        let renderer = Box::new(DummyRenderer::new(settings, None));

        let state = Box::new(InternalState {
            renderer,
            renderer_state: RendererState::new(),
            vfs,
            image_handler,
        });

        let ptr = Box::into_raw(state);
        let c_data = unsafe { c_create(settings, ptr as *const _) };
        let state = unsafe { Box::from_raw(ptr) };

        Self { c_data, state }
    }

    pub fn pre_update(&self) {
        unsafe { c_pre_update(self.c_data) }
    }

    pub fn post_update(&self) {
        unsafe { c_post_update(self.c_data) }
    }
}

pub use crate::application_settings::ApplicationSettings;



