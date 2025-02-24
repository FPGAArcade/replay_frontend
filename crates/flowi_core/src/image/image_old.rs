use simd::{f32x4, i16x8, i32x4};
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Color16 {
    pub r: i16,
    pub g: i16,
    pub b: i16,
    pub a: i16,
}

impl Color16 {
    pub fn new_splat(value: i16) -> Self {
        Self::new(value, value, value, value)
    }

    pub fn new(r: i16, g: i16, b: i16, a: i16) -> Self {
        Self { r, g, b, a }
    }
}

impl Default for Color16 {
    fn default() -> Self {
        Color16::new_splat(0)
    }
}

// Defines the type of border being used. Black means the border is black, Repeat means the border
// is repeated with the closest color next to the border.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BorderType {
    /// No border is used
    None,
    /// The border is black
    Black,
    /// The border is repeated with the closest color next to the border
    Repeat,
}

/// RenderImage that holds pixels of type Color16 (16-bit value per color channel)
/// The image data is made so there is a border around the image.
#[derive(Debug)]
pub struct RenderImage {
    /// The image data. TODO: Arena
    pub data: Vec<Color16>,
    /// Width of the image (this excludes the border)
    pub width: usize,
    /// Height of the image
    pub height: usize,
    /// Full width of the image including the border
    pub stride: usize,
    /// Border size
    pub border_size: usize,
    /// Start of the data excluding the border
    pub start_offset_ex_borders: usize,
    /// Type of border being used
    pub border_type: BorderType,
}

impl Default for RenderImage {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            width: 0,
            height: 0,
            stride: 0,
            border_size: 0,
            start_offset_ex_borders: 0,
            border_type: BorderType::None,
        }
    }
}

impl RenderImage {
    /// Get the real data of the image (excluding the border)
    #[inline]
    pub fn real_data(&self, offset: usize) -> &[Color16] {
        &self.data[self.border_size + offset..]
    }

    /// Gets the data image data with repect to a border offset. This is so we can
    /// sample "outside" of the image for cases where we do samples around a pixel.
    /// This is useful if we have a border size of two we can start from border 1.
    #[inline]
    pub fn data_with_border(&self, border: usize) -> &[Color16] {
        let border_offset = (border * self.stride) + border;
        &self.data[border_offset..]
    }
}

fn apply_falloff(v: i16x8, x_pos: usize, y_pos: usize, width: usize, height: usize) -> i16x8 {
    // TODO: Optimize
    let width_f = width as f32;
    let height_f = height as f32;
    let dx = x_pos as f32 / width_f;
    let dy = (height - y_pos) as f32 / height_f;

    // Removed .powf(1.0) since it does nothing.
    let alpha_factor = (dx * dy) * 32767.0;

    i16x8::mul_high(v, i16x8::new_splat(alpha_factor as i16))
}

fn calculate_scale_factor(
    original_width: usize,
    original_height: usize,
    target_width: usize,
    target_height: usize,
) -> usize {
    let max_scale_x = target_width / original_width;
    let max_scale_y = target_height / original_height;
    max_scale_x.min(max_scale_y).max(1) // Ensure at least 1x scaling
}

pub enum Falloff {
    Enabled,
    Disabled,
}

pub fn upscale_image_integer(
    data: &[Color16],
    size: (usize, usize),
    target_size: (usize, usize),
    falloff: Falloff,
) -> RenderImage {
    let scale = calculate_scale_factor(size.0, size.1, target_size.0, target_size.1);
    let out_width = size.0 * scale;
    let out_height = size.1 * scale;

    let mut output = RenderImage {
        data: vec![Color16::default(); out_width * out_height], // TODO: Arena
        width: out_width,
        height: out_height,
        border_size: 0,
        start_offset_ex_borders: 0,
        stride: out_width,
        border_type: BorderType::None,
    };

    for y in 0..size.1 {
        for x in 0..(size.0 >> 1) {
            let rgba0_rgba1 = i16x8::load_unaligned(data, (y * size.0) + (x * 2)); // Load two pixels

            let start_y = y * scale;
            let start_x = x * scale * 2;

            for dy in 0..scale {
                let target_y = start_y + dy;
                let target_y_offset = target_y * out_width;

                let mut dx;
                let mut color = rgba0_rgba1.shuffle::<0x0123_0123>(); // Start with rgba0

                for i in 0..2 {
                    dx = 0;
                    let base_x = start_x + i * scale; // Ensure `rgba1` starts at the correct offset

                    while dx < scale {
                        let current_x = base_x + dx; // The actual output x-coordinate
                        let current_y = target_y; // The actual output y-coordinate

                        let adjust_color = match falloff {
                            Falloff::Enabled => {
                                apply_falloff(color, current_x, current_y, out_width, out_height)
                            }
                            Falloff::Disabled => color,
                        };

                        //let falloff_factor = compute_falloff(current_x, current_y); // Pass correct (x, y)

                        // Store using SIMD-friendly vectorized writes
                        if dx + 1 < scale {
                            adjust_color.store_unaligned(
                                &mut output.data,
                                target_y_offset + current_x,
                            );
                            dx += 2;
                        } else {
                            adjust_color.store_unaligned_lower(
                                &mut output.data,
                                target_y_offset + current_x,
                            );

                            dx += 1;
                        }
                    }

                    // Switch to rgba1 for the next iteration
                    color = rgba0_rgba1.shuffle::<0x4567_4567>();
                }
            }
        }
    }

    output
}


