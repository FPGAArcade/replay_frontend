//use rayon::prelude::*;

impl Color {
    pub fn new(r: u16, g: u16, b: u16, a: u16) -> Self {
        Color { r, g, b, a }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Uv {
    pub u: f32,
    pub v: f32,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Color {
    pub r: u16,
    pub g: u16,
    pub b: u16,
    pub a: u16,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct InternalVertex {
    pub pos: Point,
    pub uv: Uv,
    pub color: u32,
}

impl InternalVertex {
    pub fn new(pos: Point, uv: Uv, color: u32) -> Self {
        InternalVertex { pos, uv, color }
    }
}

// Define the type `Vertex` based on whether the `imgui` feature is enabled or not.
#[cfg(not(feature = "imgui"))]
pub type Vertex = InternalVertex;

const RENDER_WIDTH: usize = 1920;
const RENDER_HEIGHT: usize = 1080;
pub const TILE_WIDTH: usize = 128;
pub const TILE_HEIGHT: usize = 120;

#[derive(Debug, Clone, Copy)]
struct TilePosition {
    x: i16,
    y: i16,
}

/*
#[derive(Debug)]
struct TriangleFlat {
    vertices: (Point, Point, Point),
    color: u32,
}

#[derive(Debug)]
struct TriangleTextured {
    vertices: (Point, Point, Point),
    uv: (Uv, Uv, Uv),
    texture: u32,
    color: u32,
}
*/

#[derive(Debug)]
struct QuadFlat {
    vertices: (Point, Point),
    color: u32,
}

/*
#[derive(Debug)]
struct QuadTextured {
    vertices: (Point, Point),
    uv: (Uv, Uv),
    texture: u32,
    color: u32,
}
*/

#[derive(Debug)]
enum RenderPrimitive {
    //TriangleFlat(TriangleFlat),
    //TriangleTextured(TriangleTextured),
    QuadFlat(QuadFlat),
    //QuadTextured(QuadTextured),
}

#[derive(Clone, Default)]
pub struct CommandBuffer<'a> {
    pub commands: Vec<u32>,
    vertices: Option<&'a [Vertex]>,
    index_buffer: Option<&'a [u16]>,
    count: usize,
}

/*
#[derive(Debug, Clone)]
struct TilePrimitives<'a> {
    vertices: Option<&'a [Vertex]>,
    index_buffer: Option<&'a [u16]>,
    primitives: Vec<u32>,
    primitive_count: usize,
}
*/

#[derive(Debug)]
pub struct Tile {
    primitives: Vec<RenderPrimitive>,
    min: TilePosition,
    max: TilePosition,
    local_tile_index: usize,
}

pub struct SwRasterizer<'a> {
    // TODO: Optimize this to use a single buffer
    pub commands: Vec<CommandBuffer<'a>>,
    // TODO: Optimize this to use a single buffer
    pub tiles: Vec<Tile>,
    index: usize,
    tile_width: usize,
    tile_height: usize,
}

const HAS_SAME_COLOR: u32 = 17;
const NON_TEXTURED: u32 = 18;
const IS_QUAD: u32 = 19;

/*
https://godbolt.org/z/W5ffYx9d5
Some key takeaways in the SIMD implementation that I figured out:
Only two shuffles are required
We can do the compares using u32 as we are only comparing if the floats are equal so this is fine
By using u32s we can compare the UVs and the colors at the same time
So when the compares are complete we have a vector with the result (in u32s) x compare, y compare, uv compare, color compare
I then and this vector with a fixed value which has 1 << IS_QUAD + 1, 1 << IS_QUAD, 1 << NO_TEXTURE, 1 << SAME_COLOR
Now the result will be some of these if the compare above was success or not.
Now I use advq in ARM which does a add across lanes which ends up as a single u32 value
Using this value I shift down by IS_QUAD and if the value is 3 (meaning that both x and y is equal as they have been added) I know how much to advance the index buffer
*
#include <stdint.h>
#include <arm_neon.h>

typedef struct Vertex {
    float x, y;
    uint16_t u, v;
    uint32_t color;
} Vertex;

const uint32_t HAS_SAME_COLOR = 17;
const uint32_t NON_TEXTURED = 18;
const uint32_t IS_QUAD = 19;

uint32_t cat_input_simd(uint32_t* output, const Vertex* vertices, const uint32_t* indices, int count) {
    uint32_t index = 0;
    uint32_t write_index = 0;
    uint32_t total_count = count - 3;

    uint32_t fixed_data[4] = { 1 << (IS_QUAD + 1), 1 << IS_QUAD, 1 << NON_TEXTURED, 1 << HAS_SAME_COLOR };
    uint32x4_t fixed_shift = vld1q_u32(&fixed_data);

    while (index < total_count) {
        uint32_t i0 = indices[index + 0];
        uint32_t i1 = indices[index + 1];
        uint32_t i2 = indices[index + 2];
        uint32_t i3 = indices[index + 5];

        uint32x4_t x0y0uv0c0 = vld1q_u32((uint32_t*)(vertices + i0));
        uint32x4_t x1y1uv1c1 = vld1q_u32((uint32_t*)(vertices + i1));
        uint32x4_t x2y2uv2c2 = vld1q_u32((uint32_t*)(vertices + i2));
        uint32x4_t x3y3uv3c3 = vld1q_u32((uint32_t*)(vertices + i3));

        uint32x4_t x3y1uv1c1 = __builtin_shufflevector(x3y3uv3c3, x1y1uv1c1, 0, 5, 6, 7);
        uint32x4_t x1y3uv3c3 = __builtin_shufflevector(x1y1uv1c1, x3y3uv3c3, 0, 5, 6, 7);

        uint32x4_t cmp0 = vceqq_u32(x0y0uv0c0, x3y1uv1c1);
        uint32x4_t cmp1 = vceqq_u32(x2y2uv2c2, x1y3uv3c3);

        // 0: same_x
        // 1: same_y
        // 2: same_uv
        // 3: same color
        uint32x4_t cmp = vandq_u32(cmp0, cmp1);
        uint32x4_t res = vandq_u32(cmp, fixed_shift);
        // add across for the final result
        uint32_t t = vaddvq_u32(res);
        uint32_t is_quad = t >> IS_QUAD;

        output[write_index] = t | index;
        index += is_quad == 3 ? 6 : 3;
        write_index++;
    }

    return write_index;
}
*/

/// # Safety
/// This function is unsafe because it operates on raw pointers and does not perform bounds checking.
pub unsafe fn cat_triangles(output: &mut [u32], vertices: &[Vertex], indices: &[u16]) -> usize {
    let mut index = 0;
    let mut write_index = 0;
    let total_count = indices.len() - 3;

    while index < total_count {
        let i0 = *indices.get_unchecked(index);
        let i1 = *indices.get_unchecked(index + 1);
        let i2 = *indices.get_unchecked(index + 2);
        let i3 = *indices.get_unchecked(index + 5);

        let v0 = *vertices.get_unchecked(i0 as usize);
        let v1 = *vertices.get_unchecked(i1 as usize);
        let v2 = *vertices.get_unchecked(i2 as usize);
        let v3 = *vertices.get_unchecked(i3 as usize);

        let same_color = if v0.color == v1.color && v0.color == v2.color && v0.color == v3.color {
            1
        } else {
            0
        };

        // not sure if equal compare will be ok here, must verify how imgui *exactly* calculate this value
        // We short cut this a bit given if two verts has this value, we assume the rest has it
        let white_uv =
            if v0.uv.u == v1.uv.u && v0.uv.v == v1.uv.v && v2.uv.u == v3.uv.u && v2.uv.v == v3.uv.v
            {
                1
            } else {
                0
            };

        let is_quad = if v0.pos.x == v3.pos.x
            && v1.pos.x == v2.pos.x
            && v0.pos.y == v1.pos.y
            && v2.pos.y == v3.pos.y
        {
            2
        } else {
            0
        };

        let t = (white_uv << NON_TEXTURED)
            | (same_color << HAS_SAME_COLOR)
            | (is_quad << IS_QUAD)
            | (index as u32);

        *output.get_unchecked_mut(write_index) = t;
        index += if is_quad == 2 { 6 } else { 3 };
        write_index += 1;
    }

    write_index
}

unsafe fn bin_primitives(tile: &mut Tile, commands: &[CommandBuffer]) {
    let tile_min_x = tile.min.x as f32;
    let tile_min_y = tile.min.y as f32;
    let tile_max_x = tile.max.x as f32;
    let tile_max_y = tile.max.y as f32;

    tile.primitives.clear();

    for command_buffer in commands {
        for i in 0..command_buffer.count {
            let vertices = command_buffer.vertices.unwrap_unchecked();
            let indices = command_buffer.index_buffer.unwrap_unchecked();

            let command = command_buffer.commands[i];
            let is_quad = (command >> IS_QUAD) & 3;
            let not_textured = (command >> NON_TEXTURED) & 1;
            let index = (command & 0xFFFF) as usize;
            // only deal with quads now

            if is_quad != 2 {
                continue;
            }

            /*
            let i0 = *indices.get_unchecked(index + 0);
            let i3 = *indices.get_unchecked(index + 5);

            let v0 = vertices.get_unchecked(i0 as usize);
            let v3 = vertices.get_unchecked(i3 as usize);
            */

            let i0 = indices[index] as usize;
            let i3 = indices[index + 2] as usize;

            let v0 = vertices[i0];
            let v3 = vertices[i3];

            let prim_pos_min = v0.pos;
            let prim_pos_max = v3.pos;

            // skip if the primitive is fully outside the tile, but keep if it's partially inside
            // and fully inside

            if (prim_pos_max.x < tile_min_x || prim_pos_min.x > tile_max_x)
                || prim_pos_max.y < tile_min_y
                || prim_pos_min.y > tile_max_y
            {
                continue;
            }

            if not_textured == 1 {
                let prim = RenderPrimitive::QuadFlat(QuadFlat {
                    vertices: (prim_pos_min, prim_pos_max),
                    color: v0.color,
                });

                tile.primitives.push(prim);
            } else {
                /*
                let prim = RenderPrimitive::QuadTextured(QuadTextured {
                    vertices: (prim_pos_min, prim_pos_max),
                    uv: (v0.uv, v3.uv),
                    texture: 0,
                    color: v0.color,
                });

                tile.primitives.push(prim);
                */
            }
        }
    }
}

unsafe fn rasterizer_tile(_tile_buffer: &mut [u32], tile: &Tile, main_buffer: &mut [u32]) {
    let tile_min_x = tile.min.x as f32;
    let tile_min_y = tile.min.y as f32;
    let tile_max_x = tile.max.x as f32;
    let tile_max_y = tile.max.y as f32;

    for primitive in &tile.primitives {
        match primitive {
            RenderPrimitive::QuadFlat(quad) => {
                let quad_min = quad.vertices.0;
                let quad_max = quad.vertices.1;

                // clip against the tile borders
                let x0 = quad_min.x.max(tile_min_x) as usize;
                let y0 = quad_min.y.max(tile_min_y) as usize;

                let x1 = quad_max.x.min(tile_max_x) as usize;
                let y1 = quad_max.y.min(tile_max_y) as usize;

                for y in y0..y1 {
                    for x in x0..x1 {
                        let index = y * RENDER_WIDTH + x;
                        main_buffer[index] = quad.color;
                    }
                }
            } // _ => {}
        }
    }
}

impl<'a> SwRasterizer<'a> {
    pub fn new(tile_width: usize, tile_height: usize) -> Self {
        let mut tiles =
            Vec::with_capacity((RENDER_WIDTH / tile_width) * (RENDER_HEIGHT / tile_height));
        let mut tile_index = 0;

        for y in (0..RENDER_HEIGHT).step_by(tile_height) {
            for x in (0..RENDER_WIDTH).step_by(tile_width) {
                tiles.push(Tile {
                    primitives: Vec::new(),
                    min: TilePosition {
                        x: x as i16,
                        y: y as i16,
                    },
                    max: TilePosition {
                        x: (x + tile_width) as i16,
                        y: (y + tile_height) as i16,
                    },
                    local_tile_index: tile_index & 0x3,
                });

                tile_index += 1;
            }
        }

        SwRasterizer {
            commands: Vec::new(),
            tiles,
            index: 0,
            tile_width,
            tile_height,
        }
    }

    /// Begin a new frame. vertex_lists is the number of vertex lists will be processed this frame
    pub fn begin(&mut self, vertex_lists: usize) {
        self.commands.clear();
        self.index = 0;

        if self.commands.len() < vertex_lists {
            self.commands.resize(vertex_lists, CommandBuffer::default());
        }
    }

    pub fn add_vertices(&mut self, vertices: &'a [Vertex], indices: &'a [u16]) {
        let index = self.index;
        let command_buffer = &mut self.commands[index];

        // Ensure the command buffer is large enough to hold the new commands
        if command_buffer.commands.len() < indices.len() {
            command_buffer.commands.resize(indices.len(), 0);
        }

        command_buffer.count = 0;
        command_buffer.vertices = Some(vertices);
        command_buffer.index_buffer = Some(indices);

        self.index += 1;
    }

    /// # Safety
    /// This function is unsafe because it operates on raw pointers and does not perform bounds checking.
    pub unsafe fn rasterize(&mut self, output: &mut [u32]) {
        // process the command buffers
        let mut tile_buffer = vec![0u32; self.tile_width * self.tile_height];

        for i in 0..self.index {
            let command_buffer = &mut self.commands[i];
            let vertices = command_buffer.vertices.unwrap_unchecked();
            let indices = command_buffer.index_buffer.unwrap_unchecked();
            let count = cat_triangles(&mut command_buffer.commands, vertices, indices);

            command_buffer.count = count;
        }

        for tile in self.tiles.iter_mut() {
            bin_primitives(tile, &self.commands);
        }

        for tile in self.tiles.iter_mut() {
            rasterizer_tile(&mut tile_buffer, tile, output);
        }
    }

    #[inline(always)]
    fn get_tile_buffer<'b>(
        tile_offset: usize,
        len: usize,
        tile_buffers: *mut u32,
    ) -> &'b mut [u32] {
        unsafe { std::slice::from_raw_parts_mut(tile_buffers.add(tile_offset), len) }
    }

