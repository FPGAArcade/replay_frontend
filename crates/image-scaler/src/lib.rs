use simd::*;
use color16::Color16;

pub struct ScaledImage {
    pub width: u16,
    pub height: u16,
    pub data: Vec<Color16>,
}

/// Samples aligned texture data with bilinear interpolation in vectorized form.
///
/// # Parameters
/// - `texture`: Pointer to the texture data.
/// - `texture_width`: Width of the texture in pixels.
/// - `u_fraction`, `v_fraction`: Fractions for bilinear interpolation in U and V directions.
/// - `offset`: Starting offset in the texture data.
///
/// # Returns
/// An `i16x8` vector with sampled and interpolated texture values in the lower half.
#[inline(always)]
fn sample_aligned_texture(
    texture: *const Color16,
    texture_width: usize,
    u_fraction: i16x8,
    v_fraction: i16x8,
    offset: usize,
) -> i16x8 {
    let rgba_rgba_0 = i16x8::load_unaligned_ptr(unsafe { texture.add(offset) } as *const i16);
    let rgba_rgba_1 =
        i16x8::load_unaligned_ptr(unsafe { texture.add((texture_width) + offset) } as *const i16);
    let t0_t1 = i16x8::lerp(rgba_rgba_0, rgba_rgba_1, v_fraction);
    let t = t0_t1.rotate_4();
    i16x8::lerp(t0_t1, t, u_fraction)
}

const FIXED_POINT_SHIFT: u32 = 15;
const FIXED_POINT_MASK: u32 = (1 << FIXED_POINT_SHIFT) - 1;

pub fn scale_image(
    input_image: &[Color16],
    width: usize,
    height: usize,
    out_width: usize,
    out_height: usize,
) -> ScaledImage {
    let border_size_x = 1usize;
    let border_size_y = 1usize;
    let final_width = (out_width + 2 * border_size_x) as usize;
    let final_height = (out_height + 2 * border_size_y) as usize;
    let mut output = vec![Color16::default(); final_width * final_height];

    let x_ratio = ((width as u32) << FIXED_POINT_SHIFT) / (out_width as u32);
    let y_ratio = ((height as u32) << FIXED_POINT_SHIFT) / (out_height as u32);
    let start_offset = (border_size_y * final_width) + border_size_x;

    let output_offset = &mut output[start_offset..];
    let mut output_ptr = output_offset.as_mut_ptr();

    let width = width as usize;
    let mut y_value = 0u32;

    unsafe {
        for _y in 0..out_height {
            let y_pos = (y_value >> FIXED_POINT_SHIFT) as usize;
            let y_fractional = y_value & FIXED_POINT_MASK;
            let sy_f = i16x8::new_splat(y_fractional as i16);

            let mut x_value = 0u32;

            for _x in 0..out_width {
                let x_pos = (x_value >> FIXED_POINT_SHIFT) as usize;
                let x_fractional = x_value & FIXED_POINT_MASK;

                let sx_f = i16x8::new_splat(x_fractional as i16);

                let sample = sample_aligned_texture(
                    input_image.as_ptr(),
                    width as usize,
                    sx_f,
                    sy_f,
                    (y_pos * width) + x_pos,
                );

                sample.store_unaligned_ptr_lower(output_ptr as *mut i16);

                output_ptr = output_ptr.add(1);

                x_value += x_ratio;
            }

            output_ptr = output_ptr.add(final_width - out_width);

            y_value += y_ratio;
        }
    }

    ScaledImage {
        width: final_width as _, 
        height: final_height as _,
        data: output,
    }
}

fn calculate_scale_factor(original_width: usize, original_height: usize, target_width: usize, target_height: usize) -> usize {
    let max_scale_x = target_width / original_width;
    let max_scale_y = target_height / original_height;
    max_scale_x.min(max_scale_y).max(1) // Ensure at least 1x scaling
}

