use zune_core::{
    bit_depth::BitDepth, colorspace::ColorSpace as ZuneColorSpace,
    options::DecoderOptions as ZuneDecoderOptions,
};

use thiserror::Error as ThisError;

use crate::image::{ImageInfo, LoadOptions, Resize};
use crate::primitives::Color16;
use simd::*;

use job_system::BoxAnySend;
use zune_image::{errors::ImageErrors as ZuneError, image::Image as ZuneImage};

#[derive(ThisError, Debug)]
pub enum ImageErrors {
    #[error("Zune Error")]
    ZuneError(#[from] ZuneError),
    #[error("Generic")]
    Generic(String),
}

#[inline]
fn convert_to_color16(data: &[u8], offset: usize, has_alpha: bool) -> Color16 {
    let r = SRGB_TO_LINEAR_TABLE[data[offset] as usize];
    let g = SRGB_TO_LINEAR_TABLE[data[offset + 1] as usize];
    let b = SRGB_TO_LINEAR_TABLE[data[offset + 2] as usize];
    let a = if has_alpha {
        (data[offset + 3] << 7) as i16
    } else {
        255 << 7
    };

    Color16::new(r, g, b, a)
}

pub(crate) fn decode_zune_internal(
    data: &[u8],
    load_options: LoadOptions,
) -> Result<ImageInfo, ImageErrors> {
    let image = ZuneImage::read(data, ZuneDecoderOptions::default())?;

    let depth = image.depth();
    let color_space = image.colorspace();
    let dimensions = image.dimensions();

    // Only supporting 8 bit depth for now
    if depth != BitDepth::Eight {
        return Err(ImageErrors::Generic(format!(
            "Unsupported depth: {:?}",
            depth
        )));
    }

    // Only deal with one frame for now
    let frames = image.flatten_frames();
    let output_size = frames.iter().map(|f| f.len()).sum::<usize>();
    let mut image_data = vec![0u8; output_size]; // TODO: uninit

    assert_eq!(frames.len(), 1);

    for frame in frames {
        image_data.copy_from_slice(&frame);
    }

    let mut color16_output = Vec::with_capacity(output_size);

    if color_space != ZuneColorSpace::RGB && color_space != ZuneColorSpace::RGBA {
        return Err(ImageErrors::Generic(format!(
            "Unsupported color space: {:?}",
            color_space
        )));
    }

    // Calculate the required range for the entire processing
    let bytes_per_pixel = if color_space == ZuneColorSpace::RGB { 3 } else { 4 };
    let has_alpha = color_space == ZuneColorSpace::RGBA;
    let width = dimensions.0;
    let height = dimensions.1;

    // Calculate the maximum index we'll access
    // This will be the last pixel of the bottom row
    let max_index = if height > 0 && width > 0 {
        ((height - 1) * width + (width - 1)) * bytes_per_pixel + bytes_per_pixel - 1
    } else {
        0
    };

    // Get a slice covering the entire range we'll work with
    let image_data_slice = &image_data[0..=max_index];

    // Process main image data row by row
    for y in 0..height {
        // Process each pixel in the row
        for x in 0..width {
            let offset = (y * width + x) * bytes_per_pixel;
            color16_output.push(convert_to_color16(image_data_slice, offset, has_alpha));
        }

        // Duplicate the last pixel of each row
        //let last_pixel_offset = (y * width + (width - 1)) * bytes_per_pixel;
        //color16_output.push(convert_to_color16(image_data_slice, last_pixel_offset, has_alpha));
    }

    // Duplicate the entire bottom row, including the duplicated edge pixel
    for x in 0..width {
        let last_x = if x == width { width - 1 } else { x };
        let bottom_pixel_offset = ((height - 1) * width + last_x) * bytes_per_pixel;
        color16_output.push(convert_to_color16(image_data_slice, bottom_pixel_offset, has_alpha));
    }

    for x in 0..width {
        let last_x = if x == width { width - 1 } else { x };
        let bottom_pixel_offset = ((height - 1) * width + last_x) * bytes_per_pixel;
        color16_output.push(convert_to_color16(image_data_slice, bottom_pixel_offset, has_alpha));
    }

    if load_options.resize == Resize::IntegerVignette {
        let target_size = (
            load_options.target_size.0 as _,
            load_options.target_size.1 as _,
        );
        let _falloff = match load_options.resize {
            Resize::IntegerVignette => Falloff::Enabled,
            _ => Falloff::Disabled,
        };

        let falloff = Falloff::Enabled;

        let image_info = upscale_image_integer(&color16_output, dimensions, target_size, falloff);

        Ok(image_info)
    } else {
        let image_info = ImageInfo {
            data: vec_to_u8(color16_output),
            width: dimensions.0 as i32,
            height: dimensions.1 as i32,
            //stride: dimensions.0 + 1,
            stride: dimensions.0,
            frame_count: 1,
            frame_delay: 0,
            format: crate::image::Format::Rgba16,
        };

        Ok(image_info)
    }
}

pub(crate) fn decode_zune(data: &[u8], load_options: LoadOptions) -> BoxAnySend {
    Box::new(decode_zune_internal(data, load_options).unwrap()) as BoxAnySend
}

fn apply_falloff(v: i16x8, x_pos: usize, y_pos: usize, width: usize, height: usize) -> i16x8 {
    // TODO: Optimize
    let width_f = width as f32;
    let height_f = height as f32;
    let dx = x_pos as f32 / width_f;
    let dy = (height - y_pos) as f32 / height_f;
    //let dy = y_pos as f32 / height_f;
    //let dy = 1.0;

    // Removed .powf(1.0) since it does nothing.
    let alpha_factor = ((dx * dy) * 32767.0).min(32767.0);
    let background_color = i16x8::new_splat(200);
    i16x8::lerp(background_color, v, i16x8::new_splat(alpha_factor as i16))

    //i16x8::mul_high(v, i16x8::new_splat(alpha_factor as i16))
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
) -> ImageInfo {
    let scale = calculate_scale_factor(size.0, size.1, target_size.0, target_size.1);
    let out_width = size.0 * scale;
    let out_height = size.1 * scale;
    let mut output_data = vec![Color16::default(); out_width * out_height]; // TODO: Arena

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

                        // Store using SIMD-friendly vectorized writes
                        if dx + 1 < scale {
                            adjust_color
                                .store_unaligned(&mut output_data, target_y_offset + current_x);
                            dx += 2;
                        } else {
                            adjust_color.store_unaligned_lower(
                                &mut output_data,
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

    ImageInfo {
        data: vec_to_u8(output_data),
        width: out_width as i32,
        height: out_height as i32,
        stride: out_width,
        frame_count: 1,
        frame_delay: 0,
        format: crate::image::Format::Rgba16,
    }
}

fn vec_to_u8<T>(v: Vec<T>) -> Vec<u8> {
    let element_size = std::mem::size_of::<T>();
    let len = v.len();
    let capacity = v.capacity();
    let ptr = v.as_ptr() as *mut u8;

    // Prevent the original vector from being dropped
    std::mem::forget(v);

    // Create a new Vec<u8> from the raw parts
    unsafe { Vec::from_raw_parts(ptr, len * element_size, capacity * element_size) }
}

static SRGB_TO_LINEAR_TABLE: [i16; 256] = [
    0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 99, 110, 120, 132, 144, 157, 170, 184, 198, 213, 229,
    246, 263, 281, 299, 319, 338, 359, 380, 403, 425, 449, 473, 498, 524, 551, 578, 606, 635, 665,
    695, 727, 759, 792, 825, 860, 895, 931, 968, 1006, 1045, 1085, 1125, 1167, 1209, 1252, 1296,
    1341, 1386, 1433, 1481, 1529, 1578, 1629, 1680, 1732, 1785, 1839, 1894, 1950, 2007, 2065, 2123,
    2183, 2244, 2305, 2368, 2432, 2496, 2562, 2629, 2696, 2765, 2834, 2905, 2977, 3049, 3123, 3198,
    3273, 3350, 3428, 3507, 3587, 3668, 3750, 3833, 3917, 4002, 4088, 4176, 4264, 4354, 4444, 4536,
    4629, 4723, 4818, 4914, 5011, 5109, 5209, 5309, 5411, 5514, 5618, 5723, 5829, 5936, 6045, 6154,
    6265, 6377, 6490, 6604, 6720, 6836, 6954, 7073, 7193, 7315, 7437, 7561, 7686, 7812, 7939, 8067,
    8197, 8328, 8460, 8593, 8728, 8863, 9000, 9139, 9278, 9419, 9560, 9704, 9848, 9994, 10140,
    10288, 10438, 10588, 10740, 10893, 11048, 11204, 11360, 11519, 11678, 11839, 12001, 12164,
    12329, 12495, 12662, 12831, 13000, 13172, 13344, 13518, 13693, 13869, 14047, 14226, 14406,
    14588, 14771, 14955, 15141, 15328, 15516, 15706, 15897, 16089, 16283, 16478, 16675, 16872,
    17071, 17272, 17474, 17677, 17882, 18088, 18295, 18504, 18714, 18926, 19139, 19353, 19569,
    19786, 20004, 20224, 20445, 20668, 20892, 21118, 21345, 21573, 21803, 22034, 22267, 22501,
    22736, 22973, 23211, 23451, 23692, 23935, 24179, 24425, 24672, 24920, 25170, 25421, 25674,
    25928, 26184, 26441, 26700, 26960, 27222, 27485, 27749, 28016, 28283, 28552, 28823, 29095,
    29368, 29643, 29920, 30197, 30477, 30758, 31040, 31324, 31610, 31897, 32185, 32475, 32767,
];

/*

Above table generated using this

const LINEAR_BIT_COUNT: i32 = 15;

const fn srgb_to_linear(x: f32) -> f32 {
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

const fn build_srgb_to_linear_table() -> [i16; 256] {
    let mut table = [0; 256];
    let mut i = 0;
    while i < 256 {
        let srgb = i as f32 / 255.0;
        let linear = srgb_to_linear(srgb); // Ensure srgb_to_linear is also const
        table[i] = (linear * ((1 << LINEAR_BIT_COUNT) - 1) as f32).round() as i16;
        i += 1;
    }
    table
}

 */
