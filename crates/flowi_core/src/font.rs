use crate::internal_error::{InternalError, InternalResult};
use background_worker::{AnySend, BoxAnySend, Receiver, WorkSystem, WorkerResult};
use cosmic_text::{Attrs, AttrsOwned, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache};
use flowi_renderer::RawVoidPtr;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub(crate) struct GeneratorConfig {
    font_handle: FontHandle,
    text: String,
    size: u32,
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
        *entry = (linear * 32767.0).round() as i16;
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
#[derive(Debug, Clone)]
pub struct CachedString {
    pub data: RawVoidPtr,
    pub id: u64,
    pub stride: u32,
    pub width: u32,
    pub height: u32,
    pub sub_pixel_step_x: u32,
    pub sub_pixel_step_y: u32,
}

type LoadedFonts = HashMap<FontHandle, FontInfo>;
type CachedStrings = HashMap<GeneratorConfig, CachedString>;

#[allow(dead_code)]
#[derive(Debug)]
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

#[derive(Clone, Debug)]
struct FontInfo {
    attrs: AttrsOwned,
    size: i32,
}

struct InflightGeneration {
    config: GeneratorConfig,
    receiver: Receiver<WorkerResult>,
}

#[allow(dead_code)]
pub(crate) struct TextGenerator {
    async_state: Arc<Mutex<AnySend>>,
    cached_strings: CachedStrings,
    /// These are for messure texts on the main thread.
    sync_font_system: FontSystem,
    sync_loaded_fonts: LoadedFonts,
    inflight_text_generations: Vec<InflightGeneration>,
    font_id_counter: u64,
    text_buffers_id: u64,
    load_font_async_id: usize,
    gen_text_async_id: usize,
}

pub(crate) struct LoadConfig {
    pub(crate) font_id: FontHandle,
    pub(crate) size: i32,
    pub(crate) font_path: Cow<'static, str>,
}

/// Loads a font into the font system and stores its information.
fn load_font(
    id: FontHandle,
    font_path: &str,
    font_size: i32,
    loaded_fonts: &mut LoadedFonts,
    font_system: &mut FontSystem,
) -> InternalResult<()> {
    let font_db = font_system.db_mut();

    // Load the font from the given path. This assumes that the path points to a valid font file.
    let ids = font_db.load_font_source(cosmic_text::fontdb::Source::File(font_path.into()));

    // Check if a font ID was obtained from loading the font.
    // If not, an error is returned since we can't proceed without an ID.
    let face_id = *ids.last().ok_or(InternalError::GenericError {
        text: format!("Font id not found for font {}", font_path),
    })?;

    // Retrieve the font face based on the ID.
    // If the face cannot be found, an error is returned.
    let face = font_db.face(face_id).ok_or(InternalError::GenericError {
        text: format!("Font face not found for font {}", font_path),
    })?;

    let family_name = face.families[0].0.as_str();

    let attrs = AttrsOwned::new(
        Attrs::new()
            .stretch(face.stretch)
            .style(face.style)
            .weight(face.weight)
            .family(cosmic_text::Family::Name(family_name)),
    );

    loaded_fonts.insert(
        id,
        FontInfo {
            attrs,
            size: font_size,
        },
    );
    Ok(())
}

fn measure_string_size(
    text: &str,
    font_info: &FontInfo,
    line_height: f32,
    font_system: &mut FontSystem,
) -> Option<(f32, f32)> {
    // Define metrics for the text
    let metrics = Metrics::new(font_info.size as _, line_height);

    // Create a buffer for the text
    let mut buffer = Buffer::new(font_system, metrics);

    // Set the text in the buffer with default attributes
    buffer.set_text(
        font_system,
        text,
        font_info.attrs.as_attrs(),
        Shaping::Advanced,
    );

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

#[allow(dead_code)]
fn generate_text(
    text: &str,
    font_info: &FontInfo,
    line_height: f32,
    state: &mut AsyncState,
) -> WorkerResult {
    // Define metrics for the text
    let metrics = Metrics::new(font_info.size as _, line_height);

    // Create a buffer for the text
    let mut buffer = Buffer::new(&mut state.font_system, metrics);

    // Set the text in the buffer with default attributes
    buffer.set_text(
        &mut state.font_system,
        text,
        font_info.attrs.as_attrs(),
        Shaping::Basic,
    );

    // Shape the text to compute layout without rendering
    buffer.shape_until_scroll(&mut state.font_system, true);

    // Get the layout runs which contain size information
    let layout_runs = buffer.layout_runs();

    // Calculate width and height; this assumes single line text for simplicity
    let mut width = 0.0f32;
    let mut height = 0.0f32;
    for run in layout_runs {
        width = width.max(run.line_w);
        height += run.line_height;
    }

    // + 8 as we always do 8 pixels wide in the rendering
    let width = (width + 8.0) as usize;
    let height = height as usize;

    let mut output = vec![0; width * height];

    // Create a default text color
    let text_color = Color::rgb(0xFF, 0xFF, 0xFF);
    let mut max_y_with_pixels = 0;

    // Draw the buffer (for performance, instead use SwashCache directly)
    buffer.draw(
        &mut state.font_system,
        &mut state.swash_cache,
        text_color,
        |x, y, _w, _h, color| {
            let c = (color.0 >> 24) as u8;
            if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
                return;
            }

            let pixel_value = state.srgb_to_linear[c as usize];
            max_y_with_pixels = if pixel_value != 0 {
                y
            } else {
                max_y_with_pixels
            };

            output[(y as usize * width + x as usize) as usize] = pixel_value;
        },
    );

    Ok(Box::new(CachedString {
        data: RawVoidPtr(Box::into_raw(output.into_boxed_slice()) as _),
        stride: width as u32,
        width: width as u32,
        //height: max_y_with_pixels as u32,
        height: height as u32,
        sub_pixel_step_x: 1,
        sub_pixel_step_y: 1,
        id: 0,
    }))
}

fn job_generate_text(data: BoxAnySend, state: Arc<Mutex<AnySend>>) -> WorkerResult {
    let data = data.downcast::<Box<GeneratorConfig>>().unwrap();
    let mut locked_state = state.lock().unwrap();
    let mut state = locked_state.downcast_mut::<AsyncState>().unwrap();

    if let Some(font) = state.loaded_fonts.get(&data.font_handle) {
        let font_clone = font.clone();
        generate_text(&data.text, &font_clone, font_clone.size as f32, &mut state)
    } else {
        panic!("Font not found");
    }
}

fn job_load_font(data: BoxAnySend, state: Arc<Mutex<AnySend>>) -> WorkerResult {
    let config = data.downcast::<Box<LoadConfig>>().unwrap();
    let locked_state = state.lock();
    let mut t = locked_state.unwrap();
    let state = t.downcast_mut::<AsyncState>().unwrap();

    load_font(
        config.font_id,
        &config.font_path,
        config.size,
        &mut state.loaded_fonts,
        &mut state.font_system,
    )
    .unwrap();

    // TODO: Error handling
    Ok(Box::new(()))
}

impl TextGenerator {
    pub(crate) fn new(bg_worker: &WorkSystem) -> Self {
        let async_state: Arc<Mutex<AnySend>> = Arc::new(Mutex::new(AsyncState::new()));

        let load_font_async_id =
            bg_worker.register_callback_with_state(job_load_font, async_state.clone());
        let gen_text_async_id =
            bg_worker.register_callback_with_state(job_generate_text, async_state.clone());

        Self {
            async_state,
            sync_font_system: FontSystem::new(),
            sync_loaded_fonts: HashMap::new(),
            font_id_counter: 0,
            cached_strings: HashMap::new(),
            load_font_async_id,
            gen_text_async_id,
            inflight_text_generations: Vec::new(),
            text_buffers_id: 1,
        }
    }

    pub fn load_font(
        &mut self,
        path: &str,
        font_size: i32,
        bg_worker: &WorkSystem,
    ) -> InternalResult<FontHandle> {
        let font_id = self.font_id_counter;
        // First we load the font sync so we know it loaded fine, if it's ok we
        // will also schedle it to be loaded async to be used for rendering later.
        // We load it on the main thread also for text measurement.
        load_font(
            font_id,
            path,
            font_size,
            &mut self.sync_loaded_fonts,
            &mut self.sync_font_system,
        )?;

        // Start loading the font async.
        bg_worker.add_work(
            self.load_font_async_id,
            Box::new(LoadConfig {
                font_id,
                size: font_size,
                font_path: Cow::Owned(path.to_string()),
            }),
        );

        Ok(font_id)
    }

    pub(crate) fn measure_text_size(
        &mut self,
        text: &str,
        font_id: FontHandle,
    ) -> Option<(f32, f32)> {
        if let Some(font_info) = self.sync_loaded_fonts.get(&font_id) {
            let font_size = font_info.size as f32;
            let line_height = font_size * 1.1;
            measure_string_size(text, font_info, line_height, &mut self.sync_font_system)
        } else {
            None
        }
    }

    pub fn queue_generate_text(
        &mut self,
        text: &str,
        size: u32,
        font_id: FontHandle,
        bg_worker: &WorkSystem,
    ) -> Option<CachedString> {
        let gen_config = GeneratorConfig {
            font_handle: font_id,
            text: text.to_string(),
            sub_pixel_steps_x: 1,
            sub_pixel_steps_y: 1,
            size,
        };

        // First check if we have the text cached.
        // TODO: Fix this. We should not clone because it will clone the whole text buffer.
        if let Some(cached_string) = self.cached_strings.get(&gen_config) {
            return Some(cached_string.clone());
        } else {
            // Queue the text generation if it's not cached.
            let inflight = InflightGeneration {
                config: gen_config.clone(),
                receiver: bg_worker.add_work(self.gen_text_async_id, Box::new(gen_config)),
            };

            self.inflight_text_generations.push(inflight);

            None
        }
    }

    pub fn update(&mut self) {
        let mut i = 0;
        while i < self.inflight_text_generations.len() {
            let inflight = &self.inflight_text_generations[i];
            if let Ok(data) = inflight.receiver.try_recv() {
                match data {
                    Ok(mut data) => {
                        let data = data.downcast_mut::<CachedString>().unwrap();
                        data.id = self.text_buffers_id;
                        self.cached_strings
                            .insert(inflight.config.clone(), data.clone());
                        self.inflight_text_generations.remove(i);
                        self.text_buffers_id += 1;
                    }

                    Err(e) => {
                        println!("Error generating text: {:?}", e);
                        i += 1;
                    }
                }
            }
        }
    }

    pub fn get_text(&self, text: &str, size: u32, font_id: FontHandle) -> Option<&CachedString> {
        let gen_config = GeneratorConfig {
            font_handle: font_id,
            text: text.to_string(),
            sub_pixel_steps_x: 1,
            sub_pixel_steps_y: 1,
            size,
        };

        //dbg!("{}", &gen_config);

        self.cached_strings.get(&gen_config).map(|s| &*s)
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
        assert_eq!(table[255], 32767);
        assert_eq!(table[128], 7073);
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
        let worker = WorkSystem::new(2);
        let mut state = TextGenerator::new(&worker);
        let font_id = state
            .load_font("../../data/fonts/roboto/Roboto-Regular.ttf", 56, &worker)
            .unwrap();

        let text = "Hello, World!";
        let size = state.measure_text_size(text, font_id).unwrap();
        let size = (size.0.floor(), size.1.floor());
        assert_eq!(size, (313.0, 61.0));
    }
}