pub fn apply_alpha_falloff(pixel: Color16, width: usize, height: usize, x: usize, y: usize) -> Color16 {
    let dx = x as f32 / width as f32;  // Distance from right
    let dy = (height - y) as f32 / height as f32; // Distance from bottom

    // Apply a quadratic falloff (adjust power for stronger effect)
    let alpha_factor = (dx * dy).powf(1.0); // Non-linear fade

    Color16 {
        r: (pixel.r as f32 * alpha_factor) as i16,
        g: (pixel.g as f32 * alpha_factor) as i16,
        b: (pixel.b as f32 * alpha_factor) as i16,
        a: (pixel.a as f32 * alpha_factor) as i16,
    }
}

//pub fn upscale_image_integer<const SHADE: bool>(
pub fn upscale_image_integer(
    input_image: &[Color16],
    width: usize,
    height: usize,
    target_width: usize,
    target_height: usize)
    -> ScaledImage
{
    let scale = calculate_scale_factor(width, height, target_width, target_height);
    let out_width = width * scale;
    let out_height = height * scale;

    // TODO: Hardcoded-different scaling steps to avoid branching

    let mut output = vec![Color16::default(); out_width * out_height];

    // Perform the zoom operation only for the valid source region
    for y in 0..height {
        for x in 0..width {
            let color = input_image[y * width + x];
            let start_y = y * scale;
            let start_x = x * scale;

            // Write the zoomed block directly without further checks
            for dy in 0..scale {
                let target_y = start_y + dy;

                let target_row = &mut output[target_y * out_width..(target_y + 1) * out_width];
                for dx in 0..scale {
                    let shaded_color = apply_alpha_falloff(color, width, height, x, y);
                    target_row[start_x + dx] = shaded_color;
                }
            }
        }
    }

    ScaledImage {
        width: out_width as _,
        height: out_height as _,
        data: output,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

     #[test]
    fn test_large_image_scale() {
        let width = 16;
        let height = 16;
        let out_width = 8;
        let out_height = 8;
        let input_image: Vec<Color16> = (0..(width * height))
            .map(|i| Color16::new(i as i16, (i + 1) as i16, (i + 2) as i16, (i + 3) as i16))
            .collect();
        
        let result = scale_image(&input_image, width, height, out_width, out_height);
        
        assert_eq!(result.width, (out_width as u16) + 2);
        assert_eq!(result.height, (out_height as u16) + 2);
        assert_eq!(result.data.len(), (out_width + 2) * (out_height + 2));
        
        for x in 0..result.width {
            assert_eq!(result.data[x as usize], Color16::new(0, 0, 0, 0)); // Top border
            assert_eq!(result.data[((result.height - 1) * result.width + x) as usize], Color16::new(0, 0, 0, 0)); // Bottom border
        }
        for y in 0..result.height {
            assert_eq!(result.data[(y * result.width) as usize], Color16::new(0, 0, 0, 0)); // Left border
            assert_eq!(result.data[(y * result.width + (result.width - 1)) as usize], Color16::new(0, 0, 0, 0)); // Right border
        }
    }

    #[test]
    fn test_exact_fit() {
        assert_eq!(calculate_scale_factor(500, 500, 1000, 1000), 2);
    }

    #[test]
    fn test_limited_by_width() {
        assert_eq!(calculate_scale_factor(400, 300, 1200, 1500), 3);
    }

    #[test]
    fn test_limited_by_height() {
        assert_eq!(calculate_scale_factor(600, 400, 2500, 1200), 3);
    }

    #[test]
    fn test_no_scaling_needed() {
        assert_eq!(calculate_scale_factor(800, 600, 800, 600), 1);
    }

    #[test]
    fn test_downscaling_not_allowed() {
        assert_eq!(calculate_scale_factor(1000, 800, 500, 400), 1);
    }

    #[test]
    fn test_large_scale_factor() {
        assert_eq!(calculate_scale_factor(10, 10, 100, 150), 10);
    }

    #[test]
    fn test_irregular_aspect_ratio() {
        assert_eq!(calculate_scale_factor(300, 500, 900, 1000), 2);
    }

    #[test]
    fn test_amiga_to_1080p() {
        assert_eq!(calculate_scale_factor(360, 280, 1920, 1080), 3);
    }

}
