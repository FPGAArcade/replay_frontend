use minifb::{Key, Window, WindowOptions};
use sw_rasterizer::{Vertex, SwRasterizer, Point};
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;
const TILE_WIDTH: usize = 32;
const TILE_HEIGHT: usize = 40;

fn draw_tile_grid(dest_buffer: &mut [u32]) {
    for i in 0..HEIGHT {
        for j in (0..WIDTH).step_by(TILE_WIDTH) {
            dest_buffer[i * WIDTH + j] = 0x00ff00ff;
        }

        if i % TILE_HEIGHT == 0 {
            for j in 0..WIDTH {
                dest_buffer[i * WIDTH + j] = 0x00ff00ff;
            }
        }
    }

    /*
    for j in 0..WIDTH {
        dest_buffer[NES_HEIGHT * SCREEN_SCALE * WIDTH + j] = 0x00ff00ff;
    }
    */
}

struct RenderPass {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    // TODO: add more data here
}

struct TempRenderData {
    render_passes: Vec<RenderPass>,
}

impl TempRenderData {
    fn new() -> Self {
        TempRenderData {
            render_passes: Vec::new(),
        }
    }

    pub fn read_data_from_file(&mut self, file: &str) -> Result<(), std::io::Error> {
        let mut file = std::fs::File::open(file)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        let mut data = Cursor::new(buffer);
        let passes = data.read_u32::<LittleEndian>()?;

        dbg!(passes);

        for _ in 0..passes {
            let vertices = data.read_u32::<LittleEndian>()?;
            let indices = data.read_u32::<LittleEndian>()?;

            dbg!(vertices);
            dbg!(indices);

            let mut vertex_data = Vec::with_capacity(vertices as usize);
            let mut index_data = Vec::with_capacity(indices as usize);

            for _i in 0..vertices {
                let pos_x = data.read_f32::<LittleEndian>()?;
                let pos_y = data.read_f32::<LittleEndian>()?;
                let uv_x = data.read_f32::<LittleEndian>()?;
                let uv_y = data.read_f32::<LittleEndian>()?;
                let color = data.read_u32::<LittleEndian>()?;

                let vertex = Vertex {
                    pos: Point { x: pos_x, y: pos_y },
                    uv: (uv_x, uv_y),
                    color,
                };

                vertex_data.push(vertex);
            }

            dbg!(vertex_data.len());

            for _ in 0..indices {
                index_data.push(data.read_u16::<LittleEndian>()?);
            }

            dbg!(index_data.len());

            self.render_passes.push(RenderPass {
                vertices: vertex_data,
                indices: index_data,
            });
        }

        Ok(())
    }
}


fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: false,
            ..WindowOptions::default()
        })
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let mut sw_raster = SwRasterizer::new(TILE_WIDTH, TILE_HEIGHT);

    let mut render_data = TempRenderData::new();
    render_data.read_data_from_file("test.bin").unwrap();

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in buffer.iter_mut() {
            *i = 0; // write something more funny here!
        }

        draw_tile_grid(&mut buffer);
            
        sw_raster.begin(render_data.render_passes.len());

        for pass in render_data.render_passes.iter() {
            sw_raster.add_vertices(&pass.vertices, &pass.indices);
        }

        unsafe { sw_raster.rasterize(&mut buffer) };

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
