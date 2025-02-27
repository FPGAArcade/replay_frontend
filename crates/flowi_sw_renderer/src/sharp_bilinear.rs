use crate::TileInfo;
use flowi_core::primitives::Color16;
use simd::*;
use crate::raster::calculate_render_params;

const FRACT_BITS: i32 = 15;
const FRACT_MASK: i32 = (1 << FRACT_BITS) - 1;
//const ONE_FIXED: i32 = 1 << FRACT_BITS;

fn sample_aligned_texture(
    texture: *const Color16,
    texture_width: usize,
    u_fraction: i16x8,
    v_fraction: i16x8,
    offset: usize,
) -> i16x8 {
    let rgba_rgba_0 = i16x8::load_unaligned_ptr(texture, offset);
    let rgba_rgba_1 = i16x8::load_unaligned_ptr(texture, texture_width + offset);
    let t0_t1 = i16x8::lerp(rgba_rgba_0, rgba_rgba_1, v_fraction);
    let t = t0_t1.rotate_4();
    i16x8::lerp(t0_t1, t, u_fraction)
}


fn sample_aligned_texture_2(
    texture: *const Color16,
    texture_width: usize,
    u_fraction: i16x8,
    v_fraction: i16x8,
    offset: usize,
    offset2: usize,
) -> i16x8 {
    let rgba_rgba_0 = i16x8::load_unaligned_ptr(texture, offset);
    let rgba_rgba_1 = i16x8::load_unaligned_ptr(texture, texture_width + offset);

    let rgba_rgba_2 = i16x8::load_unaligned_ptr(texture, offset2);
    let rgba_rgba_3 = i16x8::load_unaligned_ptr(texture, texture_width + offset2);

    let t0_t1 = i16x8::lerp(rgba_rgba_0, rgba_rgba_1, v_fraction);
    let t2_t3 = i16x8::lerp(rgba_rgba_2, rgba_rgba_3, v_fraction);

    // at this point we have the values stored like this
    // [t0 t1], [t2, t3] and we want [t0, t2], [t1, t3]
    let t0_t2 = i16x8::merge_low(t0_t1, t2_t3);
    let t1_t3 = i16x8::merge_high(t0_t1, t2_t3);

    i16x8::lerp(t0_t2, t1_t3, u_fraction)
}


#[inline]
fn apply_aa_simd(coord: i32x4, _scale: f32) -> i32x4 {
    coord
    /*

    let fract_mask = i32x4::new_splat(FRACT_MASK);
    let int_part = coord.shift_right::<FRACT_BITS>().shift_left::<FRACT_BITS>();
    let fract_part = coord & fract_mask;

    let fract = fract_mask.as_f32x4() * f32x4::new_splat(scale);
    let clamp = fract.min(f32x4::new_splat(FRACT_MASK as f32));
    let clamp_i = clamp.as_i32x4();

    // Final result
    int_part + clamp_i

     */
}

pub fn render_sharp_bilinear(
    output: &mut [Color16],
    scissor_rect: f32x4,
    tile_info: &TileInfo,
    coords: &[f32],
    texture_data: *const Color16,
    scale_factor: f32,
    texture_size: (i32, i32))
{
    let rp = if let Some(params) = calculate_render_params(coords, tile_info, scissor_rect) {
        params
    } else {
        return;
    };

    let x0y0x1y1_adjust =
        (f32x4::load_unaligned(coords) - tile_info.offsets) + f32x4::new_splat(0.5);
    let x0y0x1y1 = x0y0x1y1_adjust.floor();
    let x0y0x1y1_int = x0y0x1y1.as_i32x4();

    let texture_width = texture_size.0 as usize;

    let x0 = rp.x0;
    let y0 = rp.y0;
    let x1 = rp.x1;
    let y1 = rp.y1;

    let ylen = y1 - y0;
    let xlen = x1 - x0;

    let ylen_delta = x0y0x1y1_int.extract::<3>() - x0y0x1y1_int.extract::<1>();
    let xlen_delta = x0y0x1y1_int.extract::<2>() - x0y0x1y1_int.extract::<0>();

    let output = &mut output[(y0 as usize * tile_info.width as usize + x0 as usize)..];

    // calculate the u,v step for the texture
    let v_step = i32x4::new_splat((texture_size.1 << FRACT_BITS) / ylen_delta);
    let u_step = i32x4::new_splat((texture_size.0 << FRACT_BITS) / xlen_delta);

    let tex_width = i32x4::new_splat(texture_size.0);
    let fract_mask = i32x4::new_splat(FRACT_MASK);

    let clip_x = rp.clip_x as i32;

    // setup for interpolating 4 steps
    let u_start = i32x4::new(clip_x, clip_x + 1, clip_x + 2, clip_x + 3) * u_step;
    let u_step = u_step * i32x4::new_splat(4);

    let mut v = i32x4::new_splat(rp.clip_y as _) * v_step;

    // multiply scale factor up to fixed point range
    let scale_factor = scale_factor * 32767.0;

    for _y in 0..ylen {
        let vt = apply_aa_simd(v, scale_factor);
        let v_fract = vt & fract_mask;
        let v_int = vt.shift_right::<FRACT_BITS>();
        let v_offset = v_int * tex_width;
        let v_fract = v_fract.pack_i16x8();
        let v_fract = v_fract.shuffle::<0x0000_0000>();

        let mut u = u_start;

        for x in (0..xlen).step_by(4) {
            let ut = apply_aa_simd(u, scale_factor);
            let u_int = ut.shift_right::<FRACT_BITS>();
            let u_fract = ut & fract_mask;
            let u_fract = u_fract.pack_i16x8();

            let u0_fract = u_fract.shuffle::<0x0000_1111>();
            let u2_fract = u_fract.shuffle::<0x2222_3333>();

            let uv = u_int + v_offset;
            let uv0 = uv.extract::<0>();
            let uv1 = uv.extract::<1>();
            let uv2 = uv.extract::<2>();
            let uv3 = uv.extract::<3>();

            let c0 = sample_aligned_texture_2(texture_data, texture_width, u0_fract, v_fract, uv0 as _, uv1 as _);
            let c1 = sample_aligned_texture_2(texture_data, texture_width, u2_fract, v_fract, uv2 as _, uv3 as _);

            c0.store_unaligned(output, (_y * tile_info.width + x + 0) as _);
            c1.store_unaligned(output, (_y * tile_info.width + x + 2) as _);

            u += u_step;
        }

        v += v_step;
    }
}