pub fn add_border(image: &RenderImage, border_size: usize, border_type: BorderType) -> RenderImage {
    let full_width = image.width + border_size * 2;
    let full_height = image.height + border_size * 2;
    let total_size = full_width * full_height;

    let mut new_image = RenderImage {
        data: vec![Color16::default(); total_size],
        width: image.width,
        height: image.height,
        stride: full_width,
        border_size,
        start_offset_ex_borders: border_size * full_width + border_size,
        border_type,
    };

    // Copy the old image into the new image
    for y in 0..image.height {
        for x in 0..image.width {
            let old_index = y * image.width + x;
            let new_index = (y + border_size) * full_width + x + border_size;
            new_image.data[new_index] = image.data[old_index];
        }
    }

    if border_type == BorderType::Black {
        return new_image;
    }

    let source_line_top = &image.data[0..image.width];
    let source_line_bottom = &image.data[(image.height - 1) * image.width..];

    // Fill the top and bottom borders
    for y in 0..border_size {
        let index_top = y * new_image.stride;
        let target_start = index_top + border_size;
        let index_bottom = (full_height - 1 - y) * full_width;
        let target_end = index_bottom + border_size;

        new_image.data[target_start..target_start + image.width].copy_from_slice(source_line_top);
        new_image.data[target_end..target_end + image.width].copy_from_slice(source_line_bottom);
    }

    // Fill the left and right borders
    for y in 0..image.height {
        let left_pixel = image.data[y * image.width];
        let right_pixel = image.data[(y * image.width) + image.width - 1];

        let start_index = (y + border_size) * full_width;
        let end_index = start_index + image.width + border_size;

        let fill_range_start = &mut new_image.data[start_index..start_index + border_size];

        for x in 0..border_size {
            fill_range_start[x] = left_pixel;
        }

        let fill_range_end = &mut new_image.data[end_index..end_index + border_size];

        for x in 0..border_size {
            fill_range_end[x] = right_pixel;
        }
    }

    let corner0_color = image.data[0];
    let corner1_color = image.data[image.width - 1];
    let corner2_color = image.data[(image.height - 1) * image.width];
    let corner3_color = image.data[(image.height - 1) * image.width + image.width - 1];

    // Fill the corners
    for y in 0..border_size {
        for x in 0..border_size {
            new_image.data[y * full_width + x] = corner0_color;
            new_image.data[y * full_width + full_width - 1 - x] = corner1_color;
            new_image.data[(full_height - 1 - y) * full_width + x] = corner2_color;
            new_image.data[(full_height - 1 - y) * full_width + full_width - 1 - x] = corner3_color;
        }
    }

    new_image
}

fn cubic_hermite_f32(ai: f32, bi: f32, ci: f32, di: f32, t: f32) -> f32 {
    let a = -ai / 2.0 + (3.0 * bi) / 2.0 - (3.0 * ci) / 2.0 + di / 2.0;
    let b = ai - (5.0 * bi) / 2.0 + 2.0 * ci - di / 2.0;
    let c = -ai / 2.0 + ci / 2.0;
    let d = bi;
    let t2 = t * t;
    let t3 = t2 * t;

    a * t3 + b * t2 + c * t + d
}

