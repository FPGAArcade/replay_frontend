use crate::image::Image;
use flowi_core::imgui::FontAtlas;
use flowi_core::render::FlowiRenderer;
use flowi_core::renderer::Texture as CoreTexture;
use flowi_core::ApplicationSettings;
use raw_window_handle::RawWindowHandle;

pub struct SwRenderer {
    pub command_lists: Vec<RenderList>,
}

// Define the render commands
#[derive(Debug)]
enum RenderCommand {
    Triangle(usize),
    Quad(usize),
}

#[derive(Debug)]
struct RenderList {
    commands: Vec<RenderCommand>,
}

const IS_QUAD: u32 = 0x1 << 16;
const HAS_SAME_COLOR: u32 = 0x1 << 17;
const NON_TEXTURED: u32 = 0x1 << 18;

impl FlowiRenderer for SwRenderer {
    fn new(_settings: &ApplicationSettings, _window: Option<&RawWindowHandle>) -> Self {
        let _font_atlas = FontAtlas::build_r8_texture();

        Self {
            command_lists: Vec::new(),
        }
    }

    fn render(&mut self) {
        //let draw_data = DrawData::get_data();

        //let t = self.categorize_triangles(&draw_data);

        //dbg!(t);
    }

    fn get_texture(&mut self, _image: Image) -> CoreTexture {
        CoreTexture { handle: 0 }
    }
}

impl SwRenderer {
    /*
    fn categorize_triangles(&mut self, draw_data: &DrawData) -> Vec<RenderList> {
        let mut render_lists = Vec::new();

        for draw_list in draw_data.draw_lists() {
            let vertex_buffer = draw_list.vtx_buffer();
            let index_buffer = draw_list.idx_buffer();

            let mut render_commands = Vec::with_capacity(index_buffer.len() / 3);
            let mut index = 0;
            let total_indices = index_buffer.len();

            while index < total_indices - 3 {
                let i0 = index_buffer[index + 0] as usize;
                let i1 = index_buffer[index + 1] as usize;
                let i2 = index_buffer[index + 2] as usize;
                let i3 = index_buffer[index + 5] as usize;

                let v0 = vertex_buffer[i0];
                let v1 = vertex_buffer[i1];
                let v2 = vertex_buffer[i2];
                let v3 = vertex_buffer[i3];

                if v0.pos.x == v3.pos.x &&
                   v1.pos.x == v2.pos.x &&
                   v0.pos.y == v1.pos.y &&
                   v2.pos.y == v3.pos.y
                {
                    render_commands.push(RenderCommand::Quad(i0));
                    index += 6;
                    continue;
                }

                render_commands.push(RenderCommand::Triangle(i0));
                index += 3;
            }

            render_lists.push(RenderList {
                commands: render_commands,
            });
        }

        render_lists
    }
    */

    /*

    #[no_mangle]
    pub unsafe fn categorize_triangles(output: &mut [u32] , vertices: &[Vertex], indices: &[u16]) {
        let mut index = 0;
        let mut write_index = 0;
        let total_count = indices.len() - 3;

        // imgui uses a single uv coord for while pixel
        let white_u = 0.001f32;
        let white_v = 0.001f32;

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
                v0.u == white_u &&
                v0.v == white_v &&
                v3.u == white_u &&
                v3.v == white_v { 1 } else { 0 };

            let is_quad = if
                v0.x == v3.x &&
                v0.y == v1.y &&
                v1.x == v2.x &&
                v2.y == v3.y { 1 } else { 0 };

            let t = ((white_uv << NON_TEXTURED)
                | (same_color << HAS_SAME_COLOR)
                | (is_quad << IS_QUAD)
                | (index as u32)) as u32;

            *output.get_unchecked_mut(write_index) = t;
            index += if is_quad != 0 { 6 } else { 3 };
            write_index += 1;
        }
    }
    */
}
