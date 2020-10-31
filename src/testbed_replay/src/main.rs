use khronos_egl as egl;

#[derive(Debug)]
struct EglContext {
    display: egl::Display,
    context: egl::Context,
    surface: egl::Surface,
}

impl EglContext {
    pub fn new() -> Result<EglContext, egl::Error> {
        let display = egl::get_display(egl::DEFAULT_DISPLAY)?;
        let egl_ver = egl::initialize(display)?;

        println!("Found EGL ver {}.{}", egl_ver.0, egl_ver.1);

        #[rustfmt::skip]
        let config_attributes: &[egl::Int] = &[
            egl::SAMPLES,             4,
            egl::ALPHA_SIZE,          0,
            egl::RED_SIZE,            8,
            egl::GREEN_SIZE,          8,
            egl::BLUE_SIZE,           8,
            egl::BUFFER_SIZE,         32,

            egl::STENCIL_SIZE,        0,
            egl::RENDERABLE_TYPE,     egl::OPENGL_ES2_BIT,
            egl::SURFACE_TYPE,        egl::WINDOW_BIT,
            egl::DEPTH_SIZE,          16,
            egl::NONE
        ];

        #[rustfmt::skip]
        let context_attributes: &[egl::Int] = &[
            egl::CONTEXT_CLIENT_VERSION, 2,
            egl::NONE, egl::NONE,
            egl::NONE
        ];

        // do 1 config for now as we have a fixed platform
        let mut configs = Vec::with_capacity(1);
        egl::choose_config(display, config_attributes, &mut configs)?;

        if configs.len() == 0 {
            println!("Unable to find any EGL configs");
            return Err(egl::Error::BadConfig);
        }

        let surface =
            unsafe { egl::create_window_surface(display, configs[0], std::ptr::null_mut(), None)? };

        egl::bind_api(egl::OPENGL_ES_API)?;

        let context = egl::create_context(display, configs[0], None, context_attributes)?;

        Ok(EglContext {
            display,
            context,
            surface,
        })
    }

    fn close(&self) -> Result<(), egl::Error> {
        egl::make_current(self.display, None, None, Some(self.context))?;
        egl::destroy_context(self.display, self.context)?;
        egl::destroy_surface(self.display, self.surface)?;
        egl::terminate(self.display)
    }
}

impl Drop for EglContext {
    fn drop(&mut self) {
        match self.close() {
            Ok(()) => (),
            Err(e) => println!("Error when closing {}", e),
        }
    }
}

fn main() {
    let context = EglContext::new().unwrap();




    dbg!(context);
}
