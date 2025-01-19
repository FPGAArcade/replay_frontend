mod raster;

use simd::*;
use raster::Raster;
use raw_window_handle::RawWindowHandle;

use flowi_core::Renderer as FlowiRenderer;
use flowi_core::{ApplicationSettings, RenderCommand};
use flowi_core::render::SoftwareRenderData;
use flowi_core::{RenderCommandConfig, CornerRadius};

pub(crate) struct TileInfo {
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
    primitives: Vec<RenderPrimitive>,
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
fn build_linear_to_srgb_table() -> [u8; 1 << SRGB_BIT_COUNT] {
    let mut table = [0; 1 << SRGB_BIT_COUNT];

    for (i, entry) in table.iter_mut().enumerate().take(1 << SRGB_BIT_COUNT) {
        let linear = i as f32 / ((1 << SRGB_BIT_COUNT) - 1) as f32;
        let srgb = linear_to_srgb(linear);
        *entry = (srgb * (1 << 8) as f32) as u8;
    }

    table
}

#[derive(Clone, Copy)]
pub struct RenderPrimitive {
    pub aabb: f32x4,
    pub color: i16x8,
    pub corner_radius: f32x4,
}

struct Tile {
    aabb: f32x4,
    data: Vec<usize>,
    tile_index: usize,
}

impl FlowiRenderer for Renderer {
    fn new(_settings: &ApplicationSettings, _window: Option<&RawWindowHandle>) -> Self {
        let screen_size = (1920, 1080);
        let tile_count = (10, 12); 

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
            primitives: Vec::with_capacity(8192),
            tile_buffers: [t0, t1],
            tiles,
            screen_width: screen_size.0,
            output: vec![0; screen_size.0 * screen_size.1 * 3],
        }
    }

    fn software_renderer_info<'a>(&'a self) -> Option<SoftwareRenderData<'a>> {
        Some(SoftwareRenderData {
            buffer: self.output.as_slice(),
            width: 1920,
            height: 1080,
        })
    }


    fn render(&mut self, commands: &[RenderCommand]) {
        self.primitives.clear();

        for command in commands {
            let aabb = f32x4::new(
                command.bounding_box.x,
                command.bounding_box.y,
                command.bounding_box.x + command.bounding_box.width,
                command.bounding_box.y + command.bounding_box.height,
            );

            match &command.config {
                RenderCommandConfig::Rectangle(rectangle) => {
                    let color = self.get_color_from_floats_0_255(
                        rectangle.color.r,
                        rectangle.color.g,
                        rectangle.color.b,
                        rectangle.color.a,
                    );

                    let corner_radius = match rectangle.corner_radius {
                        CornerRadius::All(radius) => f32x4::new(radius, radius, radius, radius),

                        CornerRadius::Individual {
                            top_left,
                            top_right,
                            bottom_left,
                            bottom_right,
                        } => f32x4::new(top_left, top_right, bottom_left, bottom_right),
                    };

                    let primitive = RenderPrimitive {
                        aabb,
                        color,
                        corner_radius,
                    };

                    self.add_primitive(primitive);
                }

                _ => {}
            }
        }

        Self::bin_primitives(&mut self.tiles, &self.primitives);

        let mut tile_info = TileInfo {
            offsets: f32x4::new_splat(0.0),
            width: 192,
            _height: 90,
        };

        self.raster.scissor_rect = f32x4::new(0.0, 0.0, 192.0, 90.0);

        for tile in self.tiles.iter_mut() {
            let mut coords = [0f32; 4];

            let tile_buffer = &mut self.tile_buffers[tile.tile_index];

            for t in tile_buffer.iter_mut() {
                *t = 0;
            }

            // TODO: Correct clearing of of the buffer
            //tile_buffer.clear();

            let tile_aabb = tile.aabb;
            let tile_buffer = &mut self.tile_buffers[tile.tile_index];
            tile_info.offsets = tile_aabb.shuffle_0101();

            //self.raster.scissor_rect = tile_aabb;

            for primitive_index in tile.data.iter() {
                let primitive = self.primitives[*primitive_index];
                let color = primitive.color;

                // TODO: Fix this
                let coords_vec = primitive.aabb;

                // TODO: Fix this
                coords_vec.store_unaligned(&mut coords);

                /*
                self.raster.render_solid_quad(
                    tile_buffer,
                    &tile_info,
                    &coords,
                    color,
                    raster::BlendMode::None);
                */

                self.raster.render_solid_quad_rounded(
                    tile_buffer,
                    &tile_info,
                    &coords,
                    color,
                    16.0,
                    raster::BlendMode::None,
                );
            }

            // Rasterize the primitives for this tile
            Self::copy_tile_linear_to_srgb(
                &self.linear_to_srgb_table,
                &mut self.output,
                &tile_buffer,
                tile,
                self.screen_width,
            );
        }
    }
}

