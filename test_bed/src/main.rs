use minifb::{Key, Window, WindowOptions, Scale, ScaleMode};
use clay_layout::{
    elements::{rectangle::Rectangle, CornerRadius},
    fixed,
    render_commands::*,
    math::Dimensions,
    layout::Layout,
    Clay,
};

use ui_raster::{ColorSpace, Renderer, RenderPrimitive};
use ui_raster::simd::{i32x4, f32x4};

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT * 2];

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: Scale::X1,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let clay = Clay::new(Dimensions::new(WIDTH as f32, HEIGHT as f32));
    let mut renderer = Renderer::new(ColorSpace::Linear, (WIDTH, HEIGHT), (10, 12)); 

    // Limit to max ~60 fps update rate
    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in buffer.iter_mut() {
            *i = 0; // write something more funny here!
        }

        // Begin the layout
        clay.begin();

        // Adds a red rectangle with a corner radius of 5.
        // The Layout makes the rectangle have a width and height of 50.
        clay.with(
            [
                Layout::new().width(fixed!(50.)).height(fixed!(50.)).end(),
                Rectangle::new()
                    .color((0xFF, 0x00, 0x00).into())
                    .corner_radius(CornerRadius::All(5.))
                    .end("Red Rectangle".into()),
            ],
            |_| {},
        );

        // Return the list of render commands of your layout
        let render_commands = clay.end();

        renderer.begin_frame();

        for command in render_commands {
            let aabb = i32x4::new(
                command.bounding_box.x as i32,
                command.bounding_box.y as i32,
                (command.bounding_box.x + command.bounding_box.width) as i32,
                (command.bounding_box.y + command.bounding_box.height) as i32,
            );

            match &command.config {
                RenderCommandConfig::Rectangle(rectangle) => {
                    let color = renderer.get_color_from_floats_0_255(
                        rectangle.color.r, 
                        rectangle.color.g, 
                        rectangle.color.b, 
                        rectangle.color.a);

                    let corner_radius = match rectangle.corner_radius {
                        CornerRadius::All(radius) => {
                            f32x4::new(radius, radius, radius, radius)
                        },

                        CornerRadius::Individual { top_left, top_right, bottom_left, bottom_right } => {
                            f32x4::new(top_left, top_right, bottom_left, bottom_right)
                        },
                    };

                    let primitive = RenderPrimitive {
                        aabb,
                        color,
                        corner_radius,
                    };

                    renderer.add_primitive(primitive);
                }

                _ => {},
            }
        }

        renderer.flush_frame(&mut buffer);

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
