use crate::simd::*;
use crate::TileInfo;

const TEXTURE_MODE_NONE: usize = 0;
const TEXTURE_MODE_ALIGNED: usize = 1;
const PIXEL_COUNT_1: usize = 1;
const PIXEL_COUNT_2: usize = 2;
const PIXEL_COUNT_3: usize = 3;
const PIXEL_COUNT_4: usize = 4;

const COLOR_MODE_NONE: usize = 0;
const COLOR_MODE_SOLID: usize = 1;
const COLOR_MODE_LERP: usize = 2;

const BLEND_MODE_NONE: usize = 0;
const BLEND_MODE_BG_COLOR: usize = 1;
const BLEND_MODE_TEXTURE_COLOR: usize = 2;
const BLEND_MODE_BG_TEXTURE_COLOR: usize = 3;

const ROUND_MODE_NONE: usize = 0;
const ROUND_MODE_ENABLED: usize = 1;

#[derive(Copy, Clone)]
enum Corner {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

const CORNER_OFFSETS: [(f32, f32); 4] = [
    (1.0, 1.0), // TopLeft: No shift
    (0.0, 1.0), // TopRight: Shift down
    (1.0, 0.0), // BottemLeft: Shift right
    (0.0, 0.0), // BottomRight: Shift right and down
];

#[derive(Copy, Clone)]
pub enum BlendMode {
    None = BLEND_MODE_NONE as _,
    WithBackground = BLEND_MODE_BG_COLOR as _,
    WithTexture = BLEND_MODE_TEXTURE_COLOR as _,
    WithBackgroundAndTexture = BLEND_MODE_BG_TEXTURE_COLOR as _,
}

pub(crate) struct Raster {
    pub(crate) scissor_rect: i32x4,
}

/// Calculates the blending factor for rounded corners in vectorized form.
///
/// # Parameters
/// - `center_y2`: Squared y-coordinates from circle centers.
/// - `current_x`: X-coordinates of current points.
/// - `circle_center_x`: X-coordinates of circle centers.
/// - `border_radius_v`: Vertical border radii of circles.
///
/// # Returns
/// A vector of 15-bit integers representing blending factors for anti-aliasing,
/// scaled to fit within 0 to 32767.
#[inline(always)]
fn calculate_rounding_blend(
    circle_y2: f32x4,
    current_x: f32x4,
    circle_center_x: f32x4,
    border_radius_v: f32x4,
) -> i16x8 {
    let t0 = current_x - circle_center_x;
    let circle_x2 = t0 * t0;
    let dist = (circle_x2 + circle_y2).sqrt();
    let dist_to_edge = dist - border_radius_v;

    let dist_to_edge =
        f32x4::new_splat(1.0) - dist_to_edge.clamp(f32x4::new_splat(0.0), f32x4::new_splat(1.0));

    (dist_to_edge * f32x4::new_splat(32767.0))
        .as_i32x4()
        .as_i16x8()
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

fn blend_color(source: i16x8, dest: i16x8) -> i16x8 {
    let one_minus_alpha = i16x8::new_splat(0x7fff) - source.shuffle_3333_7777();
    i16x8::lerp(source, dest, one_minus_alpha)
}

/// Adjusts the color values based on the alpha value using pre-multiplied alpha.
///
/// This function takes an `i16x8` vector representing color values and adjusts
/// the color components based on the alpha value. The resulting vector will
/// have the same alpha value while the color components are modified.
///
/// # Arguments
///
/// * `color` - An `i16x8` vector representing the color values.
///
/// # Returns
///
/// An `i16x8` vector with adjusted color values based on the alpha value.
///
/// # Example
///
/// ```
/// let color = i16x8::new(255, 128, 64, 32, 255, 128, 64, 32);
/// let result = premultiply_alpha(color);
/// ```
#[inline(always)]
fn premultiply_alpha(color: i16x8) -> i16x8 {
    let alpha = color.shuffle_333_0x7fff_777_0x7fff();
    i16x8::mul_high(color, alpha)
}

#[inline(always)]
fn interpolate_color(left_colors: i16x8, color_diff: i16x8, step: i16x8) -> i16x8 {
    let color = i16x8::lerp_diff(left_colors, color_diff, step);
    premultiply_alpha(color)
}

#[inline(always)]
fn multi_sample_aligned_texture<const COUNT: usize>(
    texture: *const i16,
    width: usize,
    u: i16x8,
    v: i16x8,
) -> (i16x8, i16x8) {
    let zero = i16x8::new_splat(0);

    if COUNT == PIXEL_COUNT_1 {
        let t0 = sample_aligned_texture(texture, width, u, v, 0);
        (t0, zero)
    } else if COUNT == PIXEL_COUNT_2 {
        let t0 = sample_aligned_texture(texture, width, u, v, 0);
        let t1 = sample_aligned_texture(texture, width, u, v, 4);
        (i16x8::merge(t0, t1), zero)
    } else if COUNT == PIXEL_COUNT_3 {
        let t0 = sample_aligned_texture(texture, width, u, v, 0);
        let t1 = sample_aligned_texture(texture, width, u, v, 4);
        let t2 = sample_aligned_texture(texture, width, u, v, 8);
        (i16x8::merge(t0, t1), t2)
    } else if COUNT == PIXEL_COUNT_4 {
        let t0 = sample_aligned_texture(texture, width, u, v, 0);
        let t1 = sample_aligned_texture(texture, width, u, v, 4);
        let t2 = sample_aligned_texture(texture, width, u, v, 8);
        let t3 = sample_aligned_texture(texture, width, u, v, 12);
        (i16x8::merge(t0, t1), i16x8::merge(t2, t3))
    } else {
        unimplemented!()
    }
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn process_pixels<
    const COUNT: usize,
    const COLOR_MODE: usize,
    const TEXTURE_MODE: usize,
    const BLEND_MODE: usize,
    const ROUND_MODE: usize,
>(
    output: *mut i16,
    fixed_color: i16x8,
    texture: *const i16,
    texture_width: usize,
    fixed_u_fraction: i16x8,
    fixed_v_fraction: i16x8,
    color_diff: i16x8,
    left_colors: i16x8,
    x_step_current: i16x8,
    xi_step: i16x8,
    c_blend: i16x8,
) {
    let mut color_0 = fixed_color;
    let mut color_1 = fixed_color;

    let mut tex_0 = i16x8::new_splat(0);
    let mut tex_1 = i16x8::new_splat(0);

    if TEXTURE_MODE == TEXTURE_MODE_ALIGNED {
        (tex_0, tex_1) = multi_sample_aligned_texture::<COUNT>(
            texture,
            texture_width,
            fixed_u_fraction,
            fixed_v_fraction,
        );
    }

    if COLOR_MODE == COLOR_MODE_LERP {
        color_0 = interpolate_color(left_colors, color_diff, x_step_current);
        color_1 = interpolate_color(left_colors, color_diff, x_step_current + xi_step);
    } else if TEXTURE_MODE == TEXTURE_MODE_ALIGNED {
        color_0 = tex_0;
        color_1 = tex_1;
    }

    /*
    if BLEND_MODE == BLEND_MODE_TEXTURE_COLOR_BG {
        // TODO: Blend between texture and color
    }
    */

    // If we have rounded we need to adjust the color based on the distance to the circle center
    if ROUND_MODE == ROUND_MODE_ENABLED {
        if COUNT >= PIXEL_COUNT_3 {
            // distance to the circle center. So we need to splat distance for each radius
            // calculated to get the correct blending value.
            color_0 = i16x8::mul_high(color_0, c_blend.shuffle_0000_2222());
            color_1 = i16x8::mul_high(color_1, c_blend.shuffle_4444_6666());
        } else {
            // Only one or two pixels so we only need one shuffle/mul
            color_0 = i16x8::mul_high(color_0, c_blend.shuffle_0000_2222());
        }
    }

    // Blend between color and the background
    if BLEND_MODE == BLEND_MODE_BG_COLOR {
        if COUNT >= PIXEL_COUNT_3 {
            let bg_color_0 = i16x8::load_unaligned_ptr(output);
            let bg_color_1 = i16x8::load_unaligned_ptr(unsafe { output.add(8) });
            // Blend between the two colors
            color_0 = blend_color(color_0, bg_color_0);
            color_1 = blend_color(color_1, bg_color_1);
        } else {
            let bg_color_0 = i16x8::load_unaligned_ptr(output);
            color_0 = blend_color(color_0, bg_color_0);
        }
    }

    match COUNT {
        PIXEL_COUNT_1 => color_0.store_unaligned_ptr_lower(output),
        PIXEL_COUNT_2 => color_0.store_unaligned_ptr(output),
        PIXEL_COUNT_3 => {
            color_0.store_unaligned_ptr(output);
            color_1.store_unaligned_ptr_lower(unsafe { output.add(8) });
        }
        PIXEL_COUNT_4 => {
            color_0.store_unaligned_ptr(output);
            color_1.store_unaligned_ptr(unsafe { output.add(8) });
        }
        _ => unimplemented!(),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_internal<
    const COLOR_MODE: usize,
    const TEXTURE_MODE: usize,
    const ROUND_MODE: usize,
    const BLEND_MODE: usize,
>(
    output: &mut [i16],
    scissor_rect: i32x4,
    texture_data: *const i16,
    tile_info: &TileInfo,
    uv_data: &[f32],
    texture_sizes: &[i32],
    coords: &[f32],
    border_radius: f32,
    radius_direction: usize,
    top_colors: i16x8,
    bottom_colors: i16x8,
) {
    let x0y0x1y1_adjust =
        (f32x4::load_unaligned(coords) - tile_info.offsets) + f32x4::new_splat(0.5);
    let x0y0x1y1 = x0y0x1y1_adjust.floor();
    let x0y0x1y1_int = x0y0x1y1.as_i32x4();

    // Make sure we intersect with the scissor rect otherwise skip rendering
    if !i32x4::test_intersect(scissor_rect, x0y0x1y1_int) {
        return;
    }

    // Used for stepping for edges with radius
    let mut rounding_y_step = f32x4::new_splat(0.0);
    let mut rounding_x_step = f32x4::new_splat(0.0);
    let mut rounding_y_current = f32x4::new_splat(0.0);
    let mut rounding_x_current = f32x4::new_splat(0.0);
    let mut border_radius_v = f32x4::new_splat(0.0);
    let mut circle_center_x = f32x4::new_splat(0.0);
    let mut circle_center_y = f32x4::new_splat(0.0);

    let mut xi_start = i16x8::new_splat(0);
    let mut yi_start = i16x8::new_splat(0);

    let mut fixed_u_fraction = i16x8::new_splat(0);
    let mut fixed_v_fraction = i16x8::new_splat(0);

    let mut xi_step = i16x8::new_splat(0);
    let mut yi_step = i16x8::new_splat(0);

    let mut texture_ptr = texture_data; //.as_ptr();
    let mut texture_width = 0;

    // Calculate the difference between the scissor rect and the current rect
    // if diff is > 0 we return back a positive value to use for clipping
    let clip_diff = (x0y0x1y1_int - scissor_rect).min(i32x4::new_splat(0)).abs();

    if COLOR_MODE == COLOR_MODE_LERP {
        let x0y0x0y0 = x0y0x1y1.shuffle_0101();
        let x1y1x1y1 = x0y0x1y1.shuffle_2323();

        let xy_diff = x1y1x1y1 - x0y0x0y0;
        let xy_step = f32x4::new_splat(32767.0) / xy_diff;

        xi_step = xy_step.as_i32x4().as_i16x8().splat_0000_0000();
        yi_step = xy_step.as_i32x4().as_i16x8().splat_2222_2222();

        // The way we step across x is that we do two pixels at a time. Because of this we need
        // to adjust the stepping value to be times two and then adjust the starting value so that
        // is like this:
        // start: 0,1
        // step:  2,2

        let clip_x0 = clip_diff.as_i16x8().splat_0000_0000();
        let clip_y0 = clip_diff.as_i16x8().splat_2222_2222();

        xi_start = xi_step * clip_x0;
        yi_start = yi_step * clip_y0;

        xi_start += xi_step * i16x8::new(0, 0, 0, 0, 1, 1, 1, 1);
        xi_step = xi_step * i16x8::new_splat(2);
    }

    if TEXTURE_MODE == TEXTURE_MODE_ALIGNED {
        // For aligned data we assume that UVs are in texture space range and not normalized
        let uv = f32x4::load_unaligned(uv_data);
        let uv_i = uv.as_i32x4();

        let uv_fraction = (x0y0x1y1_adjust - x0y0x1y1) * f32x4::new_splat(0x7fff as f32);
        let uv_fraction = i16x8::new_splat(0x7fff) - uv_fraction.as_i32x4().as_i16x8();

        fixed_u_fraction = uv_fraction.splat_0000_0000();
        fixed_v_fraction = uv_fraction.splat_2222_2222();

        texture_width = texture_sizes[0] as usize;

        let clip_x = clip_diff.extract::<0>() as usize;
        let clip_y = clip_diff.extract::<1>() as usize;

        // Get the starting point in the texture data and add the clip diff to get correct starting
        // position of the texture

        let u = uv_i.extract::<0>() as usize + clip_x;
        let v = uv_i.extract::<1>() as usize + clip_y;

        texture_ptr = unsafe { texture_ptr.add((v * texture_width + u) * 4) };
    }

    // If we have rounded edges we need to adjust the start and end values
    if ROUND_MODE == ROUND_MODE_ENABLED {
        let center_adjust = CORNER_OFFSETS[radius_direction & 3];

        // TODO: Get the corret corner direction
        rounding_y_step = f32x4::new_splat(1.0);
        rounding_x_step = f32x4::new_splat(4.0);
        rounding_y_current = f32x4::new_splat(clip_diff.extract::<1>() as f32);

        let uv_fraction = x0y0x1y1_adjust - x0y0x1y1;

        // TODO: Optimize 
        border_radius_v = f32x4::new_splat(border_radius);
        circle_center_x = f32x4::new_splat(uv_fraction.extract::<0>() + (border_radius * center_adjust.0));
        circle_center_y = f32x4::new_splat(uv_fraction.extract::<1>() + (border_radius * center_adjust.1));
    }

    let min_box = x0y0x1y1_int.min(scissor_rect);
    let max_box = x0y0x1y1_int.max(scissor_rect);

    let x0 = max_box.extract::<0>();
    let y0 = max_box.extract::<1>();
    let x1 = min_box.extract::<2>();
    let y1 = min_box.extract::<3>();

    let ylen = y1 - y0;
    let xlen = x1 - x0;

    let tile_width = tile_info.width as usize;
    let output = &mut output[((y0 as usize * tile_width + x0 as usize) * 4)..];
    let mut output_ptr = output.as_mut_ptr();

    let current_color = top_colors;
    let mut color_diff = i16x8::new_splat(0);
    let mut color_top_bottom_diff = i16x8::new_splat(0);
    let mut left_colors = i16x8::new_splat(0);

    if COLOR_MODE == COLOR_MODE_LERP {
        color_top_bottom_diff = bottom_colors - top_colors;
    }

    let mut tile_line_ptr = output_ptr;
    let mut texture_line_ptr = texture_ptr;
    let mut circle_y2 = f32x4::new_splat(0.0);
    let mut circle_distance = i16x8::new_splat(0);

    for _y in 0..ylen {
        // as y2 for the circle is constant in the inner loop we can calculate it here
        if ROUND_MODE == ROUND_MODE_ENABLED {
            let t0 = rounding_y_current - circle_center_y;
            circle_y2 = t0 * t0;
            let x_start = clip_diff.extract::<0>() as f32;
            rounding_x_current = f32x4::new(x_start, x_start + 1.0, x_start + 2.0, x_start + 3.0);
        }

        if COLOR_MODE == COLOR_MODE_LERP {
            let left_right_colors = i16x8::lerp_diff(top_colors, color_top_bottom_diff, yi_start);
            let right_colors = left_right_colors.shuffle_4567_4567();
            left_colors = left_right_colors.shuffle_0123_0123();
            color_diff = right_colors - left_colors;
        }

        let mut x_step_current = xi_start;

        for _x in 0..(xlen >> 2) {
            // Calculate the distance to the circle center
            if ROUND_MODE == ROUND_MODE_ENABLED {
                circle_distance = calculate_rounding_blend(
                    circle_y2,
                    rounding_x_current,
                    circle_center_x,
                    border_radius_v,
                );
            }

            process_pixels::<PIXEL_COUNT_4, COLOR_MODE, TEXTURE_MODE, BLEND_MODE, ROUND_MODE>(
                output_ptr,
                current_color,
                texture_ptr,
                texture_width,
                fixed_u_fraction,
                fixed_v_fraction,
                color_diff,
                left_colors,
                x_step_current,
                xi_step,
                circle_distance,
            );

            output_ptr = unsafe { output_ptr.add(16) };

            if TEXTURE_MODE == TEXTURE_MODE_ALIGNED {
                texture_ptr = unsafe { texture_ptr.add(16) };
            }

            if COLOR_MODE == COLOR_MODE_LERP {
                x_step_current += xi_step * i16x8::new_splat(2);
            }

            if ROUND_MODE == ROUND_MODE_ENABLED {
                rounding_x_current += rounding_x_step;
            }
        }

        // Calculate the distance to the circle center
        if ROUND_MODE == ROUND_MODE_ENABLED && (xlen & 3) != 0 {
            circle_distance = calculate_rounding_blend(
                circle_y2,
                rounding_x_current,
                circle_center_x,
                border_radius_v,
            );
        }

        // Process the remaining pixels
        match xlen & 3 {
            1 => {
                process_pixels::<PIXEL_COUNT_1, COLOR_MODE, TEXTURE_MODE, BLEND_MODE, ROUND_MODE>(
                    output_ptr,
                    current_color,
                    texture_ptr,
                    texture_width,
                    fixed_u_fraction,
                    fixed_v_fraction,
                    color_diff,
                    left_colors,
                    x_step_current,
                    xi_step,
                    circle_distance,
                );
            }
            2 => {
                process_pixels::<PIXEL_COUNT_2, COLOR_MODE, TEXTURE_MODE, BLEND_MODE, ROUND_MODE>(
                    output_ptr,
                    current_color,
                    texture_ptr,
                    texture_width,
                    fixed_u_fraction,
                    fixed_v_fraction,
                    color_diff,
                    left_colors,
                    x_step_current,
                    xi_step,
                    circle_distance,
                );
            }
            3 => {
                process_pixels::<PIXEL_COUNT_3, COLOR_MODE, TEXTURE_MODE, BLEND_MODE, ROUND_MODE>(
                    output_ptr,
                    current_color,
                    texture_ptr,
                    texture_width,
                    fixed_u_fraction,
                    fixed_v_fraction,
                    color_diff,
                    left_colors,
                    x_step_current,
                    xi_step,
                    circle_distance,
                );
            }
            _ => {}
        }

        tile_line_ptr = unsafe { tile_line_ptr.add(tile_width * 4) };
        output_ptr = tile_line_ptr;

        if TEXTURE_MODE == TEXTURE_MODE_ALIGNED {
            texture_line_ptr = unsafe { texture_line_ptr.add(texture_width * 4) };
            texture_ptr = texture_line_ptr;
        }

        if COLOR_MODE == COLOR_MODE_LERP {
            yi_start += yi_step;
        }

        if ROUND_MODE == ROUND_MODE_ENABLED {
            rounding_y_current += rounding_y_step;
        }
    }
}

impl Raster {
    pub(crate) fn new() -> Self {
        Self {
            scissor_rect: i32x4::new_splat(0),
        }
    }

    #[inline(never)]
    pub fn render_aligned_texture(&self,
        output: &mut [i16],
        tile_info: &TileInfo,
        coords: &[f32],
        texture_data: *const i16,
        uv_data: &[f32],
        texture_sizes: &[i32],
    ) {
        render_internal::<COLOR_MODE_NONE, TEXTURE_MODE_ALIGNED, ROUND_MODE_NONE, BLEND_MODE_NONE>(
            output,
            self.scissor_rect,
            texture_data,
            tile_info,
            uv_data,
            texture_sizes,
            coords,
            0.0,
            0,
            i16x8::new_splat(0),
            i16x8::new_splat(0),
        );
    }

    #[inline(never)]
    pub fn render_solid_quad(
        &self,
        output: &mut [i16],
        tile_info: &TileInfo,
        coords: &[f32],
        color: i16x8,
        blend_mode: BlendMode,
    ) {
        let uv_data = [0.0];
        let texture_sizes = [0];

        match blend_mode {
            BlendMode::None => {
                render_internal::<
                    COLOR_MODE_SOLID,
                    TEXTURE_MODE_NONE,
                    ROUND_MODE_NONE,
                    BLEND_MODE_NONE,
                >(
                    output,
                    self.scissor_rect,
                    std::ptr::null(),
                    tile_info,
                    &uv_data,
                    &texture_sizes,
                    coords,
                    0.0,
                    0,
                    color,
                    color,
                );
            }
            BlendMode::WithBackground => {
                render_internal::<
                    COLOR_MODE_SOLID,
                    TEXTURE_MODE_NONE,
                    ROUND_MODE_NONE,
                    BLEND_MODE_BG_COLOR,
                >(
                    output,
                    self.scissor_rect,
                    std::ptr::null(),
                    tile_info,
                    &uv_data,
                    &texture_sizes,
                    coords,
                    0.0,
                    0,
                    color,
                    color,
                );
            }
            _ => unimplemented!(),
        }
    }

    #[inline(never)]
    pub fn render_gradient_quad(
        &self,
        output: &mut [i16],
        tile_info: &TileInfo,
        coords: &[f32],
        color_top: i16x8,
        color_bottom: i16x8,
        blend_mode: BlendMode,
    ) {
        let uv_data = [0.0];
        let texture_sizes = [0];

        match blend_mode {
            BlendMode::None => {
                render_internal::<
                    COLOR_MODE_LERP,
                    TEXTURE_MODE_NONE,
                    ROUND_MODE_NONE,
                    BLEND_MODE_NONE,
                >(
                    output,
                    self.scissor_rect,
                    std::ptr::null(),
                    tile_info,
                    &uv_data,
                    &texture_sizes,
                    coords,
                    0.0,
                    0,
                    color_top,
                    color_bottom,
                );
            }
            BlendMode::WithBackground => {
                render_internal::<
                    COLOR_MODE_LERP,
                    TEXTURE_MODE_NONE,
                    ROUND_MODE_NONE,
                    BLEND_MODE_BG_COLOR,
                >(
                    output,
                    self.scissor_rect,
                    std::ptr::null(),
                    tile_info,
                    &uv_data,
                    &texture_sizes,
                    coords,
                    0.0,
                    0,
                    color_top,
                    color_bottom,
                );
            }
            _ => unimplemented!(),
        }
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn render_solid_rounded_corner(
        &self,
        output: &mut [i16],
        tile_info: &TileInfo,
        coords: &[f32],
        color: i16x8,
        radius: f32,
        _blend_mode: BlendMode,
        corner: Corner,
    ) {
        let uv_data = [0.0];
        let texture_sizes = [0];

        render_internal::<COLOR_MODE_NONE, TEXTURE_MODE_NONE, ROUND_MODE_ENABLED, BLEND_MODE_NONE>(
            output,
            self.scissor_rect,
            std::ptr::null(),
            tile_info,
            &uv_data,
            &texture_sizes,
            coords,
            radius,
            corner as usize,
            color,
            color,
        );
    }

    fn get_corner_coords(corner: Corner, coords: &[f32], radius: f32) -> [f32; 4] {
        let corner_size = radius.ceil();
        let corner_exp = corner_size + 4.0;

        match corner {
            Corner::TopLeft => [
                coords[0],
                coords[1],
                coords[0] + corner_exp,
                coords[1] + corner_exp,
            ],
            Corner::TopRight => [
                coords[2] - corner_exp,
                coords[1],
                coords[2],
                coords[1] + corner_exp,
            ],
            Corner::BottomRight => [
                coords[0],
                coords[3] - corner_exp,
                coords[0] + corner_exp,
                coords[3],
            ],
            Corner::BottomLeft => [
                coords[2] - corner_exp,
                coords[3] - corner_exp,
                coords[2],
                coords[3],
            ],
        }
    }

    fn get_side_coords(side: usize, coords: &[f32], radius: f32) -> [f32; 4] {
        let corner_size = radius.ceil();
        let corner_exp = corner_size + 4.0;

        match side & 3 {
            0 => [
                coords[0] + corner_exp,
                coords[1],
                coords[2] - corner_exp,
                coords[1] + corner_exp,
            ],
            1 => [
                coords[0] + corner_exp,
                coords[3] - corner_exp,
                coords[2] - corner_exp,
                coords[3] - 2.0,
            ],
            2 => [
                coords[0],
                coords[1] + corner_size,
                coords[2] - 2.0,
                coords[3] - corner_size,
            ],
            _ => unimplemented!(),
        }
    }

    #[inline(never)]
    pub fn render_solid_quad_rounded(
        &self,
        output: &mut [i16],
        tile_info: &TileInfo,
        coords: &[f32],
        color: i16x8,
        radius: f32,
        blend_mode: BlendMode,
    ) {
        let corners = [
            Corner::TopLeft,
            Corner::TopRight,
            Corner::BottomRight,
            Corner::BottomLeft,
        ];

        // As we use pre-multiplied alpha we need to adjust the color based on the alpha value
        let color = premultiply_alpha(color);

        for corner in &corners {
            let corner_coords = Self::get_corner_coords(*corner, coords, radius);
            self.render_solid_rounded_corner(
                output,
                tile_info,
                &corner_coords,
                color,
                radius,
                blend_mode,
                *corner,
            );
        }

        for side in 0..3 {
            let side_coords = Self::get_side_coords(side, coords, radius);
            self.render_solid_quad(output, tile_info, &side_coords, color, blend_mode);
        }
    }

    #[inline(never)]
    pub fn render_solid_lerp_radius(
        &self,
        output: &mut [i16],
        tile_info: &TileInfo,
        coords: &[f32],
        radius: f32,
        top_colors: i16x8,
        bottom_colors: i16x8,
    ) {
        let uv_data = [0.0];
        let texture_sizes = [0];

        render_internal::<COLOR_MODE_LERP, TEXTURE_MODE_NONE, ROUND_MODE_ENABLED, BLEND_MODE_NONE>(
            output,
            self.scissor_rect,
            std::ptr::null(),
            tile_info,
            &uv_data,
            &texture_sizes,
            coords,
            radius,
            2,
            top_colors,
            bottom_colors,
        );
    }
}

