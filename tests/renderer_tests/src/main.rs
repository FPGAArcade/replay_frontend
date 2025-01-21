use flowi_sw_renderer::{BlendMode, Raster, TileInfo, Corner};
use minifb::{Key, Window, WindowOptions};
use simd::*;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;

const RENDER_WIDTH: usize = WIDTH / 4;
const RENDER_HEIGHT: usize = HEIGHT / 4;

#[derive(Debug, Clone, Copy)]
enum Shape {
    Quad,
    RoundRect,
    RoundedTopLeft,
    RoundedTopRight,
    RoundedBottomLeft,
    RoundedBottomRight,
}

#[derive(Debug, Clone, Copy)]
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

fn draw_pixel_grid(output: &mut [u32], zoom: usize) {
    assert!(zoom > 0, "Zoom size must be greater than 0");

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            // Draw a line every `zoom` pixels in both x and y directions
            if x % zoom == 0 || y % zoom == 0 {
                output[y * WIDTH + x] = 0xFF000000; // Black color for the grid lines
            }
        }
    }
}
 
fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut tile_output = vec![0; RENDER_WIDTH * RENDER_HEIGHT * 4];
    let mut tile_output_u32 = vec![0; RENDER_WIDTH * RENDER_HEIGHT * 4];
    let linear_to_srgb_table = flowi_sw_renderer::build_linear_to_srgb_table();

    let mut raster = Raster::new();
    raster.scissor_rect = f32x4::new(0.0, 0.0, RENDER_WIDTH as f32, RENDER_HEIGHT as f32);



    let radius = 31.0; // actually 16 

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

    let mut shape = Shape::RoundRect;
    let mut render_mode = RenderMode::Flat;
    let mut blend_mode = BlendMode::None;

    // Limit to max ~60 fps update rate
    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in tile_output.iter_mut() {
            *i = 0;
        }

        render_shapes(
            &mut tile_output_u32,
            &mut tile_output,
            &raster,
            shape,
            &[10.0, 10.0, 200.0, 200.0], 
            i16x8::new(0x7fff,0x7fff,0x7fff,0x7fff,0x7fff,0x7fff,0x7fff,0x7fff),
            i16x8::new(0x7fff,0x7fff,0x7fff,0x7fff,0x7fff,0x7fff,0x7fff,0x7fff),
            &linear_to_srgb_table, 
        );
        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();

        let zoom = 1;

        zoom_buffer(&mut buffer, &tile_output_u32, zoom);
        //draw_pixel_grid(&mut buffer, zoom);
    }
}

fn render_shapes(
    output: &mut [u32],
    temp_output: &mut [i16],
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
            raster.render_solid_rounded_corner(temp_output, &tile_info, &coords, color_top, radius, BlendMode::None, Corner::TopLeft);
        }

        Shape::RoundedTopRight => {
            let coords = [0.0, 0.0, radius + 1.0, radius + 1.0];
            raster.render_solid_rounded_corner(temp_output, &tile_info, &coords, color_top, radius, BlendMode::None, Corner::TopRight);
        }

        Shape::RoundedBottomLeft => {
            let coords = [0.0, 0.0, radius + 1.0, radius + 1.0];
            raster.render_solid_rounded_corner(temp_output, &tile_info, &coords, color_top, radius, BlendMode::None, Corner::BottomLeft);
        }

        Shape::RoundedBottomRight => {
            let coords = [0.0, 0.0, radius + 1.0, radius + 1.0];
            raster.render_solid_rounded_corner(temp_output, &tile_info, &coords, color_top, radius, BlendMode::None, Corner::BottomRight);
        }

        Shape::RoundRect => {
            raster.render_solid_quad_rounded(temp_output, &tile_info, coords, color_top, radius, BlendMode::None);
        }
    }

    copy_tile_linear_to_srgb(linear_to_srgb_table, output, temp_output);
}

#[inline(never)]
pub fn copy_tile_linear_to_srgb(
    linear_to_srgb_table: &[u8; 4096],
    output: &mut [u32],
    tile: &[i16],
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

