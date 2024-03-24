use rayon::prelude::*;

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

#[derive(Debug, Clone, Copy, Default)]
pub struct InternalVertex {
    pub pos: Point,
    pub uv: Uv,
    pub color: u32
}

impl InternalVertex {
    pub fn new(pos: Point, uv: Uv, color: u32) -> Self {
        InternalVertex {
            pos,
            uv,
            color
        }
    }
}

// Define the type `Vertex` based on whether the `imgui` feature is enabled or not.
#[cfg(feature = "imgui")]
pub type Vertex = imgui::DrawVert;

#[cfg(not(feature = "imgui"))]
pub type Vertex = InternalVertex;

const RENDER_WIDTH: usize = 1920;
const RENDER_HEIGHT: usize = 1080;
const TILE_PRIM_PREALLOC: usize = 32 * 1024;
pub const TILE_WIDTH: usize = 128;
pub const TILE_HEIGHT: usize = 120;

// This macro measures the execution time of the given block and prints it with the specified name.
#[macro_export]
macro_rules! measure_time {
    ($name:expr, $block:block) => {
        // Check if the "timing_enabled" feature is active
        #[cfg(feature = "timing_enabled")]
        {
            let start = std::time::Instant::now();
            let result = $block;
            let duration = start.elapsed();
            println!("{} took: {} microseconds", $name, duration.as_micros());
            result
        }
        // If "timing_enabled" feature is not active, just execute the block without measuring
        #[cfg(not(feature = "timing_enabled"))]
        {
            $block
        }
    };
}

#[derive(Debug, Clone, Copy)]
struct TilePosition {
    x: i16,
    y: i16,
}


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

#[derive(Debug)]
struct QuadFlat {
    vertices: (Point, Point),
    color: u32,
}

#[derive(Debug)]
struct QuadTextured {
    vertices: (Point, Point),
    uv: (Uv, Uv),
    texture: u32,
    color: u32,
}

#[derive(Debug)]
enum RenderPrimitive {
    TriangleFlat(TriangleFlat),
    TriangleTextured(TriangleTextured),
    QuadFlat(QuadFlat),
    QuadTextured(QuadTextured),
}

#[derive(Clone, Default)]
pub struct CommandBuffer<'a> {
    pub commands: Vec<u32>,
    vertices: Option<&'a [Vertex]>,
    index_buffer: Option<&'a [u16]>,
    count: usize,
}

#[derive(Debug, Clone)]
struct TilePrimitives<'a> {
    vertices: Option<&'a [Vertex]>,
    index_buffer: Option<&'a [u16]>,
    primitives: Vec<u32>,
    primitive_count: usize,
}

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

pub unsafe fn cat_triangles(output: &mut [u32] , vertices: &[Vertex], indices: &[u16]) -> usize {
    let mut index = 0;
    let mut write_index = 0;
    let total_count = indices.len() - 3;

    while index < total_count {
        let i0 = *indices.get_unchecked(index + 0);
        let i1 = *indices.get_unchecked(index + 1);
        let i2 = *indices.get_unchecked(index + 2);
        let i3 = *indices.get_unchecked(index + 5);

        let v0 = *vertices.get_unchecked(i0 as usize);
        let v1 = *vertices.get_unchecked(i1 as usize);
        let v2 = *vertices.get_unchecked(i2 as usize);
        let v3 = *vertices.get_unchecked(i3 as usize);

        let same_color = if  
            v0.color == v1.color &&
            v0.color == v2.color &&
            v0.color == v3.color { 1 } else { 0 };

        // not sure if equal compare will be ok here, must verify how imgui *exactly* calculate this value
        // We short cut this a bit given if two verts has this value, we assume the rest has it
        let white_uv = if 
            v0.uv.u == v1.uv.u &&
            v0.uv.v == v1.uv.v &&
            v2.uv.u == v2.uv.u &&
            v2.uv.v == v3.uv.v { 1 } else { 0 };

        let is_quad = if 
            v0.pos.x == v3.pos.x &&
            v1.pos.x == v2.pos.x && 
            v0.pos.y == v1.pos.y &&
            v2.pos.y == v3.pos.y { 2 } else { 0 };

        let t = ((white_uv << NON_TEXTURED) 
            | (same_color << HAS_SAME_COLOR) 
            | (is_quad << IS_QUAD) 
            | (index as u32)) as u32;

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

    for (i, command_buffer) in commands.iter().enumerate() {

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
            
            let i0 = indices[index + 0] as usize;
            let i3 = indices[index + 2] as usize;

            let v0 = vertices[i0];
            let v3 = vertices[i3];

            let prim_pos_min = v0.pos; 
            let prim_pos_max = v3.pos; 

            // skip if the primitive is fully outside the tile, but keep if it's partially inside
            // and fully inside
            
            if (prim_pos_max.x < tile_min_x || prim_pos_min.x > tile_max_x) || 
                prim_pos_max.y < tile_min_y || prim_pos_min.y > tile_max_y 
            {
                continue;
            }

            if not_textured == 1 {
                let prim = RenderPrimitive::QuadFlat(QuadFlat {
                    vertices: (prim_pos_min, prim_pos_max),
                    color: v0.color,
                });

                tile.primitives.push(prim);
            } 
            else {
                let prim = RenderPrimitive::QuadTextured(QuadTextured {
                    vertices: (prim_pos_min, prim_pos_max),
                    uv: (v0.uv, v3.uv), 
                    texture: 0,
                    color: v0.color,
                });

                tile.primitives.push(prim);
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
            }
            _ => {}
        }
    }
}

