use khronos_egl as egl;

fn init_egl() -> Result<(), egl::Error> {
    let display = match egl::get_display(egl::DEFAULT_DISPLAY) {
        None => return Err(egl::Error::NotInitialized),
        Some(d) => d,
    };

    let egl_ver = egl::initialize(display)?;

    println!("Found EGL ver {}.{}", egl_ver.0, egl_ver.1);

    Ok(())
}

fn main() {
    init_egl().unwrap();
}
