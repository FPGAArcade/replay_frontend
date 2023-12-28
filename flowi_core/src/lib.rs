pub mod generated;
pub mod image_api;
pub mod imgui;
pub use generated::*;
pub use manual::Color;
pub use manual::Result;
mod io_handler;
mod manual;
pub mod render;
mod tests;

use crate::render::RendererState;
pub use crate::render::{DummyRenderer, FlowiRenderer};
use core::ffi::c_void;
use fileorama::Fileorama;
use io_handler::IoHandler;

#[repr(C)]
pub struct InternalState {
    pub renderer: Box<dyn FlowiRenderer>,
    pub(crate) renderer_state: RendererState,
    pub(crate) vfs: Fileorama,
    pub(crate) io_handler: IoHandler,
}

#[repr(C)]
pub struct Instance {
    c_data: *const c_void,
    pub state: Box<InternalState>,
}

extern "C" {
    fn c_create(settings: *const ApplicationSettings, rust_state: *const c_void) -> *const c_void;
    fn c_pre_update(data: *const c_void);
    fn c_post_update(data: *const c_void);
}

impl Instance {
    pub fn new(settings: &ApplicationSettings) -> Self {
        let vfs = Fileorama::new(2);
        let io_handler = IoHandler::new(&vfs);
        let renderer = Box::new(DummyRenderer::new(settings, None));

        crate::image_api::install_image_loader(&vfs);

        let state = Box::new(InternalState {
            renderer,
            renderer_state: RendererState::new(),
            vfs,
            io_handler,
        });

        let ptr = Box::into_raw(state);
        let c_data = unsafe { c_create(settings, ptr as *const _) };
        let state = unsafe { Box::from_raw(ptr) };

        Self { c_data, state }
    }

    pub fn pre_update(&self) {
        unsafe { c_pre_update(self.c_data) }
    }

    pub fn update(&mut self) {
        self.state.io_handler.update();
    }

    pub fn post_update(&self) {
        unsafe { c_post_update(self.c_data) }
    }
}

pub use crate::application_settings::ApplicationSettings;
