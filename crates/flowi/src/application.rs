//use flowi_core::imgui::{DrawCmd, DrawData, DrawVert, FontAtlas, ImDrawIdx};
//use crate::bgfx_renderer::BgfxRenderer;
//use crate::glfw_window::GlfwWindow;
use crate::sdl_window::Sdl2Window;
use crate::sw_renderer::SwRenderer;

use core::ptr::null_mut;
use core::{ffi::c_void, mem::transmute};
use flowi_core::ApplicationSettings;
use flowi_core::render::FlowiRenderer;
use flowi_core::Flowi;
//use flowi_core::Instance;
//use flowi_core::Result;
//use raw_window_handle::RawWindowHandle;

pub(crate) trait Window {
    fn new(settings: &ApplicationSettings) -> Self
    where
        Self: Sized;
    fn update(&mut self);
    fn should_close(&mut self) -> bool;
    //fn is_focused(&self) -> bool;
    //fn raw_window_handle(&self) -> RawWindowHandle;
}

#[repr(C)]
struct WrappedMainData {
    user_data: *const c_void,
    user_func: *const c_void,
}

#[repr(C)]
pub struct Application {
    pub(crate) window: Box<dyn Window>,
    pub(crate) core: Box<Flowi>,
    user: WrappedMainData,
    pub(crate) settings: ApplicationSettings,
}

#[allow(clippy::transmute_ptr_to_ref)]
unsafe extern "C" fn user_trampoline_ud<T>(wd: &WrappedMainData) {
    let f: &&(dyn Fn(&mut T) + 'static) = transmute(wd.user_func);
    let data = wd.user_data as *mut T;
    f(&mut *data);
}

#[allow(clippy::transmute_ptr_to_ref)]
unsafe extern "C" fn mainloop_app<T>(user_data: *mut c_void) {
    let state: &mut Application = transmute(user_data);

    while !state.window.should_close() {
        //state.core.pre_update();
        state.window.update();
        //state.core.update();

        user_trampoline_ud::<T>(&state.user);

        //state.core.post_update();
        //state.core.state.renderer.render();

        // TODO: This is a hack to not use 100% CPU
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

impl Application {
    pub fn new(settings: &ApplicationSettings) -> Box<Self> {
        let core = Flowi::new();
        let window = Box::new(Sdl2Window::new(settings));

        Box::new(Self {
            window,
            core,
            settings: *settings,
            user: WrappedMainData {
                user_data: null_mut(),
                user_func: null_mut(),
            },
        })
    }

    #[allow(clippy::type_complexity)]
    pub fn run<'a, F, T>(&mut self, data: Box<T>, func: F) -> bool
    where
        F: Fn(&mut T) + 'a,
    {
        let renderer = Box::new(SwRenderer::new(&self.settings, None));
        //self.core.state.renderer = renderer;

        // Having the data on the stack is safe as the mainloop only exits after the application is about to end
        let f: Box<Box<dyn Fn(&mut T) + 'a>> = Box::new(Box::new(func));
        let func = Box::into_raw(f) as *const _;

        self.user.user_data = Box::into_raw(data) as *const _;
        self.user.user_func = func;

        /*
        * TODO: If web target we should use emscripten_set_main_loop_arg
        unsafe {
            emscripten_set_main_loop_arg(
                mainloop_trampoline_ud::<T>,
                Box::into_raw(wrapped_data) as *const _,
                0,
                1,
            );
        }
        */
        unsafe { mainloop_app::<T>(self as *mut _ as *mut c_void) };

        true
    }
}