struct RenderDataTempInput8bit {
    texture0: Vec<u32>,
    texture1: Vec<u32>,
    texture2: Vec<u32>,
}

struct RenderDataTemp8bit<'a> {
    texture0: &'a [u32], 
    texture1: &'a [u32], 
    texture2: &'a [u32],
}

impl<'a> SwRasterizer<'a> {
    pub fn new(tile_width: usize, tile_height: usize) -> Self {
        let mut tiles = Vec::with_capacity((RENDER_WIDTH / tile_width) * (RENDER_HEIGHT / tile_height));
        let mut tile_index = 0;

        for y in (0..RENDER_HEIGHT).step_by(tile_height) {
            for x in (0..RENDER_WIDTH).step_by(tile_width) {
                tiles.push(Tile {
                    primitives: Vec::new(),
                    min: TilePosition { x: x as i16, y: y as i16 },
                    max: TilePosition { x: (x + tile_width) as i16, y: (y + tile_height) as i16 },
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

    pub unsafe fn rasterize(&mut self, output: &mut [u32]) {
        // process the command buffers
        let mut tile_buffer = vec![0u32; self.tile_width * self.tile_height];
        
        measure_time!("categorize triangles", {
            for i in 0..self.index {
                let command_buffer = &mut self.commands[i];
                let vertices = command_buffer.vertices.unwrap_unchecked();
                let indices = command_buffer.index_buffer.unwrap_unchecked();
                let count = cat_triangles(&mut command_buffer.commands, vertices, indices);

                command_buffer.count = count;
            }
        });

        for tile in self.tiles.iter_mut() {
            bin_primitives(tile, &self.commands);
        }

        for tile in self.tiles.iter_mut() {
            rasterizer_tile(&mut tile_buffer, tile, output);
        }
    }

    fn get_tile_buffer(tile: &Tile, tile_buffers: *mut u32) -> &mut [u32] {
        let len = TILE_WIDTH * TILE_HEIGHT;
        let offset = tile.local_tile_index * len;
        unsafe {
            std::slice::from_raw_parts_mut(tile_buffers.add(offset), len)
        }
    }

    unsafe fn render_to_tile_8bit(tile_buffer: &mut [u32], tile: &Tile, render_data: &RenderDataTemp8bit) {
        let tile_min_x = tile.min.x as usize;
        let tile_min_y = tile.min.y as usize;

        let target_offset = tile_min_y * RENDER_WIDTH + tile_min_x;

        // Copy texture t tile
        for y in 0..TILE_HEIGHT {
            let tile_line = &mut tile_buffer[y * TILE_WIDTH..(y + 1) * TILE_WIDTH];
            let texture_line = &render_data.texture0[target_offset + y * RENDER_WIDTH..(y + 1) * RENDER_WIDTH];
            tile_line.copy_from_slice(texture_line);
        }
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
            let output_line = unsafe { std::slice::from_raw_parts_mut(output.add(target_offset + y * RENDER_WIDTH), TILE_WIDTH) }; 
            let tile_line = &tile_buffer[y * TILE_WIDTH..(y + 1) * TILE_WIDTH];
            output_line.copy_from_slice(tile_line);
        }
    }

    pub fn clear_all_single(&self, output: &mut [u32], tile_buffers: *mut u32) {
        for tile in self.tiles.iter() {
            unsafe {
                let tile_buffer = Self::get_tile_buffer(tile, tile_buffers);
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

pub fn sol_copy_to_buffer(dest: *mut u32, src: &[u32], offset: usize) {
    let slice_size = 1920 * (1080 / 4);
    let slice_start = offset * slice_size;
    let slice_end = slice_start + slice_size;
    let src = &src[slice_start..slice_end];
    let offset = offset * slice_size;
    let dest = unsafe { dest.add(offset) };

    unsafe {
        std::ptr::copy_nonoverlapping(src.as_ptr(), dest, src.len());
    }
}

pub fn copy_single_threaded(dest: *mut u32, src: &[u32]) {
    sol_copy_to_buffer(dest, src, 0);
    sol_copy_to_buffer(dest, src, 1);
    sol_copy_to_buffer(dest, src, 2);
    sol_copy_to_buffer(dest, src, 3);
}

fn copy_tiled(dest: &mut [u32], src: &[u32], offset: usize) {
    let slice_size = 1920 * (1080 / 4);
    let slice_start = offset * slice_size;
    let slice_end = slice_start + slice_size;
    let src = &src[slice_start..slice_end];
    let offset = offset * slice_size;

    dest[offset..offset + slice_size].copy_from_slice(src);
}

// Copy multi threaded using rayon using a parallel iterator
pub fn copy_multi_threaded(dest_in: *mut u32, src: &[u32]) {
    let dest_thread = dest_in as u64; 
    (0..4).into_par_iter().for_each(|i| {
        let dest = dest_thread as *mut u32;
        sol_copy_to_buffer(dest, src, i);
    });
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
