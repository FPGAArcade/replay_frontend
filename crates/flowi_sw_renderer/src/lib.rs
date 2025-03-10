use simd::*;
use tracy_client::span;

pub mod raster;
pub mod sharp_bilinear;

pub use flowi_core::image::image::ImageInfo;
pub use flowi_core::primitives::Color16;
pub use flowi_core::Color;

pub use raster::{BlendMode, Corner, Raster};
use raw_window_handle::RawWindowHandle;

use flowi_core::render_api::{RenderCommand, RenderType, SoftwareRenderData};
use std::hash::{Hash, Hasher};

pub struct TileInfo {
    pub offsets: f32x4,
    pub width: i32,
    pub _height: i32,
}

pub enum ColorSpace {
    Linear,
    Srgb,
}

const SRGB_BIT_COUNT: i32 = 11;
const LINEAR_BIT_COUNT: i32 = 15;
const LINEAR_TO_SRGB_SHIFT: i32 = LINEAR_BIT_COUNT - SRGB_BIT_COUNT;

pub struct Renderer {
    raster: Raster,
    linear_to_srgb_table: [u8; 1 << SRGB_BIT_COUNT],
    srgb_to_linear_table: [u16; 1 << 8],
    // TODO: Arena
    tiles: Vec<Tile>,
    tile_buffer: Vec<Color16>,
    output: Vec<u8>,
    //tile_size: (usize, usize),
    screen_size: (usize, usize),
}

fn linear_to_srgb(x: f32) -> f32 {
    if x <= 0.0031308 {
        x * 12.92
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    }
}