fn cubic_hermite_c16(a: Color16, b: Color16, c: Color16, d: Color16, t: f32) -> Color16 {
    let c0 = cubic_hermite_f32(a.r as f32, b.r as f32, c.r as f32, d.r as f32, t);
    let c1 = cubic_hermite_f32(a.g as f32, b.g as f32, c.g as f32, d.g as f32, t);
    let c2 = cubic_hermite_f32(a.b as f32, b.b as f32, c.b as f32, d.b as f32, t);
    let c3 = cubic_hermite_f32(a.a as f32, b.a as f32, c.a as f32, d.a as f32, t);

    let c0 = c0.max(0.0).min(32767.0) as i16;
    let c1 = c1.max(0.0).min(32767.0) as i16;
    let c2 = c2.max(0.0).min(32767.0) as i16;
    let c3 = c3.max(0.0).min(32767.0) as i16;

    Color16::new(c0, c1, c2, c3)
}

fn get_sample(image: &[Color16], y: i32, x: i32, width: usize) -> Color16 {
    let x = x.max(0).min(4);
    let y = y.max(0).min(4);
    image[y as usize * width + x as usize]
}

pub const SAMPLING_METHOD_REF_BICUBIC : usize = 0;
pub const SAMPLING_METHOD_REF_BILINEAR : usize = 1;

