use simd::*;

pub struct ScaledImage {
    pub width: u16,
    pub height: u16,
    pub data: Vec<i16>,
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
/// An `i16x8` vector with sampled and interpolated texture values.
#[inline(always)]
fn sample_aligned_texture(
    texture: *const i16,
    texture_width: usize,
    u_fraction: i16x8,
    v_fraction: i16x8,
    offset: usize,
) -> i16x8 {
    let rgba_rgba_0 = i16x8::load_unaligned_ptr(unsafe { texture.add(offset) });
    let rgba_rgba_1 =
        i16x8::load_unaligned_ptr(unsafe { texture.add((texture_width * 4) + offset) });
    let t0_t1 = i16x8::lerp(rgba_rgba_0, rgba_rgba_1, v_fraction);
    let t = t0_t1.rotate_4();
    i16x8::lerp(t0_t1, t, u_fraction)
}


pub fn scale_image(input_image: &[i16], width: u32, height: u32, out_width: u32, out_height: u32) -> ScaledImage {
    let border_size_x = 4;
    let border_size_y = 1;
    let final_width = out_width + 2 * border_size_x;
    let final_height = out_height + 2 * border_size_y;
    let mut output = vec![0i16; ((final_width * final_height) * 4) as usize]; // Black background (0)

    let x_ratio = ((width as u32) << 15) / out_width;
    let y_ratio = ((height as u32) << 15) / out_height;
    let start_offset = (border_size_y as usize * final_width as usize + border_size_x as usize) * 4;

    let output_offset = &mut output[start_offset..];
    let mut output_ptr = output_offset.as_mut_ptr();
    
    let mut y_value = 0u32;

    unsafe {
        for _y in 0..out_height {
            let y_pos = y_value >> 15; 
            let y_fractional = y_value & 0xFFFF;
            let sy_f = i16x8::new_splat(y_fractional as i16);

            let mut x_value = 0u32;

            for _x in 0..out_width {
                let x_pos = (x_value >> 15) as usize; 
                let x_fractional = x_value & 0xFFFF;

                let sx_f = i16x8::new_splat(x_fractional as i16);

                let sample = sample_aligned_texture(
                    input_image.as_ptr(),
                    width as usize,
                    sx_f,
                    sy_f,
                    ((y_pos as usize) * (width as usize) + x_pos) * 4,
                );

                sample.store_unaligned_ptr_lower(output_ptr);

                output_ptr = output_ptr.add(4);

                x_value += x_ratio;
            }

            output_ptr = output_ptr.add(((final_width * 4) as usize) - ((out_width + 6) * 4) as usize);

            y_value += y_ratio;
        }
    }

    ScaledImage {
        width: out_width as u16 + 2,
        height: out_height as u16 + 2,
        data: output,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upscale() {
        let input_image = vec![32767; 16 * 16 * 4];
        let scaled = scale_image(&input_image, 16, 16, 32, 32);
        assert_eq!(scaled.width, 34);
        assert_eq!(scaled.height, 34);
        assert_eq!(scaled.data.len(), (40 * 34 * 4) as usize);

        for y in 0..scaled.height as usize {
            for x in 0..scaled.width as usize {
                let idx = (y * scaled.width as usize + x) * 4;
                let v = scaled.data[idx];
                assert!(v == 32767 || v == 0, "Pixel at (y={}, x={}) has unexpected value: {}", y, x, v);
            }
        } 
    }

    #[test]
    fn test_downscale() {
        let input_image = vec![32767; 64 * 64 * 4];
        let scaled = scale_image(&input_image, 64, 64, 16, 16);
        assert_eq!(scaled.width, 18);
        assert_eq!(scaled.height, 18);
        assert_eq!(scaled.data.len(), (24 * 18 * 4) as usize);
        assert!(scaled.data.iter().all(|&v| v == 32767 || v == 0));
    }

 #[test]
    fn test_black_border() {
        let input_image = vec![32767; 16 * 16 * 4];
        let scaled = scale_image(&input_image, 16, 16, 16, 16);
        assert_eq!(scaled.width, 18);
        assert_eq!(scaled.height, 18);
        let border_x = 4;
        let border_y = 1;
        let final_width = 16 + 2 * border_x;
        let final_height = 16 + 2 * border_y;
        assert_eq!(scaled.data.len(), (final_width * final_height * 4) as usize);
        for y in 0..final_height {
            for x in 0..final_width {
                let idx = (y * final_width + x) * 4;
                if x < border_x || x >= (16 + border_x) && y < border_y || y >= (16 + border_y) {
                    assert_eq!(scaled.data[idx], 0);
                    assert_eq!(scaled.data[idx + 1], 0);
                    assert_eq!(scaled.data[idx + 2], 0);
                    assert_eq!(scaled.data[idx + 3], 0);
                }
            }
        }
    }
}
