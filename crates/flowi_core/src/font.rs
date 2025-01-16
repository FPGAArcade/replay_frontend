//use cosmic_text::{Attrs, Color, FontSystem, SwashCache, Buffer, Metrics, Shaping};
use crate::internal_error::{InternalError, InternalResult};
//use background_worker::BoxAnySend;
use cosmic_text::{Attrs, AttrsOwned, FontSystem, SwashCache, Color, Shaping, Metrics, Buffer};
//use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Hash, Eq, PartialEq)]
pub(crate) struct GeneratorConfig {
    font_path: String,
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

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct FontFaceInfo {
    stretch: cosmic_text::fontdb::Stretch,
    style: cosmic_text::fontdb::Style,
    weight: cosmic_text::fontdb::Weight,
    family_name: String,
    font_id: u32,
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

type LoadedFonts = HashMap<String, FontFaceInfo>;
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

#[allow(dead_code)]
pub(crate) struct TextGenerator {
    async_state: Arc<Mutex<AsyncState>>,
    cached_strings: CachedStrings, 
}

#[allow(dead_code)]
fn get_attrs_from_font(font: &FontFaceInfo) -> InternalResult<cosmic_text::AttrsOwned> {
    Ok(AttrsOwned::new(
        Attrs::new()
            .stretch(font.stretch)
            .style(font.style)
            .weight(font.weight)
            .family(cosmic_text::Family::Name(font.family_name.as_str())),
    )) // TODO: Fix alloc
}

#[allow(dead_code)]
fn get_or_load_font(
    font_path: &str,
    loaded_fonts: &mut LoadedFonts,
    font_system: &mut FontSystem,
) -> InternalResult<cosmic_text::AttrsOwned> {
    if let Some(font) = loaded_fonts.get(font_path) {
        get_attrs_from_font(font)
    } else {
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

        let font = FontFaceInfo {
            stretch: face.stretch,
            style: face.style,
            weight: face.weight,
            family_name: family_name.to_owned(),
            font_id: 0,
        };

        loaded_fonts.insert(font_path.to_string(), font.clone());

        get_attrs_from_font(&font)
    }
}

#[allow(dead_code)]
fn load_sync(data: &GeneratorConfig, state: &mut AsyncState) -> InternalResult<CachedString> {
    let attrs = get_or_load_font(
        &data.font_path,
        &mut state.loaded_fonts,
        &mut state.font_system,
    )?;

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
            cached_strings: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn generate_text_sync(&mut self, config: &GeneratorConfig) -> InternalResult<CachedString> {
        load_sync(config, &mut self.async_state.lock().unwrap())
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
}
