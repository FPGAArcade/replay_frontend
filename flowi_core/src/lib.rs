use core::ffi::c_void;
//use fileorama::Fileorama;
use image_api::ImageHandler;

pub mod generated;
pub mod image_api;
pub use generated::*;
mod manual;

mod tests;

pub mod imgui;
pub use manual::Result;

#[repr(C)]
pub(crate) struct InternalState {
    //pub(crate) vfs: Fileorama,
    pub(crate) image_handler: ImageHandler,
}

#[repr(C)]
pub struct Instance {
    c_data: *const c_void,
    state: Box<InternalState>,
}

extern "C" {
    fn c_create(
        settings: *const ApplicationSettings,
        rust_state: *const InternalState,
    ) -> *const c_void;
    fn c_pre_update(data: *const c_void);
    fn c_post_update(data: *const c_void);
    fn fl_set_display_size_impl(data: *const c_void, width: u32, height: u32);
    fn fl_set_display_buffer_scale_impl(data: *const c_void, width: f32, height: f32);
    fn fl_set_delta_time_impl(data: *const c_void, delta_time: f32);
}

impl Instance {
    pub fn new(settings: &ApplicationSettings) -> Self {
        //let vfs = Fileorama::new(vfs_thread_count);
        //let image_handler = ImageHandler::new(&vfs);
        let image_handler = ImageHandler::new();

        let state = Box::new(InternalState {
            //vfs,
            image_handler,
        });

        let ptr = Box::into_raw(state);
        let c_data = unsafe { c_create(settings, ptr) };
        let state = unsafe { Box::from_raw(ptr) };

        Self { c_data, state }
    }

    pub fn set_display_size(&self, width: u32, height: u32) {
        unsafe { fl_set_display_size_impl(self.c_data, width, height) }
    }

    pub fn set_display_buffer_scale(&self, width: f32, height: f32) {
        unsafe { fl_set_display_buffer_scale_impl(self.c_data, width, height) }
    }

    pub fn set_delta_time(&self, delta_time: f32) { 
        unsafe { fl_set_delta_time_impl(self.c_data, delta_time) }
    }

    pub fn pre_update(&self) {
        unsafe { c_pre_update(self.c_data) }
    }

    pub fn post_update(&self) {
        unsafe { c_post_update(self.c_data) }
    }
}

pub use crate::application_settings::ApplicationSettings;
