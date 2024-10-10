use flowi_core::primitives::{Color32, Uv, IRect, Primitive};

// Number of bits to repserent a color channel in sRGB color space. We use 16-bit colors to allow
// for high range of colors. Most input images are in 8-bit sRGB color space, but as we convert
// thesee to linear color space, we need to use higher bit depth to avoid banding artifacts.

const SRGB_BIT_COUNT: u32 = 12;
const LINEAR_BIT_COUNT: u32 = 15;
const LINEAR_TO_SRGB_SHIFT: u32 = LINEAR_BIT_COUNT - SRGB_BIT_COUNT;

pub const CORNER_TOP_LEFT: usize = 0;
pub const CORNER_TOP_RIGHT: usize = 1;
pub const CORNER_BOTTOM_RIGHT: usize = 2;
pub const CORNER_BOTTOM_LEFT: usize = 3; 

const COLORS: [u32; 16] = [
    0x0FF5733, // Red-Orange
    0x0DAF7A6, // Green-Mint
    0x0FFC300, // Bright Yellow
    0x0900C3F, // Deep Blue
    0x0C70039, // Dark Red
    0x02ECC71, // Emerald Green
    0x09B59B6, // Purple
    0x0F39C12, // Bright Green
    0x0A569BD, // Sky Blue
    0x0F1C40F, // Forest Green
    0x08E44AD, // Red
    0x02C3E50, // Teal
    0x0BDC3C7, // Silver
    0x09B870C, // Dark Cyan
    0x0E74C3C, // Soft Blue
    0x0D35400, // Burnt Orange
];

fn linear_to_srgb(x: f32) -> f32 {
    if x <= 0.0031308 {
        x * 12.92
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    }
}

