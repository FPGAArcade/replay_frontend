//use cosmic_text::{Attrs, Color, FontSystem, SwashCache, Buffer, Metrics, Shaping};
use crate::internal_error::{InternalError, InternalResult};
use std::borrow::Cow;
//use background_worker::BoxAnySend;
use cosmic_text::{Attrs, AttrsOwned, FontSystem, SwashCache, Shaping, Metrics, Buffer};
//use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Hash, Eq, PartialEq)]
pub(crate) struct GeneratorConfig {
    font_handle: FontHandle,
    text: String,
    font_size: u32,
    sub_pixel_steps_x: u32,
    sub_pixel_steps_y: u32,
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

pub type FontHandle = u64;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct FontFaceInfo {
    stretch: cosmic_text::fontdb::Stretch,
    style: cosmic_text::fontdb::Style,
    weight: cosmic_text::fontdb::Weight,
    family_name: String,
}

/// A cached string is a pre-rendered string that can be drawn to the screen
#[allow(dead_code)]
pub(crate) struct CachedString {
    data: Vec<i16>,
    stride: u32,
    width: u32,
    height: u32,
    sub_pixel_step_x: u32,
    sub_pixel_step_y: u32,
}

type LoadedFonts = HashMap<FontHandle, FontInfo>;
type CachedStrings = HashMap<GeneratorConfig, CachedString>;

#[allow(dead_code)]
struct AsyncState {
    loaded_fonts: LoadedFonts,
    font_system: FontSystem,
    swash_cache: SwashCache,
    srgb_to_linear: [i16; 256],
}

impl AsyncState {
    fn new() -> Self {
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        let srgb_to_linear = build_srgb_to_linear_table();

        Self {
            font_system,
            swash_cache,
            srgb_to_linear,
            loaded_fonts: HashMap::new(),
        }
    }
}

struct FontInfo {
    attrs: AttrsOwned,
    size: i32,
}

#[allow(dead_code)]
pub(crate) struct TextGenerator {
    async_state: Arc<Mutex<AsyncState>>,
    cached_strings: CachedStrings, 
    /// These are for messure texts on the main thread.
    sync_font_system: FontSystem,
    sync_loaded_fonts: LoadedFonts,
    font_id_counter: u64,
}

pub(crate) struct LoadConfig {
    pub(crate) font_id: FontHandle,
    pub(crate) font_path: Cow<'static, str>,
}

#[allow(dead_code)]
fn load_font(
    id: FontHandle,
    font_path: &str,
    font_size: i32,
    loaded_fonts: &mut LoadedFonts,
    font_system: &mut FontSystem,
) -> InternalResult<()> {
    let font_db = font_system.db_mut();
    let ids = font_db.load_font_source(cosmic_text::fontdb::Source::File(font_path.into()));

    // TODO: it is assumed this is the right font face
    let face_id = *ids.last().ok_or(InternalError::GenericError {
        text: "No font_id found".to_owned(),
    })?;
    let face = font_db.face(face_id).ok_or(InternalError::GenericError {
        text: "No font face found".to_owned(),
    })?;
    let family_name = face.families[0].0.as_str();

    let attrs = AttrsOwned::new(
        Attrs::new()
            .stretch(face.stretch)
            .style(face.style)
            .weight(face.weight)
            .family(cosmic_text::Family::Name(family_name)));

    loaded_fonts.insert(id, FontInfo { attrs, size: font_size }); 
    Ok(())
}

fn measure_string_size(text: &str, font_info: &FontInfo, line_height: f32, font_system: &mut FontSystem) -> Option<(f32, f32)> {
    // Define metrics for the text
    let metrics = Metrics::new(font_info.size as _, line_height);

    // Create a buffer for the text
    let mut buffer = Buffer::new(font_system, metrics);

    // Set the text in the buffer with default attributes
    buffer.set_text(font_system, text, font_info.attrs.as_attrs(), Shaping::Basic);

    // Shape the text to compute layout without rendering
    buffer.shape_until_scroll(font_system, true);

    // Get the layout runs which contain size information
    let layout_runs = buffer.layout_runs();
    
    // Calculate width and height; this assumes single line text for simplicity
    let mut width = 0.0f32;
    let mut height = 0.0f32;
    for run in layout_runs {
        width = width.max(run.line_w);
        height += run.line_height;
    }

    Some((width, height))
}

/*
#[allow(dead_code)]
fn load_async(data: &LoadConfig, state: &mut AsyncState) -> InternalResult<()> {
    load_font(data.font_id, &data.font_path, &mut state.loaded_fonts, &mut state.font_system)
}
*/

/*
#[allow(dead_code)]
fn generate_text(data: &GeneratorConfig, state: &mut AsyncState) -> InternalResult<CachedString> {
    let font_size = data.font_size;

    // Text metrics indicate the font size and line height of a buffer
    let metrics = Metrics::new(
        (font_size * data.sub_pixel_steps_x) as f32, 
        (font_size * data.sub_pixel_steps_y) as f32);

    // A Buffer provides shaping and layout for a UTF-8 string, create one per text widget
    let mut buffer = Buffer::new(&mut state.font_system, metrics);

    // Set a size for the text buffer, in pixels
    //buffer.set_size(&mut font_system, f32::MAX, f32::MAX);

    buffer.set_text(&mut state.font_system, &data.text, attrs.as_attrs(), Shaping::Advanced);

    // Perform shaping as desired
    buffer.shape_until_scroll(&mut state.font_system, true);

    let line = buffer.layout_runs().next().unwrap();
    let width = line.line_w as usize;
    let height = (line.line_y + 54.0) as usize;

    let mut output = vec![0; width * height];

    // Create a default text color
    let text_color = Color::rgb(0xFF, 0xFF, 0xFF);

    // Draw the buffer (for performance, instead use SwashCache directly)
    buffer.draw(&mut state.font_system, &mut state.swash_cache, text_color, |x, y, _w, _h, color| {
        let c = (color.0 >> 24) as u8;
        if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
            return;
        }

        output[(y as usize * width + x as usize) as usize] = state.srgb_to_linear[c as usize];
    });

    Ok(CachedString {
        data: output,
        stride: width as u32,
        width: width as u32,
        height: height as u32,
        sub_pixel_step_x: data.sub_pixel_steps_x,
        sub_pixel_step_y: data.sub_pixel_steps_y,
    })
}
*/

/*
fn job_generate_text(data: BoxAnySend, state: Arc<Mutex<dyn Any + Send>>) {
    let mut locked_state = state.lock().unwrap();
    let state = locked_state.downcast_mut::<AsyncState>().unwrap();
    let data = data.downcast::<GeneratorConfig>().unwrap();
    load_sync(&data, state);
}
*/

impl TextGenerator {
    pub(crate) fn new() -> Self {
        let async_state = Arc::new(Mutex::new(AsyncState::new()));

        Self {
            async_state,
            sync_font_system: FontSystem::new(),
            sync_loaded_fonts: HashMap::new(),
            font_id_counter: 0,
            cached_strings: HashMap::new(),
        }
    }