    unsafe fn clear_tile(tile_buffer: &mut [u32]) {
        tile_buffer.fill(0x00ff00ff);
    }

    unsafe fn copy_tile_to_output(output: *mut u32, tile_buffer: &[u32], tile: &Tile) {
        let tile_min_x = tile.min.x as usize;
        let tile_min_y = tile.min.y as usize;

        let target_offset = tile_min_y * RENDER_WIDTH + tile_min_x;

        // copy tile back to main buffer
        for y in 0..TILE_HEIGHT {
            // get target output slice
            let output_line = unsafe {
                std::slice::from_raw_parts_mut(
                    output.add(target_offset + y * RENDER_WIDTH),
                    TILE_WIDTH,
                )
            };
            let tile_line = &tile_buffer[y * TILE_WIDTH..(y + 1) * TILE_WIDTH];
            output_line.copy_from_slice(tile_line);
        }
    }

    pub fn clear_all_single(&self, output: &mut [u32], tile_buffers: *mut u32) {
        let tile_len = TILE_WIDTH * TILE_HEIGHT;
        for tile in self.tiles.iter() {
            unsafe {
                let tile_offset = tile.local_tile_index * tile_len;
                let tile_buffer = Self::get_tile_buffer(tile_offset, tile_len, tile_buffers);
                SwRasterizer::clear_tile(tile_buffer);
                Self::copy_tile_to_output(output.as_mut_ptr(), tile_buffer, tile);
            }
        }
    }

