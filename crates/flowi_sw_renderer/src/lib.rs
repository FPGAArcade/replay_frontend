use simd::*;

pub mod raster;
pub use raster::{BlendMode, Corner, Raster};
use raw_window_handle::RawWindowHandle;

use flowi_renderer::{Color, RenderCommand, RenderType, SoftwareRenderData};

pub struct TileInfo {
    pub offsets: f32x4,
    pub width: i32,
    pub _height: i32,
}

pub enum ColorSpace {
    Linear,
    Srgb,
}

const SRGB_BIT_COUNT: i32 = 12;
const LINEAR_BIT_COUNT: i32 = 15;
const LINEAR_TO_SRGB_SHIFT: i32 = LINEAR_BIT_COUNT - SRGB_BIT_COUNT;

pub struct Renderer {
    raster: Raster,
    linear_to_srgb_table: [u8; 1 << SRGB_BIT_COUNT],
    srgb_to_linear_table: [u16; 1 << 8],
    // TODO: Arena
    tiles: Vec<Tile>,
    tile_buffers: [Vec<i16>; 2],
    output: Vec<u8>,
    screen_width: usize,
}

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

// TODO: Verify that we are building the range correctly here
fn build_srgb_to_linear_table() -> [u16; 1 << 8] {
    let mut table = [0; 1 << 8];

    for (i, entry) in table.iter_mut().enumerate().take(1 << 8) {
        let srgb = i as f32 / 255.0;
        let linear = srgb_to_linear(srgb);
        *entry = (linear * ((1 << LINEAR_BIT_COUNT) - 1) as f32).round() as u16;
    }

    table
}

// TODO: Verify that we are building the range correctly here
pub fn build_linear_to_srgb_table() -> [u8; 1 << SRGB_BIT_COUNT] {
    let mut table = [0; 1 << SRGB_BIT_COUNT];

    for (i, entry) in table.iter_mut().enumerate().take(1 << SRGB_BIT_COUNT) {
        let linear = i as f32 / ((1 << SRGB_BIT_COUNT) - 1) as f32;
        let srgb = linear_to_srgb(linear);
        *entry = (srgb * (1 << 8) as f32) as u8;
    }

    table
}

pub struct Tile {
    aabb: f32x4,
    data: Vec<usize>,
    tile_index: usize,
}

pub fn get_color_from_floats_0_255(color: Color, srgb_to_linear_table: &[u16; 1 << 8]) -> i16x8 {
    let r = srgb_to_linear_table[(color.r as u8) as usize] as i16;
    let g = srgb_to_linear_table[(color.g as u8) as usize] as i16;
    let b = srgb_to_linear_table[(color.b as u8) as usize] as i16;
    let a = (color.a as i16) << 7;

    i16x8::new(r, g, b, a, r, g, b, a)
}

// Reference implementation. This will run in hw on the device.
#[inline(never)]
#[allow(clippy::identity_op)]
pub fn copy_tile_linear_to_srgb(
    linear_to_srgb_table: &[u8; 4096],
    output: &mut [u8],
    tile: &[i16],
    tile_info: &Tile,
    width: usize,
) {
    let x0 = tile_info.aabb.extract::<0>() as usize;
    let y0 = tile_info.aabb.extract::<1>() as usize;
    let x1 = tile_info.aabb.extract::<2>() as usize;
    let y1 = tile_info.aabb.extract::<3>() as usize;

    let tile_width = x1 - x0;
    let tile_height = y1 - y0;

    let mut tile_ptr = tile.as_ptr();
    let mut output_index = ((y0 * width) + x0) * 3;

    for _y in 0..tile_height {
        let mut current_index = output_index;
        for _x in 0..(tile_width >> 1) {
            let rgba_rgba = i16x8::load_unaligned_ptr(tile_ptr);
            let rgba_rgba = rgba_rgba.shift_right::<LINEAR_TO_SRGB_SHIFT>();

            let r0 = rgba_rgba.extract::<0>() as u16;
            let g0 = rgba_rgba.extract::<1>() as u16;
            let b0 = rgba_rgba.extract::<2>() as u16;

            let r1 = rgba_rgba.extract::<4>() as u16;
            let g1 = rgba_rgba.extract::<5>() as u16;
            let b1 = rgba_rgba.extract::<6>() as u16;

            unsafe {
                let r0 = *linear_to_srgb_table.get_unchecked(r0 as usize);
                let g0 = *linear_to_srgb_table.get_unchecked(g0 as usize);
                let b0 = *linear_to_srgb_table.get_unchecked(b0 as usize);

                let r1 = *linear_to_srgb_table.get_unchecked(r1 as usize);
                let g1 = *linear_to_srgb_table.get_unchecked(g1 as usize);
                let b1 = *linear_to_srgb_table.get_unchecked(b1 as usize);

                tile_ptr = tile_ptr.add(8);

                *output.get_unchecked_mut(current_index + 0) = r0;
                *output.get_unchecked_mut(current_index + 1) = g0;
                *output.get_unchecked_mut(current_index + 2) = b0;
                *output.get_unchecked_mut(current_index + 3) = r1;
                *output.get_unchecked_mut(current_index + 4) = g1;
                *output.get_unchecked_mut(current_index + 5) = b1;
            }

            current_index += 6;
        }

        output_index += width * 3;
    }
}

