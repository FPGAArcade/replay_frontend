use simd::*;
use crate::TileInfo;
use flowi_core::primitives::Color16;
use simd::*;
use crate::raster::calculate_render_params;

const FRACT_BITS: i32 = 15;
const FRACT_MASK: i32 = (1 << FRACT_BITS) - 1;

pub(crate) fn render_sharp_bilinear(
    output: &mut [Color16],
    scissor_rect: f32x4,
    tile_info: &TileInfo,
    coords: &[f32],
    texture_data: *const Color16,
    texture_size: (i32, i32))
{
    let rp = if let Some(params) = calculate_render_params(coords, tile_info, scissor_rect) {
        params
    } else {
        return;
    };

    let texture_width = texture_size.0 as usize;
    let text_data = unsafe { texture_data.add((rp.clip_y * texture_width) + rp.clip_x) };

    let x0 = rp.x0;
    let y0 = rp.y0;
    let x1 = rp.x1;
    let y1 = rp.y1;

    let ylen = y1 - y0;
    let xlen = x1 - x0;

    // calculate the u,v step for the texture
    let v_step = i32x4::new_splat((texture_size.1 << FRACT_BITS) / ylen);
    let u_step = i32x4::new_splat((texture_size.0 << FRACT_BITS) / xlen);

    let tex_width = i32x4::new_splat(texture_size.0);
    let fract_mask = i32x4::new_splat(FRACT_MASK);

    // setup for interpolating 4 steps
    let u_start = i32x4::new(0, 1, 2, 3) * u_step;
    let u_step = u_step * i32x4::new_splat(4);
    let mut v = i32x4::new_splat(0);

    for _y in 0..ylen {
        let v_fract = v & fract_mask;
        let v_int = v.shift_right::<FRACT_BITS>();
        let v_offset = v_int * tex_width;

        let mut u = u_start;

        for _x in 0..(xlen >> 1) {
            let u_int = u.shift_right::<FRACT_BITS>();
            let uv = u_int + v_offset;


            u += u_step;
        }

        v += v_step;
    }
}