fn srgb_to_linear2(x: f32) -> f32 {
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

// TODO: Verify that we are building the range correctly here
pub fn build_srgb_to_linear_table() -> [u16; 1 << 8] {
    let mut table = [0; 1 << 8];

    for (i, entry) in table.iter_mut().enumerate().take(1 << 8) {
        let srgb = i as f32 / 255.0;
        let linear = srgb_to_linear2(srgb);
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
    prev_hash: u64,
    current_hash: u64,
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
    linear_to_srgb_table: &[u8; 2048],
    output: &mut [u8],
    tile: &[Color16],
    tile_info: &Tile,
    width: usize,
) {
    let copy_tile = span!("copy_tile_linear_to_srgb");
    copy_tile.emit_color(0xFF00FF);

    let x0 = tile_info.aabb.extract::<0>() as usize;
    let y0 = tile_info.aabb.extract::<1>() as usize;
    let x1 = tile_info.aabb.extract::<2>() as usize;
    let y1 = tile_info.aabb.extract::<3>() as usize;

    let tile_width = x1 - x0;
    let tile_height = y1 - y0;

    let mut tile_ptr = tile.as_ptr();
    let mut output_index = ((y0 * width) + x0) * 3;
    let and_mask = i16x8::new_splat(0xfff);

    for _y in 0..tile_height {
        let mut current_index = output_index;
        for _x in 0..(tile_width >> 1) {
            let rgba_rgba = i16x8::load_unaligned_ptr(tile_ptr as _, 0);
            let rgba_rgba = rgba_rgba.shift_right::<LINEAR_TO_SRGB_SHIFT>();
            let rgba_rgba = rgba_rgba.and(and_mask);

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

                tile_ptr = tile_ptr.add(2);

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

#[inline(never)]
fn clear_tile_buffer(tile_buffer: &mut [Color16]) {
    let clear_tile = span!("clear tile");
    clear_tile.emit_color(0x0000FF);

    let c = i16x8::new_splat(200);
    let count = tile_buffer.len();

    for i in (0..count).step_by(8) {
        c.store_unaligned(tile_buffer, i + 0);
        c.store_unaligned(tile_buffer, i + 2);
        c.store_unaligned(tile_buffer, i + 4);
        c.store_unaligned(tile_buffer, i + 6);
    }
}

fn render_tiles(renderer: &mut Renderer, commands: &[RenderCommand]) {
    let span = span!("render_tiles");
    span.emit_color(0xFFFF00);

    for tile in renderer.tiles.iter_mut() {
        let tile_aabb = tile.aabb;

        let _ = span!("tile loop");

        if tile.prev_hash == tile.current_hash {
            continue;
        }

        /*
        if tile.data.is_empty() {
            continue;
        }

         */

        let tile_width = tile_aabb.extract::<2>() - tile_aabb.extract::<0>();
        let tile_height = tile_aabb.extract::<3>() - tile_aabb.extract::<1>();

        let tile_info = TileInfo {
            offsets: tile_aabb.shuffle_0101(),
            width: tile_width as _,
            _height: tile_height as _,
        };

        renderer.raster.scissor_rect = f32x4::new(0.0, 0.0, tile_width as _, tile_height as _);

        let tile_buffer = &mut renderer.tile_buffer;

        clear_tile_buffer(tile_buffer);

        for index in tile.data.iter() {
            let command = span!("commands");
            command.emit_color(0x00FF00);

            let render_cmd = &commands[*index];
            let blend_mode = if render_cmd.color.a == 255.0 {
                BlendMode::None
            } else {
                BlendMode::WithBackground
            };

            let color =
                get_color_from_floats_0_255(render_cmd.color, &renderer.srgb_to_linear_table);

            match &render_cmd.render_type {
                RenderType::DrawRect => {
                    let zone = span!("DrawRect");
                    zone.emit_color(0xFF00FF);
                    renderer.raster.render_solid_quad(
                        tile_buffer,
                        &tile_info,
                        &render_cmd.bounding_box,
                        color,
                        blend_mode,
                    );
                }

                RenderType::DrawRectRounded(rect) => {
                    let zone = span!("DrawRectRounded");
                    zone.emit_color(0xFF00FF);

                    renderer.raster.render_solid_quad_rounded(
                        tile_buffer,
                        &tile_info,
                        &render_cmd.bounding_box,
                        color,
                        &rect.corners,
                        blend_mode,
                    );
                }

                RenderType::DrawTextBuffer(buffer) => {
                    let zone = span!("DrawTextBuffer");
                    zone.emit_color(0xFF00FF);
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

                RenderType::DrawImage(buffer) => {
                    let zone = span!("DrawImage");
                    zone.emit_color(0xFF00FF);
                    let texture_sizes = [
                        buffer.width as _, buffer.height as _,
                        buffer.width as _, buffer.height as _];

                    let color = if blend_mode == BlendMode::WithBackground {
                        i16x8::new_splat((render_cmd.color.a as i16) << 7)
                    } else {
                        i16x8::new_splat(0x07fff)
                    };

                    //let uv = [0.0, 0.0, buffer.width as _, buffer.height as _];

                    sharp_bilinear::render_sharp_bilinear(
                        tile_buffer,
                        renderer.raster.scissor_rect,
                        &tile_info,
                        &render_cmd.bounding_box,
                        buffer.handle as _,
                        1.0,
                        blend_mode,
                        color,
                        buffer.stride as _,
                        &texture_sizes);

                    //dbg!("DrawImage {:?}", render_cmd.bounding_box);
                    //dbg!("DrawImage {:?}", buffer.handle);

                    /*
                    renderer.raster.render_aligned_texture(
                        tile_buffer,
                        &tile_info,
                        &render_cmd.bounding_box,
                        buffer.handle as _,
                        &uv,
                        &texture_sizes,
                    );

                     */
                }

                RenderType::DrawBackground(buffer) => {
                    let zone = span!("DrawBackground");
                    zone.emit_color(0xFF00FF);

                    renderer.raster.draw_background(
                        tile_buffer,
                        &tile_info,
                        &render_cmd.bounding_box,
                        buffer.width as _,
                        buffer.handle as _,
                    );
                }

                _ => {}
            }
        }

        // Rasterize the primitives for this tile
        copy_tile_linear_to_srgb(
            &renderer.linear_to_srgb_table,
            &mut renderer.output,
            tile_buffer,
            tile,
            renderer.screen_size.0,
        );
    }
}

fn get_tile_size(pos: usize, max_size: usize, tile_size: usize) -> usize {
    if pos + tile_size > max_size {
        max_size - pos
    } else {
        tile_size
    }
}

impl flowi_core::Renderer for Renderer {
    fn new(screen_size: (usize, usize), _window: Option<&RawWindowHandle>) -> Self {
        let tile_size = (128, 128);

        let mut tiles = Vec::new();

        for y in (0..screen_size.1).step_by(tile_size.1) {
            for x in (0..screen_size.0).step_by(tile_size.0) {
                let tile_width = get_tile_size(x, screen_size.0, tile_size.0);
                let tile_height = get_tile_size(y, screen_size.1, tile_size.1);

                tiles.push(Tile {
                    aabb: f32x4::new(
                        x as f32,
                        y as f32,
                        (x + tile_width) as f32,
                        (y + tile_height) as f32,
                    ),
                    data: Vec::with_capacity(8192),
                    prev_hash: 1,
                    current_hash: 0,
                });
            }
        }

        let tile_buffer = vec![Color16::default(); tile_size.0 * tile_size.1 * 8];

        Self {
            linear_to_srgb_table: build_linear_to_srgb_table(),
            srgb_to_linear_table: build_srgb_to_linear_table(),
            raster: Raster::new(),
            tile_buffer,
            tiles,
            screen_size,
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
        Self::hash_all_tiles(&mut self.tiles, commands);
        render_tiles(self, commands);
    }
}

impl Renderer {
    pub fn begin_frame(&mut self) {}

    /// Bins the render primitives into the provided tiles.
    ///
    /// This function iterates over the provided render primitives and checks if the
    /// primitive's (AABB) intersects with the tile's AABB. If there is an intersection,
    /// the index of the primitive is added to the tile's data.
    ///
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

    fn hash_all_tiles(tiles: &mut [Tile], commands: &[RenderCommand]) {
        let zone = span!("hash_all_tiles");
        zone.emit_color(0xFFFF00);
        for tile in tiles.iter_mut() {
            tile.prev_hash = tile.current_hash;
            tile.current_hash = Self::hash_tile_data(tile, commands);
        }
    }
    fn hash_tile_data(tile: &Tile, commands: &[RenderCommand]) -> u64 {
        let mut hasher = fxhash::FxHasher::default();
        for index in tile.data.iter() {
            let command = &commands[*index];
            command.hash(&mut hasher);
        }
        hasher.finish()
    }
}
