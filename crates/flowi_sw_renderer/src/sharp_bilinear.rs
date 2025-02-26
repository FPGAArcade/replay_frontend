use simd::*;
use crate::TileInfo;
use flowi_core::primitives::Color16;
use simd::*;
use crate::raster::calculate_render_params;

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
    let u_step = i32x4::new_splat((texture_size.0 << 15) / xlen);
    let v_step = i32x4::new_splat((texture_size.1 << 15) / ylen);

    for _y in 0..ylen {
        for _x in 0..(xlen >> 1) {

        }
    }
}