pub fn draw_scaled_image<const SAMPLING_METHOD: usize>(
    output: &mut [Color16],
    scissor_rect: f32x4,
    image: &RenderImage,
    tile_offsets: f32x4,
    tile_width: usize,
    coords: &[f32],
    _color: i16x8)
{
    let x0y0x1y1_adjust =
        (f32x4::load_unaligned(coords) - tile_offsets) + f32x4::new_splat(0.5);
    let x0y0x1y1 = x0y0x1y1_adjust.floor();
    let x0y0x1y1_int = x0y0x1y1.as_i32x4();

    // Make sure we intersect with the scissor rect otherwise skip rendering
    if !f32x4::test_intersect(scissor_rect, x0y0x1y1) {
        return;
    }

    // Calculate the difference between the scissor rect and the current rect
    // if diff is > 0 we return back a positive value to use for clipping
    let clip_diff = (x0y0x1y1_int - scissor_rect.as_i32x4())
        .min(i32x4::new_splat(0))
        .abs();

    let clip_x = clip_diff.extract::<0>() as usize;
    let clip_y = clip_diff.extract::<1>() as usize;

    let min_box = x0y0x1y1_int.min(scissor_rect.as_i32x4());
    let max_box = x0y0x1y1_int.max(scissor_rect.as_i32x4());

    let x0 = max_box.extract::<0>() as usize;
    let y0 = max_box.extract::<1>() as usize;
    let x1 = min_box.extract::<2>() as usize;
    let y1 = min_box.extract::<3>() as usize;

    let ylen = y1 - y0;
    let xlen = x1 - x0;

    let output = &mut output[(y0 as usize * tile_width) + x0..];
    let image_data = image.real_data(clip_y * image.stride + clip_x);

    let y_step = ((image.height as u32) << 15) / ylen as u32;
    let x_step = ((image.width as u32) << 15) / xlen as u32;
    let mut y_current = 0u32;

    for y in 0..ylen {
        let y_int = (y_current >> 15) as i32;
        let y_fract = (y_current & 0x7fff) as u16;
        let mut x_current = 0;
        let v_fraction = i16x8::new_splat(y_fract as i16);

        for x in 0..xlen {
            let x_int = ((x_current >> 15)) as i32;
            let x_fract = (x_current & 0x7fff) as u16;
            let offset = (y * tile_width) + x;

            if SAMPLING_METHOD == SAMPLING_METHOD_REF_BICUBIC {
                let p00 = get_sample(image_data, x_int - 1, y_int - 1, image.stride);
                let p10 = get_sample(image_data, x_int + 0, y_int - 1, image.stride);
                let p20 = get_sample(image_data, x_int + 1, y_int - 1, image.stride);
                let p30 = get_sample(image_data, x_int + 2, y_int - 1, image.stride);

                let p01 = get_sample(image_data, x_int - 1, y_int + 0, image.stride);
                let p11 = get_sample(image_data, x_int + 0, y_int + 0, image.stride);
                let p21 = get_sample(image_data, x_int + 1, y_int + 0, image.stride);
                let p31 = get_sample(image_data, x_int + 2, y_int + 0, image.stride);

                let p02 = get_sample(image_data, x_int - 1, y_int + 1, image.stride);
                let p12 = get_sample(image_data, x_int + 0, y_int + 1, image.stride);
                let p22 = get_sample(image_data, x_int + 1, y_int + 1, image.stride);
                let p32 = get_sample(image_data, x_int + 2, y_int + 1, image.stride);

                let p03 = get_sample(image_data, x_int - 1, y_int + 2, image.stride);
                let p13 = get_sample(image_data, x_int + 0, y_int + 2, image.stride);
                let p23 = get_sample(image_data, x_int + 1, y_int + 2, image.stride);
                let p33 = get_sample(image_data, x_int + 2, y_int + 2, image.stride);

                let col0 = cubic_hermite_c16(p00, p10, p20, p30, x_fract as f32 / 32768.0);
                let col1 = cubic_hermite_c16(p01, p11, p21, p31, x_fract as f32 / 32768.0);
                let col2 = cubic_hermite_c16(p02, p12, p22, p32, x_fract as f32 / 32768.0);
                let col3 = cubic_hermite_c16(p03, p13, p23, p33, x_fract as f32 / 32768.0);
                let val = cubic_hermite_c16(col0, col1, col2, col3, y_fract as f32 / 32768.0);

                output[offset] = val;
            } else {
                let image_offset = (y_int as usize * image.stride) + x_int as usize;

                let u_fraction = i16x8::new_splat(x_fract as i16);
                let rgba_rgba_0 = i16x8::load_unaligned(image_data, image_offset);
                let rgba_rgba_1 = i16x8::load_unaligned(image_data, image_offset + image.stride);
                let t0_t1 = i16x8::lerp(rgba_rgba_0, rgba_rgba_1, v_fraction);
                let t = t0_t1.rotate_4();
                let res = i16x8::lerp(t0_t1, t, u_fraction);
                res.store_unaligned_lower(output, offset);
            }

            x_current += x_step;
        }

        y_current += y_step;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to compare two Color16 values.
    fn assert_color_eq(actual: &Color16, expected: &Color16) {
        assert_eq!(actual.r, expected.r, "r channel mismatch");
        assert_eq!(actual.g, expected.g, "g channel mismatch");
        assert_eq!(actual.b, expected.b, "b channel mismatch");
        assert_eq!(actual.a, expected.a, "a channel mismatch");
    }

    #[test]
    fn test_color16_default() {
        let c = Color16::default();
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 0);
    }

    #[test]
    fn test_color16_new_splat() {
        let c = Color16::new_splat(42);
        assert_eq!(c.r, 42);
        assert_eq!(c.g, 42);
        assert_eq!(c.b, 42);
        assert_eq!(c.a, 42);
    }

    #[test]
    fn test_color16_new() {
        let c = Color16::new(1, 2, 3, 4);
        assert_eq!(c.r, 1);
        assert_eq!(c.g, 2);
        assert_eq!(c.b, 3);
        assert_eq!(c.a, 4);
    }

    #[test]
    fn test_calculate_scale_factor() {
        assert_eq!(calculate_scale_factor(2, 2, 4, 4), 2);
        // When the target is smaller than twice the original size, the scale factor should be 1.
        assert_eq!(calculate_scale_factor(2, 2, 3, 3), 1);
        assert_eq!(calculate_scale_factor(3, 3, 9, 9), 3);
    }

    #[test]
    fn test_upscale_image_integer_scale2() {
        // Create a 2x2 input image.
        // (Ensure the width is even since the code processes pixels in pairs.)
        let pixel_a = Color16::new(1, 1, 1, 1);
        let pixel_b = Color16::new(2, 2, 2, 2);
        let pixel_c = Color16::new(3, 3, 3, 3);
        let pixel_d = Color16::new(4, 4, 4, 4);
        let input_data = vec![pixel_a, pixel_b, pixel_c, pixel_d];
        let input_image = RenderImage {
            data: input_data,
            width: 2,
            height: 2,
            stride: 2,
            ..Default::default()
        };

        // Upscale to 4x4. (Scale factor = 2)
        let output_image = upscale_image_integer(&input_image, (4, 4), Falloff::Disabled);
        assert_eq!(output_image.width, 4);
        assert_eq!(output_image.height, 4);

        // Expected layout:
        // - Rows 0-1 come from the first row of the input: [pixel_a, pixel_b],
        //   each expanded to 2 pixels.
        // - Rows 2-3 come from the second row: [pixel_c, pixel_d].
        //
        // That is, row 0 should be: [pixel_a, pixel_a, pixel_b, pixel_b],
        // and similarly for the other rows.
        for row in 0..2 {
            let base = row * 4;
            // Columns 0 and 1: pixel_a.
            for col in 0..2 {
                assert_color_eq(&output_image.data[base + col], &pixel_a);
            }
            // Columns 2 and 3: pixel_b.
            for col in 2..4 {
                assert_color_eq(&output_image.data[base + col], &pixel_b);
            }
        }
        for row in 2..4 {
            let base = row * 4;
            // Columns 0 and 1: pixel_c.
            for col in 0..2 {
                assert_color_eq(&output_image.data[base + col], &pixel_c);
            }
            // Columns 2 and 3: pixel_d.
            for col in 2..4 {
                assert_color_eq(&output_image.data[base + col], &pixel_d);
            }
        }
    }

    #[test]
    fn test_upscale_image_integer_scale3() {
        // Create a 2x2 input image.
        let pixel_a = Color16::new(10, 10, 10, 10);
        let pixel_b = Color16::new(20, 20, 20, 20);
        let pixel_c = Color16::new(30, 30, 30, 30);
        let pixel_d = Color16::new(40, 40, 40, 40);
        let input_data = vec![pixel_a, pixel_b, pixel_c, pixel_d];
        let input_image = RenderImage {
            data: input_data,
            width: 2,
            height: 2,
            border_size: 0,
            ..Default::default()
        };

        // For a target size of 6x6, the calculated scale factor is 3.
        let output_image = upscale_image_integer(&input_image, (6, 6), Falloff::Disabled);
        assert_eq!(output_image.width, 6);
        assert_eq!(output_image.height, 6);

        // Expected layout:
        // - Rows 0-2: expanded from the first row of input [pixel_a, pixel_b]
        //   => columns 0-2: pixel_a, columns 3-5: pixel_b.
        // - Rows 3-5: expanded from the second row of input [pixel_c, pixel_d]
        //   => columns 0-2: pixel_c, columns 3-5: pixel_d.
        for row in 0..3 {
            let base = row * 6;
            for col in 0..3 {
                assert_color_eq(&output_image.data[base + col], &pixel_a);
            }
            for col in 3..6 {
                assert_color_eq(&output_image.data[base + col], &pixel_b);
            }
        }
        for row in 3..6 {
            let base = row * 6;
            for col in 0..3 {
                assert_color_eq(&output_image.data[base + col], &pixel_c);
            }
            for col in 3..6 {
                assert_color_eq(&output_image.data[base + col], &pixel_d);
            }
        }
    }

    #[test]
    fn test_add_border_black() {
        let pixel = Color16::new(1, 2, 3, 4);
        let input_image = RenderImage {
            data: vec![pixel; 4],
            width: 2,
            height: 2,
            stride: 2,
            ..Default::default()
        };

        let border_size = 1;
        let output_image = add_border(&input_image, border_size, BorderType::Black);

        assert_eq!(output_image.width, 2);
        assert_eq!(output_image.height, 2);
        assert_eq!(output_image.border_size, 1);

        let full_width = 4;
        let full_height = 4;

        let black_pixel = Color16::default();
        for y in 0..full_height {
            for x in 0..full_width {
                let index = y * output_image.stride + x;
                if x == 0 || x == 3 || y == 0 || y == 3 {
                    assert_color_eq(&output_image.data[index], &black_pixel);
                } else {
                    assert_color_eq(&output_image.data[index], &pixel);
                }
            }
        }
    }

    #[test]
    fn test_add_border_repeat() {
        let pixel0 = Color16::new(40, 40, 40, 40);
        let pixel1 = Color16::new(50, 50, 50, 50);
        let pixel2 = Color16::new(60, 60, 60, 60);
        let pixel3 = Color16::new(70, 70, 70, 70);
        let input_image = RenderImage {
            data: vec![pixel0, pixel1, pixel2, pixel3].to_vec(),
            width: 2,
            height: 2,
            stride: 2,
            ..Default::default()
        };

        let border_size = 1;
        let output_image = add_border(&input_image, border_size, BorderType::Repeat);

        assert_eq!(output_image.width, 2);
        assert_eq!(output_image.height, 2);
        assert_eq!(output_image.border_size, 1);

        let expected_data = &[
            pixel0, pixel0, pixel1, pixel1,
            pixel0, pixel0, pixel1, pixel1,
            pixel2, pixel2, pixel3, pixel3,
            pixel2, pixel2, pixel3, pixel3,
        ];

        let full_width = 4;
        let full_height = 4;

        for y in 0..full_height {
            for x in 0..full_width {
                let index = y * output_image.stride + x;
                assert_color_eq(&output_image.data[index], &expected_data[index]);
            }
        }
    }}
