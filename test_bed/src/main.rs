use minifb::{Key, Window, WindowOptions};
use flowi_core::Flowi;
use flowi_sw_renderer::SwRenderer;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut flowi_core = Flowi::new();
    let mut sw_renderer = SwRenderer::new();

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in buffer.iter_mut() {
            *i = 0; // write something more funny here!
        }

        flowi_core.begin(0.0, WIDTH, HEIGHT);

        flowi_core.create_box_with_string("Hello, World!");
        flowi_core.create_box_with_string("Hello, World! 2");

        flowi_core.end();

        let primitives = flowi_core.primitives();

        sw_renderer.render(&mut buffer, WIDTH, HEIGHT, primitives);

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
