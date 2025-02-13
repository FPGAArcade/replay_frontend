use flowi_core::Color16;
use flowi_renderer::Renderer;
use flowi_sw_renderer::Renderer as SoftwareRenderer;
use flowi_sw_renderer::{BlendMode, Corner, Raster, TileInfo};

use minifb::{Key, Window, WindowOptions};
use simd::*;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

const RENDER_WIDTH: usize = WIDTH / 4;
const RENDER_HEIGHT: usize = HEIGHT / 4;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum Shape {
    Quad,
    RoundRect,
    RoundedTopLeft,
    RoundedTopRight,
    RoundedBottomLeft,
    RoundedBottomRight,
    TextBuffer,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum RenderMode {
    Flat,
    Gradient,
    Texture,
    TextureFlat,
    TexureGradient,
}

/*
#[derive(Debug, Clone, Copy)]
enum BlendMode {
    None,
    Background,
    BackgroundGraidient,
    Multiply,
}
*/

fn zoom_buffer(output: &mut [u32], input: &[u32], zoom: usize) {
    // Perform the zoom operation only for the valid source region
    for y in 0..RENDER_WIDTH {
        for x in 0..RENDER_HEIGHT {
            let color = input[y * RENDER_WIDTH + x];
            let start_y = y * zoom;
            let start_x = x * zoom;

            // Write the zoomed block directly without further checks
            for dy in 0..zoom {
                let target_y = start_y + dy;

                if target_y >= HEIGHT {
                    return;
                }

                let target_row = &mut output[target_y * WIDTH..(target_y + 1) * WIDTH];
                for dx in 0..zoom {
                    if start_x + dx >= WIDTH {
                        break;
                    }

                    target_row[start_x + dx] = color;
                }
            }
        }
    }
}

#[allow(dead_code)]
fn draw_pixel_grid(output: &mut [u32], zoom: usize) {
    assert!(zoom > 0, "Zoom size must be greater than 0");

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            // Draw a line every `zoom` pixels in both x and y directions
            if x % zoom == 0 || y % zoom == 0 {
                output[y * WIDTH + x] = 0x00FF0000; // Black color for the grid lines
            }
        }
    }
}

/*
fn generate_sample_test_image() -> Image {
    // colors
    let colors = [
        (121,209,81), (253,231,36), (52, 94, 141), (68, 190, 112), (189, 222, 48),
        (68, 112, 112), (41, 120, 142), (34, 167, 132), (72, 45, 116), (64, 67, 135),
        (41, 120, 142), (68,190,112), (64,67,135), (189,222,38), (68,1,84),
        (72, 35, 116), (64, 67, 135), (52,94,141), (41,120,142), (32,144,140),
        (41,120,142), (68,190,112), (68,1,84), (52,94,141), (72,35,116)
    ];

}

 */

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut tile_output = vec![Color16::default(); RENDER_WIDTH * RENDER_HEIGHT * 4];
    let mut tile_output_u32 = vec![0; RENDER_WIDTH * RENDER_HEIGHT * 4];
    let linear_to_srgb_table = flowi_sw_renderer::build_linear_to_srgb_table();
    let _application_settings = flowi_core::ApplicationSettings {
        width: WIDTH,
        height: HEIGHT,
    };

    let mut core = flowi_core::Ui::new(Box::new(SoftwareRenderer::new((WIDTH, HEIGHT), None)));
    let font = core
        .load_font("data/fonts/roboto/Roboto-Regular.ttf")
        .unwrap();

    let text_to_render = "Hello";

    core.queue_generate_text(text_to_render, 16, font);

    let mut raster = Raster::new();
    raster.scissor_rect = f32x4::new(0.0, 0.0, RENDER_WIDTH as f32, RENDER_HEIGHT as f32);

    /*
    let mut text_test = vec![0i16; 128 * 128];

    for y in 0..128 {
        for x in 0..128 {
            if (x & 1) == 0 {
                text_test[y * 128 + x] = 0x7fff;
            }
            text_test[y * 128 + x] = (((y ^ x) as i16) * 64) & 0x7fff;
        }
    }
    */

    //let radius = 31.0; // actually 16

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: false,
            scale: minifb::Scale::X1,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let shape = Shape::RoundedTopLeft;
    let _render_mode = RenderMode::Flat;
    let _blend_mode = BlendMode::None;

    // Limit to max ~60 fps update rate
    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in tile_output.iter_mut() {
            *i = Color16::new_splat(0x4000);
        }

        if let Some(text) = core.get_text(text_to_render, 16, font) {
            render_shapes(
                &mut tile_output_u32,
                &mut tile_output,
                text.data.0 as _,
                text.width as _,
                &raster,
                shape,
                &[
                    0.0,
                    0.0,
                    RENDER_WIDTH.min(text.width as _) as _,
                    HEIGHT.min(text.height as _) as _,
                ],
                i16x8::new(
                    0x7fff, 0x7fff, 0x7fff, 0x7fff, 0x7fff, 0x7fff, 0x7fff, 0x7fff,
                ),
                i16x8::new(
                    0x7fff, 0x7fff, 0x7fff, 0x7fff, 0x7fff, 0x7fff, 0x7fff, 0x7fff,
                ),
                &linear_to_srgb_table,
            );
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();

        let zoom = 16;

        zoom_buffer(&mut buffer, &tile_output_u32, zoom);
        //draw_pixel_grid(&mut buffer, zoom);

        core.update();
    }
}