    /*
    #[allow(dead_code)]
    pub(crate) fn generate_text_sync(&mut self, config: &GeneratorConfig) -> InternalResult<CachedString> {
        //load_sync_async(config, &mut self.async_state.lock().unwrap())
    }
    */

    pub fn load_font_async(&mut self, path: &str, font_size: i32) -> FontHandle {
        let font_id = self.font_id_counter;
        load_font(font_id, path, font_size, &mut self.sync_loaded_fonts, &mut self.sync_font_system).unwrap();
        self.font_id_counter += 1;
        font_id
    }

    pub(crate) fn messure_text_size(&mut self, text: &str, load_config: &LoadConfig) -> Option<(f32, f32)> { 
        if let Some(font_info) = self.sync_loaded_fonts.get(&load_config.font_id) {
            let font_size = font_info.size as f32;
            let line_height = font_size * 1.5;
            measure_string_size(text, font_info, line_height, &mut self.sync_font_system) 
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srgb_to_linear() {
        assert_eq!(srgb_to_linear(0.0), 0.0);
        assert_eq!(srgb_to_linear(1.0), 1.0);
        assert_eq!(srgb_to_linear(0.5), 0.21404114048207108);
    }

    #[test]
    fn test_build_srgb_to_linear_table() {
        let table = build_srgb_to_linear_table();
        assert_eq!(table[0], 0);
        assert_eq!(table[255], 255);
        assert_eq!(table[128], 55);
    }

    /*
    #[test]
    fn test_load_sync() {
        let state = TextGenerator::new();
        let config = GeneratorConfig {
            font_path: "../../data/fonts/roboto/Roboto-Regular.ttf".to_string(),
            font_size: 56,
            text: "Hello, World!".to_string(),
            sub_pixel_steps_x: 1,
            sub_pixel_steps_y: 1,
        };

        let _res = load_sync(&config, &mut state.async_state.lock().unwrap()).unwrap();


        let config = GeneratorConfig {
            font_path: "../../data/fonts/roboto/Roboto-Bold.ttf".to_string(),
            font_size: 56,
            text: "Hello, World!".to_string(),
            sub_pixel_steps_x: 1,
            sub_pixel_steps_y: 1,
        };

        let _res = load_sync(&config, &mut state.async_state.lock().unwrap()).unwrap();
    }
    */
    #[test]
    fn test_load_sync() {
        let mut state = TextGenerator::new();
        let font_id = state.load_font_async("../../data/fonts/roboto/Roboto-Regular.ttf", 56);
        let load_config = LoadConfig {
            font_id,
            font_path: Cow::Borrowed("../../data/fonts/roboto/Roboto-Regular.ttf"),
        };

        let text = "Hello, World!";
        let size = state.messure_text_size(text, &load_config).unwrap();
        assert_eq!(size, (313.71484, 84.0));
    }
}
