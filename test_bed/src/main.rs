use minifb::{Key, Window, WindowOptions};
use flowi_core::layout::Axis;
use flowi_core::Ui;
use flowi_sw_renderer::SwRenderer;
use flowi_core::primitives::{Primitive, Uv, Color32};
use flowi_core::box_area::Rect;

const WIDTH: usize = 1280;
const HEIGHT: usize = 768;

fn render_circle(buffer: &mut [u32], pos: (f32, f32), offset: f32, radius: f32) { 
    let height = radius.floor() as usize;
    let width = height * 2; 

    // Loop through each pixel
    for y in 0..height {
        for x in 0..width {
            // Calculate pixel position in buffer
            let idx = (y * WIDTH) + x;

            // Calculate the distance from the pixel center to the circle's center
            let px = (x as f32 + 0.5) + offset; // Pixel center X (floating point)
            let py = y as f32 + 0.5; // Pixel center Y (floating point)
            let distance = ((px - pos.0).powi(2) + (py - pos.1).powi(2)).sqrt();

            // Calculate how close the pixel is to the boundary of the circle
            let dist_to_edge = (distance - radius).abs();

            // Use the distance to determine the alpha value (anti-aliasing)
            let alpha = if distance < radius {
                255 // Fully inside the circle
            } else if dist_to_edge < 1.0 {
                // Smooth edge using a simple linear interpolation based on distance
                ((1.0 - dist_to_edge) * 255.0) as u8
            } else {
                0 // Outside the circle
            };

            let alpha = alpha as u32;
            let color = (alpha << 16) | (alpha << 8) | alpha;
            buffer[idx] = color; // Red
        }
    }
}

fn main() {
    Ui::create();

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut sw_renderer = SwRenderer::new(WIDTH, HEIGHT, 128, 128);

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
    let mut offset = 0.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in buffer.iter_mut() {
            *i = 0; // write something more funny here!
        }

        let input = Ui::input();

        // Update the input state for the frame
        window.get_mouse_pos(minifb::MouseMode::Clamp).map(|mouse| {
            input.set_mouse_position(mouse.0 as f32, mouse.1 as f32);
        });

        /*
        Ui::begin(0.0, WIDTH, HEIGHT);
        
        Ui::create_box_with_string("Hello, World!");

        if Ui::button("MyButtont").hovering() {
            println!("Button hovered!");
        } else {
            println!("Button not hovered!");
        }

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

        render_circle(&mut buffer, (0 as f32, 0 as f32), offset, 160.0);

        offset -= 0.1;
        */

        let c0 = Color32::new(0x0, 0, 0, 0xff); 
        let c1 = Color32::new(0x0, 0, 0, 0xff); 
        let c2 = Color32::new(0x0, 0xff, 0, 0xff); 
        let c3 = Color32::new(0xff, 0, 0, 0xff); 

        let primitive = Primitive {
            rect: Rect::new(10.0, 10.0, 100.0, 100.0),
            uvs: [Uv::new(0.0, 0.0); 4],
            colors: [c0, c1, c2, c3],
            _corners: [0.0; 4],
            _texture_handle: 0,
        };

        let tile = sw_renderer.tiles[0];
        sw_renderer.quad_ref_renderer(&tile, &primitive);
        
        //sw_renderer.test_render_in_tile();
        sw_renderer.copy_tile_buffer_to_output(buffer.as_mut_ptr());

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