fn render_shapes(
    output: &mut [u32],
    temp_output: &mut [Color16],
    text_object: *const i16,
    text_object_width: usize,
    raster: &Raster,
    shape: Shape,
    coords: &[f32; 4],
    color_top: i16x8,
    _color_botttom: i16x8,
    linear_to_srgb_table: &[u8; 4096],
) {
    let radius = 16.0;

    let tile_info = TileInfo {
        offsets: f32x4::new_splat(0.0),
        width: RENDER_WIDTH as i32,
        _height: RENDER_HEIGHT as i32,
    };

    match shape {
        Shape::Quad => {
            raster.render_solid_quad(temp_output, &tile_info, coords, color_top, BlendMode::None);
        }

        Shape::RoundedTopLeft => {
            let coords = [0.0, 0.0, radius + 1.0, radius + 1.0];
            raster.render_solid_rounded_corner(
                temp_output,
                &tile_info,
                &coords,
                color_top,
                radius,
                BlendMode::None,
                Corner::TopLeft,
            );
        }

        Shape::RoundedTopRight => {
            let coords = [0.0, 0.0, radius + 1.0, radius + 1.0];
            raster.render_solid_rounded_corner(
                temp_output,
                &tile_info,
                &coords,
                color_top,
                radius,
                BlendMode::None,
                Corner::TopRight,
            );
        }

        Shape::RoundedBottomLeft => {
            let coords = [0.0, 0.0, radius + 1.0, radius + 1.0];
            raster.render_solid_rounded_corner(
                temp_output,
                &tile_info,
                &coords,
                color_top,
                radius,
                BlendMode::None,
                Corner::BottomLeft,
            );
        }

        Shape::RoundedBottomRight => {
            let coords = [0.0, 0.0, radius + 1.0, radius + 1.0];
            raster.render_solid_rounded_corner(
                temp_output,
                &tile_info,
                &coords,
                color_top,
                radius,
                BlendMode::None,
                Corner::BottomRight,
            );
        }

        Shape::RoundRect => {
            let radius = [radius, radius, radius, radius];
            raster.render_solid_quad_rounded(
                temp_output,
                &tile_info,
                coords,
                color_top,
                &radius,
                BlendMode::None,
            );
        }

        Shape::TextBuffer => {
            raster.render_text_texture(
                temp_output,
                text_object,
                &tile_info,
                text_object_width,
                coords,
                color_top,
            );
        }
    }

    copy_tile_linear_to_srgb(linear_to_srgb_table, output, temp_output);
}

#[inline(never)]
pub fn copy_tile_linear_to_srgb(
    linear_to_srgb_table: &[u8; 4096],
    output: &mut [u32],
    tile: &[Color16],
) {
    let tile_width = RENDER_WIDTH;
    let tile_height = RENDER_HEIGHT;
    let width = RENDER_WIDTH;

    let mut tile_ptr = tile.as_ptr();
    let mut output_index = 0;

    for _y in 0..tile_height {
        let mut current_index = output_index;
        for _x in 0..(tile_width >> 1) {
            let rgba_rgba = i16x8::load_unaligned_ptr(tile_ptr);
            let rgba_rgba = rgba_rgba.shift_right::<3>();

            let r0 = (rgba_rgba.extract::<0>() as u16) & 0xfff;
            let g0 = (rgba_rgba.extract::<1>() as u16) & 0xfff;
            let b0 = (rgba_rgba.extract::<2>() as u16) & 0xfff;

            let r1 = (rgba_rgba.extract::<4>() as u16) & 0xfff;
            let g1 = (rgba_rgba.extract::<5>() as u16) & 0xfff;
            let b1 = (rgba_rgba.extract::<6>() as u16) & 0xfff;

            unsafe {
                let r0 = *linear_to_srgb_table.get_unchecked(r0 as usize) as u32;
                let g0 = *linear_to_srgb_table.get_unchecked(g0 as usize) as u32;
                let b0 = *linear_to_srgb_table.get_unchecked(b0 as usize) as u32;

                let r1 = *linear_to_srgb_table.get_unchecked(r1 as usize) as u32;
                let g1 = *linear_to_srgb_table.get_unchecked(g1 as usize) as u32;
                let b1 = *linear_to_srgb_table.get_unchecked(b1 as usize) as u32;

                let rgb0 = (r0 << 16) | (g0 << 8) | b0;
                let rgb1 = (r1 << 16) | (g1 << 8) | b1;

                tile_ptr = tile_ptr.add(8);

                *output.get_unchecked_mut(current_index + 0) = rgb0;
                *output.get_unchecked_mut(current_index + 1) = rgb1;
            }

            current_index += 2;
        }

        output_index += width;
    }
}