fn render_tiles(renderer: &mut Renderer, commands: &[RenderCommand]) {
    let mut tile_info = TileInfo {
        offsets: f32x4::new_splat(0.0),
        width: 128,
        _height: 90,
    };

    renderer.raster.scissor_rect = f32x4::new(0.0, 0.0, 128.0, 90.0);
        
    //dbg!("---------------------------------");

    for tile in renderer.tiles.iter_mut() {
        let tile_aabb = tile.aabb;
        tile_info.offsets = tile_aabb.shuffle_0101();

        if tile.data.is_empty() {
            continue;
        }

        let tile_buffer = &mut renderer.tile_buffers[tile.tile_index];

        // TODO: We should here support clearing the buffer with another color, bg image or have a
        // check during the binning if we need to clear at all.
        for t in tile_buffer.iter_mut() {
            *t = 0;
        }

        for index in tile.data.iter() {
            let render_cmd = &commands[*index];
            let color =
                get_color_from_floats_0_255(render_cmd.color, &renderer.srgb_to_linear_table);

            match &render_cmd.render_type {
                RenderType::DrawRect => {
                    //dbg!("DrawRect {:?}", render_cmd.bounding_box);
                    renderer.raster.render_solid_quad(
                        tile_buffer,
                        &tile_info,
                        &render_cmd.bounding_box,
                        color,
                        raster::BlendMode::None,
                    );
                }

                RenderType::DrawRectRounded(rect) => {
                    //dbg!("DrawRectRounded {:?}", render_cmd.bounding_box);
                    //dbg!("DrawRectRounded");
                    renderer.raster.render_solid_quad_rounded(
                        tile_buffer,
                        &tile_info,
                        &render_cmd.bounding_box,
                        color,
                        &rect.corners,
                        raster::BlendMode::None,
                    );
                }

                RenderType::DrawTextBuffer(buffer) => {
                    if buffer.data.0 == core::ptr::null() {
                        continue;
                    }

                    let coords = [
                        render_cmd.bounding_box[0],
                        render_cmd.bounding_box[1],
                        render_cmd.bounding_box[0] + buffer.width as f32,
                        render_cmd.bounding_box[1] + buffer.height as f32,
                    ];

                    renderer.raster.render_text_texture(
                        tile_buffer,
                        buffer.data.0 as _,
                        &tile_info,
                        buffer.width as _,
                        &coords,
                        color,
                    );
                }

                _ => {}
            }

            /*
            self.raster.render_solid_quad(
                tile_buffer,
                &tile_info,
                &coords,
                color,
                raster::BlendMode::None);

            self.raster.render_solid_quad_rounded(
                tile_buffer,
                &tile_info,
                &render_command.coords,
                color,
                16.0,
                raster::BlendMode::None,
            );
            */
        }

        // Rasterize the primitives for this tile
        copy_tile_linear_to_srgb(
            &renderer.linear_to_srgb_table,
            &mut renderer.output,
            tile_buffer,
            tile,
            renderer.screen_width,
        );
    }
}

impl flowi_renderer::Renderer for Renderer {
    fn new(_window: Option<&RawWindowHandle>) -> Self {
        let screen_size = (1280, 720);
        let tile_count = (10, 8);

        let tile_size = (screen_size.0 / tile_count.0, screen_size.1 / tile_count.1);
        let total_tile_count = tile_count.0 * tile_count.1;
        let tile_full_size = tile_size.0 * tile_size.1;

        let mut tiles = Vec::with_capacity(total_tile_count);
        let mut tile_index = 0;

        for y in (0..screen_size.1).step_by(tile_size.1) {
            for x in (0..screen_size.0).step_by(tile_size.0) {
                tiles.push(Tile {
                    aabb: f32x4::new(
                        x as f32,
                        y as f32,
                        (x + tile_size.0) as f32,
                        (y + tile_size.1) as f32,
                    ),
                    data: Vec::with_capacity(8192),
                    tile_index: tile_index & 1,
                });

                tile_index += 1;
            }
        }

        let t0 = vec![i16::default(); tile_full_size * 8];
        let t1 = vec![i16::default(); tile_full_size * 8];

        Self {
            linear_to_srgb_table: build_linear_to_srgb_table(),
            srgb_to_linear_table: build_srgb_to_linear_table(),
            raster: Raster::new(),
            tile_buffers: [t0, t1],
            tiles,
            screen_width: screen_size.0,
            output: vec![0; screen_size.0 * screen_size.1 * 3],
        }
    }

    fn software_renderer_info(&self) -> Option<SoftwareRenderData<'_>> {
        Some(SoftwareRenderData {
            buffer: self.output.as_slice(),
            width: 1280,
            height: 720,
        })
    }

    fn render(&mut self, commands: &[RenderCommand]) {
        Self::bin_primitives(&mut self.tiles, commands);
        render_tiles(self, commands);
    }
}

impl Renderer {
    pub fn begin_frame(&mut self) {}

    /// Bins the render primitives into the provided tiles.
    ///
    /// This function iterates over each tile and clears its data. Then, it iterates
    /// over the provided render primitives and checks if the primitive's axis-aligned
    /// bounding box (AABB) intersects with the tile's AABB. If there is a intersection,
    /// the index of the primitive is added to the tile's data.
    ///
    /// # Parameters
    /// - `tiles`: A mutable slice of `Tile` objects to bin the primitives into.
    /// - `commands`: RenderCommands to be binned
    fn bin_primitives(tiles: &mut [Tile], commands: &[RenderCommand]) {
        for tile in tiles.iter_mut() {
            let tile_aabb = tile.aabb;
            tile.data.clear();
            for (i, command) in commands.iter().enumerate() {
                let prim_aabb = f32x4::load_unaligned(&command.bounding_box);
                if f32x4::test_intersect(tile_aabb, prim_aabb) {
                    tile.data.push(i);
                }
            }
        }
    }
}
