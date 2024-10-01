use minifb::{Key, Window, WindowOptions};
use flowi_core::layout::Axis;
use flowi_core::Ui;
use flowi_sw_renderer::SwRenderer;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

fn main() {
    Ui::create();

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
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

        Ui::begin(0.0, WIDTH, HEIGHT);

        Ui::create_box_with_string("Hello, World!");
        Ui::create_box_with_string("Hello, World! 2");
        Ui::create_box_with_string("Hello, World! 3");

        /*
        Ui::layout.child_layout_axis.push(Axis::Vertical);
        let b = Ui::create_box_with_string("Hello, World! 4");
        Ui::layout.owner.push(b);
        
        Ui::create_box_with_string("Hello, World! 5");
        Ui::create_box_with_string("Hello, World! 6");

        Ui::layout.child_layout_axis.pop();
        Ui::layout.owner.pop();
        */
        
        Ui::end();

        let primitives = Ui::primitives();

        sw_renderer.render(&mut buffer, WIDTH, HEIGHT, primitives);

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