fn srgb_to_linear(x: f32) -> f32 {
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

fn build_srgb_to_linear_table() -> [u16; 1 << 8] {
    let mut table = [0; 1 << 8];

    for i in 0..(1 << 8) {
        let srgb = i as f32 / (1 << 8) as f32;
        let linear = srgb_to_linear(srgb);
        table[i] = (linear * (1 << LINEAR_BIT_COUNT) as f32) as u16;
        dbg!(table[i]);
    }

    table
}

fn build_linear_to_srgb_table() -> [u8; 1 << SRGB_BIT_COUNT] {
    let mut table = [0; 1 << SRGB_BIT_COUNT];

    for i in 0..(1 << SRGB_BIT_COUNT) {
        let linear = i as f32 / (1 << SRGB_BIT_COUNT) as f32;
        let srgb = linear_to_srgb(linear);
        table[i] = (srgb * (1 << 8) as f32) as u8;
    }

    table
}

#[derive(Debug, Clone, Copy)]
struct ColorF32_16 {
    r: f32,
    g: f32,
    b: f32,
    a: f32, 
}

#[derive(Debug, Copy, Clone)]
struct TilePosition {
    x: i16,
    y: i16,
}

#[derive(Debug, Copy, Clone)]
pub struct Tile {
    min: TilePosition,
    max: TilePosition,
    local_tile_index: usize,
}

impl ColorF32_16 {
    fn new_from_color32_srgb(color: Color32) -> Self {
        Self {
            r: srgb_to_linear(color.r as f32 / 255.0),
            g: srgb_to_linear(color.g as f32 / 255.0),
            b: srgb_to_linear(color.b as f32 / 255.0),
            a: color.a as f32 / 255.0,
        }
    }

    fn interpolate(a: Self, b: Self, t: f32) -> Self {
        Self {
            r: a.r * (1.0 - t) + b.r * t,
            g: a.g * (1.0 - t) + b.g * t,
            b: a.b * (1.0 - t) + b.b * t,
            a: a.a * (1.0 - t) + b.a * t,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
struct Color16 {
    r: u16,
    g: u16,
    b: u16,
    a: u16,
}

pub struct SwRenderer {
    linear_to_srgb: [u8; 1 << SRGB_BIT_COUNT],
    srgb_to_linear: [u16; 1 << 8],
    pub tiles: Vec<Tile>,
    tile_buffer: Vec<Color16>,
    tile_width: usize,
    tile_height: usize,
    screen_width: usize,
    screen_height: usize,
}

impl SwRenderer {
    pub fn new(screen_width: usize, screen_height: usize, tile_width: usize, tile_height: usize) -> Self {
        // TODO: Make sure to clamp against borders if not even divide
        let mut tiles =
            Vec::with_capacity((screen_width / tile_width) * (screen_height / tile_height));
        let mut tile_index = 0;

        for y in (0..screen_height).step_by(tile_height) {
            for x in (0..screen_width).step_by(tile_width) {
                tiles.push(Tile {
                    min: TilePosition {
                        x: x as i16,
                        y: y as i16,
                    },
                    max: TilePosition {
                        x: (x + tile_width) as i16,
                        y: (y + tile_height) as i16,
                    },
                    local_tile_index: tile_index & 0x3,
                });

                tile_index += 1;
            }
        }

        Self {
            tiles,
            tile_buffer: vec![Color16::default(); tile_width * tile_height],
            linear_to_srgb: build_linear_to_srgb_table(),
            srgb_to_linear: build_srgb_to_linear_table(),
            tile_width,
            tile_height,
            screen_width,
            screen_height,
        }
    }

    pub fn render(
        &mut self,
        dest: &mut [u32],
        width: usize,
        height: usize,
        primitives: &[Primitive],
    ) {
        let mut color_index = 0;

        for prim in primitives {
            let min_x = prim.rect.min[0] as usize;
            let min_y = prim.rect.min[1] as usize;
            let max_x = prim.rect.max[0] as usize;
            let max_y = prim.rect.max[1] as usize;

            let max_x = max_x.min(width);
            let max_y = max_y.min(height);
            let min_x = min_x.max(0);
            let min_y = min_y.max(0);
            let color = COLORS[color_index & 0xf];

            for y in min_y..max_y {
                for x in min_x..max_x {
                    dest[y * width + x] = color;
                }
            }

            color_index += 1;
        }
    }

    // Premultiplied alpha color
    fn color32_16_from_color32_srgb(&self, color: Color32) -> ColorF32_16 {
        let a = color.a as f32 * 1.0/255.0;
        ColorF32_16 {
            r: (self.srgb_to_linear[color.r as usize] as f32) * a,
            g: (self.srgb_to_linear[color.g as usize] as f32) * a,
            b: (self.srgb_to_linear[color.b as usize] as f32) * a,
            a: a * (1 << LINEAR_BIT_COUNT) as f32,
        }
    }


    // Reference implementation for quad rendering. This is used to compare the output of the of
    // the optimized renderer. It is not used in the final implementation as it will be way slower.
    //
    // Supported functionallity of this code:
    //
    // * Rounded corners
    // * Single texture lookup
    // * Linear color space
    // * 16-bit per channel output color buffer
    // * Color interpolation between the corners and blending with the texture.
    // * Clipping to the screen/tile bounds
    //
    pub fn quad_ref_renderer(&mut self, tile: &Tile, primitive: &Primitive) {
        // pixel center at (0.5, 0.5)
        let min_xf = (primitive.rect.min[0] - tile.min.x as f32) + 0.5;
        let min_yf = (primitive.rect.min[1] - tile.min.y as f32) + 0.5;
        let max_xf = (primitive.rect.max[0] - tile.min.x as f32) + 0.5;
        let max_yf = (primitive.rect.max[1] - tile.min.y as f32) + 0.5;

        // Clip the quad's min/max to the screen bounds, using truncation with the 0.5 pixel center
        // Add the offset before applying floor to get proper pixel center alignment
        let clipped_min_x = min_xf.floor().max(0.0);
        let clipped_min_y = min_yf.floor().max(0.0);
        let clipped_max_x = max_xf.floor().min(tile.max.x as f32);
        let clipped_max_y = max_yf.floor().min(tile.max.y as f32);

        //let x_length = clipped_max_x - clipped_min_x;
        //let y_length = clipped_max_y - clipped_min_y;

        let y_delta = 1.0 / (max_yf - min_yf);
        let x_delta = 1.0 / (max_xf - min_xf);

        // Calculate interpolation factors based on the clipped coordinates
        let x_min_step = (clipped_min_x - min_xf) * x_delta;
        let x_max_step = (clipped_max_x - min_xf) * x_delta;
        let y_min_step = (clipped_min_y - min_yf) * y_delta;
        let y_max_step = (clipped_max_y - min_yf) * y_delta;

        // Get the corner colors as linear f32
        let c_tl_color = self.color32_16_from_color32_srgb(primitive.colors[CORNER_TOP_LEFT]);
        let c_tr_color = self.color32_16_from_color32_srgb(primitive.colors[CORNER_TOP_RIGHT]);
        let c_br_color = self.color32_16_from_color32_srgb(primitive.colors[CORNER_BOTTOM_RIGHT]);
        let c_bl_color = self.color32_16_from_color32_srgb(primitive.colors[CORNER_BOTTOM_LEFT]);

        dbg!(c_tl_color, c_tr_color);
        dbg!(x_min_step, x_max_step);
        dbg!(y_min_step, y_max_step);

        // Interpolate horizontally first between top-left and top-right (for top side)
        let color_top_left = ColorF32_16::interpolate(c_tl_color, c_tr_color, y_min_step);
        let color_top_right = ColorF32_16::interpolate(c_tl_color, c_tr_color, y_max_step);

        // Interpolate horizontally first between bottom-left and bottom-right (for bottom side)
        let color_bottom_left = ColorF32_16::interpolate(c_bl_color, c_bl_color, x_min_step);
        let color_bottom_right = ColorF32_16::interpolate(c_br_color, c_br_color, x_max_step);

        // Interpolate vertically between bottom and top sides
        let uv_bottom_left = Uv::interpolate(primitive.uvs[0], primitive.uvs[1], x_min_step);
        let uv_bottom_right = Uv::interpolate(primitive.uvs[0], primitive.uvs[1], x_max_step);

        // Interpolate vertically between bottom and top sides
        let uv_top_left = Uv::interpolate(primitive.uvs[3], primitive.uvs[2], y_min_step);
        let uv_top_right = Uv::interpolate(primitive.uvs[3], primitive.uvs[2], y_max_step);

        let y_start = clipped_min_y as usize;
        let y_end = clipped_max_y as usize;
        let x_start = clipped_min_x as usize;
        let x_end = clipped_max_x as usize;
        let mut yc = 0.0;
        let one_shifted = (1 << LINEAR_BIT_COUNT) as f32;

        dbg!(color_top_left, color_bottom_left);
        dbg!(color_top_right, color_bottom_right);

        for y in y_start..y_end { 
            let mut xc = 0.0;

            let c0 = ColorF32_16::interpolate(color_top_left, color_bottom_left, yc);
            let c1 = ColorF32_16::interpolate(color_top_right, color_bottom_right, yc);

            let uv0 = Uv::interpolate(uv_bottom_left, uv_bottom_right, yc);
            let uv1 = Uv::interpolate(uv_top_left, uv_top_right, yc);

            let dest_row = &mut self.tile_buffer[(y * tile.max.x as usize)..]; 

            for x in x_start..x_end {
                let color = ColorF32_16::interpolate(c0, c1, xc);
                let uv = Uv::interpolate(uv0, uv1, xc);

                let dest_pixel = &mut dest_row[x]; 
                let one_minus_a = one_shifted - color.a;

                // in the ref renderer we use floats, but we still use 16-bit colors so we convert
                // to floats here to keep it easier

                let bg_r = dest_pixel.r as f32;
                let bg_g = dest_pixel.g as f32;
                let bg_b = dest_pixel.b as f32;
                let bg_a = dest_pixel.a as f32;

                let r = color.r + (bg_r * one_minus_a); 
                let g = color.g + (bg_g * one_minus_a); 
                let b = color.b + (bg_b * one_minus_a); 
                let a = color.a + (bg_a * one_minus_a); 

                dest_pixel.r = r as u16;
                dest_pixel.g = g as u16;
                dest_pixel.b = b as u16;
                dest_pixel.a = 0;//a as u16;

                xc += x_delta;
            }
        
            yc += y_delta;
        }
    }

    unsafe fn copy_tile_to_output(&self, output: *mut u32, render_width: usize, tile: &Tile) {
        let tile_min_x = tile.min.x as usize;
        let tile_min_y = tile.min.y as usize;

        let target_offset = (tile_min_y * render_width) + tile_min_x;

        // copy tile back to main buffer
        for y in 0..self.tile_height {
            // get target output slice
            let output_line = unsafe {
                std::slice::from_raw_parts_mut(
                    output.add(target_offset + y * render_width),
                    self.tile_width,
                )
            };
            
            let tile_line = &self.tile_buffer[y * self.tile_width..(y + 1) * self.tile_width];

            // Convert back to sRGB
            for (src_pixel, dst_pixel) in tile_line.iter().zip(output_line.iter_mut()) {
                let r = self.linear_to_srgb[(src_pixel.r >> LINEAR_TO_SRGB_SHIFT as u16) as usize] as u32;
                let g = self.linear_to_srgb[(src_pixel.g >> LINEAR_TO_SRGB_SHIFT as u16) as usize] as u32;
                let b = self.linear_to_srgb[(src_pixel.b >> LINEAR_TO_SRGB_SHIFT as u16) as usize] as u32;
                let a = self.linear_to_srgb[(src_pixel.a >> LINEAR_TO_SRGB_SHIFT as u16) as usize] as u32;
                *dst_pixel = (a << 24) | (r << 16) | (g << 8) | b;
            }
        }
    }

    pub fn test_render_in_tile(&mut self) {
        for y in 0..self.tile_height {
            for x in 0..self.tile_width {
                let v = (x ^ y) & 0xff;
                let color = self.srgb_to_linear[v];
                self.tile_buffer[(y * self.tile_width) + x] = Color16 {
                    r: color,
                    g: color,
                    b: color,
                    a: (LINEAR_BIT_COUNT - 1) as u16,
                };
            }
        }
    }

    pub fn copy_tile_buffer_to_output(&self, output: *mut u32) {
        for tile in &self.tiles {
            unsafe {
                self.copy_tile_to_output(output, self.screen_width, &tile);
            }
        }
    }
}