impl Renderer {
    pub fn begin_frame(&mut self) {
        self.primitives.clear();
    }

    pub fn add_primitive(&mut self, primitive: RenderPrimitive) {
        self.primitives.push(primitive);
    }

    /*
    pub fn flush_frame(&mut self, output: &mut [u32]) {
        Self::bin_primitives(&mut self.tiles, &self.primitives);

        let mut tile_info = TileInfo {
            offsets: f32x4::new_splat(0.0),
            width: 192,
            _height: 90,
        };

        self.raster.scissor_rect = f32x4::new(0.0, 0.0, 192.0, 90.0);

        //let tile = &self.tiles[0];

        for tile in self.tiles.iter_mut() {
            let mut coords = [0f32; 4];

            let tile_buffer = &mut self.tile_buffers[tile.tile_index];

            for t in tile_buffer.iter_mut() {
                *t = 0;
            }

            // TODO: Correct clearing of of the buffer
            //tile_buffer.clear();

            let tile_aabb = tile.aabb;
            let tile_buffer = &mut self.tile_buffers[tile.tile_index];
            tile_info.offsets = tile_aabb.shuffle_0101();

            //self.raster.scissor_rect = tile_aabb;

            for primitive_index in tile.data.iter() {
                let primitive = self.primitives[*primitive_index];
                let color = primitive.color;

                // TODO: Fix this
                let coords_vec = primitive.aabb;

                // TODO: Fix this
                coords_vec.store_unaligned(&mut coords);

                /*
                self.raster.render_solid_quad(
                    tile_buffer,
                    &tile_info,
                    &coords,
                    color,
                    raster::BlendMode::None);
                */

                self.raster.render_solid_quad_rounded(
                    tile_buffer,
                    &tile_info,
                    &coords,
                    color,
                    16.0,
                    raster::BlendMode::None,
                );
            }

            // Rasterize the primitives for this tile
            Self::copy_tile_linear_to_srgb(
                &self.linear_to_srgb_table,
                &mut self.output,
                &tile_buffer,
                tile,
                self.screen_width,
            );
        }
    }
    */

    pub fn get_color_from_floats_0_255(&self, r: f32, g: f32, b: f32, a: f32) -> i16x8 {
        let r = self.get_color_from_float_0_255(r);
        let g = self.get_color_from_float_0_255(g);
        let b = self.get_color_from_float_0_255(b);
        let a = self.get_color_from_float_0_255(a);

        i16x8::new(r, g, b, a, r, g, b, a)
    }

    /// Bins the render primitives into the provided tiles.
    ///
    /// This function iterates over each tile and clears its data. Then, it iterates
    /// over the provided render primitives and checks if the primitive's axis-aligned
    /// bounding box (AABB) intersects with the tile's AABB. If there is a intersection,
    /// the index of the primitive is added to the tile's data.
    ///
    /// # Parameters
    /// - `tiles`: A mutable slice of `Tile` objects to bin the primitives into.
    /// - `primitives`: A slice of `RenderPrimitive` objects to be binned.
    fn bin_primitives(tiles: &mut [Tile], primitives: &[RenderPrimitive]) {
        for tile in tiles.iter_mut() {
            let tile_aabb = tile.aabb;
            tile.data.clear();
            for (i, primitive) in primitives.iter().enumerate() {
                if f32x4::test_intersect(tile_aabb, primitive.aabb) {
                    tile.data.push(i);
                }
            }
        }
    }

    fn get_color_from_float_0_255(&self, color: f32) -> i16 {
        let color = color.max(0.0).min(255.0) as usize;
        self.srgb_to_linear_table[color & 0xff] as i16
    }

    /*
    fn copy_tile_srgb(&self,
        output: &mut [u32],
        tile: &[i16],
        linear_to_srgb: &[u8; 4096],
        tile_width: usize,
        tile_height: usize,
        width: usize,
    ) {


    }
    */

    // Reference implementation. This will run in hw on the device.
    #[inline(never)]
    fn copy_tile_linear_to_srgb(
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
}
