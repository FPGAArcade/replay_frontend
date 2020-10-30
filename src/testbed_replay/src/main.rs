use khronos_egl as egl;

fn init_egl() -> Result<(), egl::Error> {
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
    let _context_attributes: &[egl::Int] = &[
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

    Ok(())
}

fn main() {
    init_egl().unwrap();
}