    /*
    pub fn clear_all_multi(&self, output: &mut [u32], tile_buffers: &mut [u32]) {
        let output = output.as_mut_ptr() as u64;
        let tile_buffers = tile_buffers.as_mut_ptr() as u64;
        self.tiles.par_iter().for_each(|tile| {
            let output = output as *mut u32;
            let tile_buffers = tile_buffers as *mut u32;

            // Get the tile buffer as a slice
            let tile_buffer = unsafe {
                let len = TILE_WIDTH * TILE_HEIGHT;
                let offset = tile.local_tile_index * len;
                std::slice::from_raw_parts_mut(tile_buffers.add(offset), len)
            };

            unsafe {
                SwRasterizer::clear_tiles_single(tile_buffer, tile, output);
            }
        });
    }
    */
}

#[cfg(test)]
mod tests {
    use super::*;

    /*
    #[test]
    fn test_single_quad() {
        let vertex_quad_arry = [
            InternalVertex::new((0.0, 0.0), (0.0, 0.0), 0),
            InternalVertex::new((1.0, 0.0), (0.0, 0.0), 0),
            InternalVertex::new((1.0, 1.0), (0.0, 0.0), 0),
            InternalVertex::new((0.0, 1.0), (0.0, 0.0), 0),
        ];

        let quad_index_arry = [0, 1, 2, 0, 2, 3];
        let output = &mut [0u32; 1];

        let count = unsafe { cat_triangles(output, &vertex_quad_arry, &quad_index_arry) };

        assert_eq!(count, 1);
        assert_eq!((output[0] >> IS_QUAD) & 3, 2);
        assert_eq!((output[0] >> NON_TEXTURED) & 1, 1);
        assert_eq!((output[0] >> HAS_SAME_COLOR) & 1, 1)
    }

    #[test]
    fn test_triangle() {
        let vertex_quad_arry = [
            InternalVertex::new((0.0, 0.0), (0.0, 0.0), 0),
            InternalVertex::new((1.0, 0.0), (0.0, 0.0), 0),
            InternalVertex::new((1.0, 1.0), (0.0, 0.0), 0),

            InternalVertex::new((0.0, 5.0), (0.0, 0.0), 0),
            InternalVertex::new((2.0, 3.0), (0.0, 0.0), 0),
            InternalVertex::new((5.0, 5.0), (0.0, 0.0), 0),
        ];

        let quad_index_arry = [0, 1, 2, 0, 2, 3];
        let output = &mut [0u32; 1];

        let count = unsafe { cat_triangles(output, &vertex_quad_arry, &quad_index_arry) };

        assert_eq!(count, 1);
        assert_eq!((output[0] >> IS_QUAD) & 3, 0);
        assert_eq!((output[0] >> NON_TEXTURED) & 1, 1);
        assert_eq!((output[0] >> HAS_SAME_COLOR) & 1, 1)
    }

    #[test]
    fn test_quad_triangle() {
        let vertex_quad_arry = [
            InternalVertex::new((0.0, 0.0), (0.0, 0.0), 0),
            InternalVertex::new((1.0, 0.0), (0.0, 0.0), 0),
            InternalVertex::new((1.0, 1.0), (0.0, 0.0), 0),
            InternalVertex::new((0.0, 1.0), (0.0, 0.0), 0),

            InternalVertex::new((2.0, 5.0), (0.0, 0.0), 0),
            InternalVertex::new((2.0, 3.0), (0.0, 0.0), 0),
            InternalVertex::new((3.0, 7.0), (0.0, 0.0), 0),

            InternalVertex::new((2.0, 5.0), (0.0, 0.0), 0),
            InternalVertex::new((2.0, 3.0), (0.0, 0.0), 0),
            InternalVertex::new((1.0, 7.0), (0.0, 0.0), 0),
        ];

        let quad_index_arry = [0,1,2,0,2,3, 4,5,6, 7,8,9];
        let output = &mut [0u32; 2];

        let count = unsafe { cat_triangles(output, &vertex_quad_arry, &quad_index_arry) };

        assert_eq!(count, 2);
        assert_eq!((output[0] >> IS_QUAD) & 3, 2);
        assert_eq!((output[0] >> NON_TEXTURED) & 1, 1);
        assert_eq!((output[0] >> HAS_SAME_COLOR) & 1, 1);

        assert_eq!((output[1] >> IS_QUAD) & 3, 0);
        assert_eq!((output[1] >> NON_TEXTURED) & 1, 1);
        assert_eq!((output[1] >> HAS_SAME_COLOR) & 1, 1);
    }
    */
}
