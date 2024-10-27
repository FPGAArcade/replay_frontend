use minifb::{Key, Window, WindowOptions, Scale, ScaleMode};
use flowi_core::layout::Axis;
use flowi_core::Ui;
use flowi_sw_renderer::{SwRenderer, Color16};
use flowi_core::primitives::{Primitive, Uv, Color32, Vec2};
use flowi_core::box_area::Rect;

use std::arch::x86_64::*;

const WIDTH: usize = 1280/2;
const HEIGHT: usize = 768/2;

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

#[derive(Copy, Clone)]
enum Corner {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

const CORNER_OFFSETS: [(f32, f32); 4] = [
    (1.0, 1.0),       // TopLeft: No shift
    (1.0, 0.0),       // TopRight: Shift right
    (0.0, 1.0),       // BottomLeft: Shift down
    (1.0, 1.0),       // BottomRight: Shift right and down
];


const CORNER_DIRECTIONS: [(f32, f32); 4] = [
    (1.0, 1.0),  // TopLeft: positive x and positive y
    (-1.0, 1.0), // TopRight: negative x and positive y
    (1.0, -1.0), // BottomLeft: positive x and negative y
    (-1.0, -1.0) // BottomRight: negative x and negative y
];

fn get_corner_direction(corner: Corner, pos: Vec2) -> (f32, f32) {
    // Lookup direction values (x_direction, y_direction) for the specified corner
    CORNER_DIRECTIONS[corner as usize]
}

/// Calculates the bounding box for rendering a quarter of a circle (a corner) based on its
/// position, radius, and which corner of the circle is being rendered.
///
/// # Parameters
/// 
/// * `pos` - The center position of the circle in the image, given as a `Vec2` (with `x` and `y` coordinates).
/// * `radius` - The radius of the circle. The bounding box will be calculated based on this radius.
/// * `corner` - The specific corner of the circle being rendered. This determines how the bounding
///              box is shifted relative to the circle's position. 
/// # Returns
/// 
/// A tuple `(min_x, max_x, min_y, max_y)` representing the bounding box for the given corner:
/// 
/// * `min_x` - The leftmost x-coordinate of the bounding box.
/// * `max_x` - The rightmost x-coordinate of the bounding box.
/// * `min_y` - The topmost y-coordinate of the bounding box.
/// * `max_y` - The bottommost y-coordinate of the bounding box.
///
fn calculate_bounding_box(pos: Vec2, radius: f32, corner: Corner) -> (i32, i32, i32, i32) {
    // Lookup the corner's x and y shift factors from the static table
    let (x_factor, y_factor) = CORNER_OFFSETS[corner as usize];

    // Precompute the shifted values
    let x_shifted = x_factor * radius;
    let y_shifted = y_factor * radius;

    // Calculate the bounding box using the precomputed shifts
    let min_x = (pos.x).floor() as i32;
    let max_x = (pos.x + x_shifted).ceil() as i32;
    let min_y = (pos.y).floor() as i32;
    let max_y = (pos.y + y_shifted).ceil() as i32;

    (min_x, max_x, min_y, max_y)
}

/*
#[target_feature(enable = "sse4.2")]
unsafe fn load_and_convert_color_to_float(color: &Color16) -> (__m128, __m128) {
    // Step 1: Load the 128-bit value (8 x 16-bit integers)
    let color_data = _mm_loadu_si128(color.as_ptr() as *const __m128i);

    // Step 2: Unpack the lower and upper 64 bits into 32-bit integers
    let lo_32 = _mm_unpacklo_epi16(color_data, _mm_setzero_si128()); // First 4 values (low part)
    let hi_32 = _mm_unpackhi_epi16(color_data, _mm_setzero_si128()); // Last 4 values (high part)

    // Step 3: Convert the unpacked 32-bit integers to 32-bit floats
    let lo_float = _mm_cvtepi32_ps(lo_32); // Convert low part to float
    let hi_float = _mm_cvtepi32_ps(hi_32); // Convert high part to float

    // Return two SSE registers with floating-point values
    (lo_float, hi_float)
}
*/

#[target_feature(enable = "sse4.2")]
unsafe fn pack_float_to_color16(lo_float: __m128, hi_float: __m128) -> __m128i {
    // Step 1: Convert floating-point values to 32-bit integers
    let lo_32 = _mm_cvtps_epi32(lo_float); // Convert lo_float to 32-bit integers
    let hi_32 = _mm_cvtps_epi32(hi_float); // Convert hi_float to 32-bit integers

    // Step 2: Pack 32-bit integers into 16-bit integers
    let packed_16 = _mm_packs_epi32(lo_32, hi_32); // Packs lower 16 bits from lo_32 and hi_32

    packed_16
}

#[target_feature(enable = "sse4.2")]
#[inline(never)]
unsafe fn render_corner_with_border(
    output: *mut Color16,
    buffer_width: usize,
    buffer_height: usize,
    border_radius: f32, 
    inner_radius: f32, 
    pos: Vec2,
    corner: Corner) 
{
    let (min_x, max_x, min_y, max_y) = calculate_bounding_box(pos, border_radius, corner);

    let min_x = min_x.max(0) as usize;
    let max_x = max_x.min(buffer_width as _) as usize;
    let min_y = min_y.max(0) as usize;
    let max_y = max_y.min(buffer_height as _) as usize;

    let x_start = _mm_set_ps(
        (pos.x + 0.5) + 1.0, 
        (pos.x + 0.5) + 0.0, 
        (pos.x + 0.5) + 1.0, 
        (pos.x + 0.5) + 0.0);
   
    let mut y_circle = _mm_set_ps(
        (pos.y + 0.5) + 1.0, 
        (pos.y + 0.5) + 1.0, 
        (pos.y + 0.5) + 0.0, 
        (pos.y + 0.5) + 0.0);

    let y_increment = _mm_set1_ps(2.0);
    let x_increment = _mm_set1_ps(2.0);

    // New: Set both inner and outer radii
    let inner_radius_ps = _mm_set_ps1(inner_radius);
    let border_radius_ps = _mm_set_ps1(border_radius);

    let inner_color_ps = _mm_set_ps(25000.0, 0.0, 25000.0, 25000.0);
    let border_color_ps = _mm_set_ps(0.0, 25000.0, 25000.0, 0.0);

    // Load both inner and outer colors
    //let inner_color_ps = _mm_set_ps(inner_color.r as f32, inner_color.g as f32, inner_color.b as f32, 0.0);
    //let border_color_ps = _mm_set_ps(border_color.r as f32, border_color.g as f32, border_color.b as f32, 0.0);
    let one_const = _mm_set_ps1(1.0);
    let zero_const = _mm_set_ps1(0.0);

    for y in (min_y..max_y).step_by(2) {
        let mut x_vals = x_start;
        let y2 = _mm_mul_ps(y_circle, y_circle);

        for x in (min_x..max_x).step_by(2) {
            let x2 = _mm_mul_ps(x_vals, x_vals);
            let x2_y2 = _mm_add_ps(x2, y2);

            // Calculate distance to the 2x2 quad (distance to center)
            let dist_to_center = _mm_sqrt_ps(x2_y2);

            let dist_inner = _mm_sub_ps(dist_to_center, inner_radius_ps);
            let dist_inner = _mm_sub_ps(one_const, _mm_max_ps(zero_const, _mm_min_ps(dist_inner, one_const)));

            let dist_border = _mm_sub_ps(dist_to_center, border_radius_ps);
            let dist_border = _mm_sub_ps(one_const, _mm_max_ps(zero_const, _mm_min_ps(dist_border, one_const)));

            // Calculate the difference between border color and inner color: (border_color - inner_color)
            let color_diff = _mm_sub_ps(border_color_ps, inner_color_ps);

            // Multiply the difference by the blend factor: blend_factor * (border_color - inner_color)
            let f0 = _mm_mul_ps(color_diff, _mm_shuffle_ps(dist_inner, dist_inner, 0b00_00_00_00));
            let f1 = _mm_mul_ps(color_diff, _mm_shuffle_ps(dist_inner, dist_inner, 0b01_01_01_01));
            let f2 = _mm_mul_ps(color_diff, _mm_shuffle_ps(dist_inner, dist_inner, 0b10_10_10_10));
            let f3 = _mm_mul_ps(color_diff, _mm_shuffle_ps(dist_inner, dist_inner, 0b11_11_11_11));

            let f0 = _mm_add_ps(inner_color_ps, f0);
            let f1 = _mm_add_ps(inner_color_ps, f1);
            let f2 = _mm_add_ps(inner_color_ps, f2);
            let f3 = _mm_add_ps(inner_color_ps, f3);

            // Blend with the edge
            let c0_mul = _mm_shuffle_ps(dist_border, dist_border, 0b00_00_00_00);
            let c1_mul = _mm_shuffle_ps(dist_border, dist_border, 0b01_01_01_01);
            let c2_mul = _mm_shuffle_ps(dist_border, dist_border, 0b10_10_10_10);
            let c3_mul = _mm_shuffle_ps(dist_border, dist_border, 0b11_11_11_11);

            let c0 = _mm_mul_ps(f0, c0_mul);
            let c1 = _mm_mul_ps(f1, c1_mul);
            let c2 = _mm_mul_ps(f2, c2_mul);
            let c3 = _mm_mul_ps(f3, c3_mul);

            let col0 = pack_float_to_color16(c0, c1);
            let col1 = pack_float_to_color16(c2, c3);

            let idx0 = ((y + 0) * buffer_width) + x;
            let idx1 = ((y + 1) * buffer_width) + x;

            let offset0 = output.add(idx0);
            let offset1 = output.add(idx1);

            _mm_storeu_si128(offset0 as *mut _, col0);
            _mm_storeu_si128(offset1 as *mut _, col1);

            x_vals = _mm_add_ps(x_vals, x_increment);
        }

        y_circle = _mm_add_ps(y_circle, y_increment);
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
        WindowOptions {
            scale: Scale::X4,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.set_target_fps(60);
    let mut offset = 63.0f32;

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

        let c0 = Color32::new(0xff,  0xff, 0xff, 0xff); 
        let c1 = Color32::new(0x00,  0xff, 0x00, 0xff); 

        let primitive = Primitive {
            rect: Rect::new(20.0, 10.0, 100.0, 100.0),
            uvs: [Uv::new(0.0, 0.0); 4],
            colors: [c0, c1, c1, c0],
            _corners: [0.0; 4],
            _texture_handle: 0,
        };

        let primitives = [primitive];

        /*
        offset -= 0.02;
        let offset = offset.max(1.0);

        let tile = sw_renderer.tiles[0];
        */

        sw_renderer.render(&mut buffer, WIDTH, HEIGHT, &primitives);

        //sw_renderer.clear_tile();
        //sw_renderer.quad_ref_renderer(&tile, &primitive);
        
        /*
        unsafe {
            render_corner_with_border(
                sw_renderer.tile_buffer.as_mut_ptr(),
                128, 128,
                64.0,
                offset,
                Vec2::new(0.0, 0.0),
                Corner::TopLeft);
        }
        */

        //sw_renderer.test_render_in_tile();
        //sw_renderer.copy_tile_buffer_to_output(buffer.as_mut_ptr());

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_left_corner() {
        let pos = Vec2 { x: 50.0, y: 50.0 };
        let radius = 20.0;
        let corner = Corner::TopLeft;
        let (min_x, max_x, min_y, max_y) = calculate_bounding_box(pos, radius, corner);
        
        // Expected bounding box for TopLeft
        assert_eq!(min_x, 30);
        assert_eq!(max_x, 50);
        assert_eq!(min_y, 30);
        assert_eq!(max_y, 50);
    }

    #[test]
    fn test_top_right_corner() {
        let pos = Vec2 { x: 50.0, y: 50.0 };
        let radius = 20.0;
        let corner = Corner::TopRight;
        let (min_x, max_x, min_y, max_y) = calculate_bounding_box(pos, radius, corner);

        // Expected bounding box for TopRight
        assert_eq!(min_x, 50);
        assert_eq!(max_x, 70);
        assert_eq!(min_y, 30);
        assert_eq!(max_y, 50);
    }

    #[test]
    fn test_bottom_left_corner() {
        let pos = Vec2 { x: 50.0, y: 50.0 };
        let radius = 20.0;
        let corner = Corner::BottomLeft;
        let (min_x, max_x, min_y, max_y) = calculate_bounding_box(pos, radius, corner);

        // Expected bounding box for BottomLeft
        assert_eq!(min_x, 30);
        assert_eq!(max_x, 50);
        assert_eq!(min_y, 50);
        assert_eq!(max_y, 70);
    }

    #[test]
    fn test_bottom_right_corner() {
        let pos = Vec2 { x: 50.0, y: 50.0 };
        let radius = 20.0;
        let corner = Corner::BottomRight;
        let (min_x, max_x, min_y, max_y) = calculate_bounding_box(pos, radius, corner);

        // Expected bounding box for BottomRight
        assert_eq!(min_x, 50);
        assert_eq!(max_x, 70);
        assert_eq!(min_y, 50);
        assert_eq!(max_y, 70);
    }

    #[test]
    fn test_small_radius() {
        let pos = Vec2 { x: 50.0, y: 50.0 };
        let radius = 5.0;
        let corner = Corner::TopLeft;
        let (min_x, max_x, min_y, max_y) = calculate_bounding_box(pos, radius, corner);

        // Expected bounding box for a small radius
        assert_eq!(min_x, 45);
        assert_eq!(max_x, 50);
        assert_eq!(min_y, 45);
        assert_eq!(max_y, 50);
    }

    #[test]
    fn test_large_radius() {
        let pos = Vec2 { x: 100.0, y: 100.0 };
        let radius = 50.0;
        let corner = Corner::BottomRight;
        let (min_x, max_x, min_y, max_y) = calculate_bounding_box(pos, radius, corner);

        // Expected bounding box for a large radius
        assert_eq!(min_x, 100);
        assert_eq!(max_x, 150);
        assert_eq!(min_y, 100);
        assert_eq!(max_y, 150);
    }
}

