//use cosmic_text::{Attrs, Color, FontSystem, SwashCache, Buffer, Metrics, Shaping};
use cosmic_text::{FontSystem, SwashCache};
use std::collections::HashMap;
use background_worker::BoxAnySend;
use std::sync::{Arc, Mutex};
use std::any::Any;

#[derive(Debug)]
pub(crate) struct Glyph {
    pub(crate) character: char,        // The character represented by the glyph
    pub(crate) atlas_position: (u32, u32),  // Top-left position in the texture atlas (in pixels)
    pub(crate) atlas_size: (u32, u32),      // Width and height in the texture atlas (in pixels)
    pub(crate) advance: f32,                // Advance width for positioning the next glyph
    pub(crate) offset: (f32, f32),          // Offset for rendering the glyph
}

pub(crate) type KerningMap = HashMap<(char, char), f32>;

struct FontAtlas {
    texture: Vec<u8>,
    texture_width: u32,
    texture_height: u32,
    kerning_map: KerningMap,
    glyphs: HashMap<char, Glyph>,
}

struct TextGenerator {
    font_system: FontSystem,
    swash_cache: SwashCache,
    srgb_to_linear: [i16; 256],
}

#[derive(Debug, Hash)]
enum TextureFormat {
    I16,
    RGBA16, // sub-pixel based
}

//enum ColorSpace {
//    Linear,
//    Srgb,
//}

#[derive(Debug, Hash)]
struct GeneratorConfig {
    font_path: String,
    font_name: String,
    texture_format: TextureFormat,
}

fn srgb_to_linear(srgb: f32) -> f32 {
    if srgb <= 0.04045 {
        srgb / 12.92
    } else {
        ((srgb + 0.055) / 1.055).powf(2.4)
    }
}

fn build_srgb_to_linear_table() -> [i16; 256] {
    let mut table = [0; 256];

    for (i, entry) in table.iter_mut().enumerate().take(256) {
        let srgb = i as f32 / 255.0;
        let linear = srgb_to_linear(srgb);
        *entry = (linear * 255.0).round() as i16;
    }

    table
}

struct FontInfo {
    font_id: u32,
    font_name: String,
    font_path: String,
}

fn load_sync(data: &GeneratorConfig, state: &mut TextGenerator) {
    let font_db = state.font_system.db_mut();
    let ids = font_db.load_font_source(cosmic_text::fontdb::Source::File(data.font_path.clone().into()));

    // TODO: it is assumed this is the right font face
    let face_id = *ids.last().unwrap();
    let face = font_db.face(face_id).unwrap();
    let family_name = face.families[0].0.as_str();

    dbg!(&family_name);

    /*
    // Text metrics indicate the font size and line height of a buffer
    let metrics = Metrics::new(60.0 * FONT_SUB_STEPS as f32, 58.0 * FONT_SUB_STEPS as f32);

    // A Buffer provides shaping and layout for a UTF-8 string, create one per text widget
    let mut buffer = Buffer::new(&mut font_system, metrics);

    // Set a size for the text buffer, in pixels
    buffer.set_size(&mut font_system, f32::MAX, f32::MAX);

    // Attributes indicate what font to choose
    let attrs = Attrs::new();

    buffer.set_text(&mut font_system, "Replay²", attrs, Shaping::Advanced);

    // Perform shaping as desired
    buffer.shape_until_scroll(&mut font_system, true);

    let line = buffer.layout_runs().next().unwrap();
    let width = line.line_w as usize;
    let height = (line.line_y + 54.0) as usize;

    let mut output = vec![0; width * height];

    // Create a default text color
    let text_color = Color::rgb(0xFF, 0xFF, 0xFF);

    // Draw the buffer (for performance, instead use SwashCache directly)
    buffer.draw(&mut font_system, &mut swash_cache, text_color, |x, y, _w, _h, color| {
        let c = (color.0 >> 24) as u8;
        if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
            return;
        }
        output[(y as usize * width + x as usize) as usize] = c;
    });
    */
}

fn job_generate_text(data: BoxAnySend, state: Arc<Mutex<dyn Any + Send>>) {
    let mut locked_state = state.lock().unwrap();
    let state = locked_state.downcast_mut::<TextGenerator>().unwrap();
    let data = data.downcast::<GeneratorConfig>().unwrap();
    load_sync(&data, state);
}

impl TextGenerator {
    pub fn new() -> Self {
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        let srgb_to_linear = build_srgb_to_linear_table();

        Self {
            srgb_to_linear,
            font_system,
            swash_cache,
        }
    }

    pub fn generate_text(&mut self, config: GeneratorConfig) {
        /*
        let state = Arc::new(Mutex::new(self));
        let _id = background_worker::add_work(job_generate_text, Box::new(config), state);
        */
    }

    pub fn update(&mut self) {
    }
}


