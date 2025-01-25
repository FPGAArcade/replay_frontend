pub mod font;
mod image;
mod image_api;
pub mod input;
mod internal_error;
mod io_handler;
pub mod primitives;
pub mod render;
pub mod signal;
pub mod widgets;

use arena_allocator::Arena;
use background_worker::WorkSystem;
use clay_layout::{
    math::Dimensions, Clay, Clay_Dimensions,
    Clay_StringSlice, Clay_TextElementConfig, TypedConfig,
    render_commands::RenderCommand as ClayRenderCommand,
    color::Color as ClayColor,
    render_commands::RenderCommandConfig,
};
use fileorama::Fileorama;
use internal_error::InternalResult;
pub use io_handler::IoHandler;
use signal::Signal;
use std::cell::UnsafeCell;

use font::{CachedString, FontHandle};

pub use clay_layout::{
    elements::{rectangle::Rectangle, text::Text, CornerRadius},
    fixed,
    grow,
    id::Id,
    layout::{alignment::Alignment, padding::Padding, sizing::Sizing, Layout, LayoutDirection},
};

use flowi_renderer::{Renderer, RenderCommand, Color};

type FlowiKey = u64;

#[allow(dead_code)]
struct State<'a> {
    pub(crate) text_generator: font::TextGenerator,
    pub(crate) vfs: Fileorama,
    pub(crate) io_handler: IoHandler,
    pub(crate) input: input::Input,
    pub(crate) primitives: Arena,
    pub(crate) hot_item: FlowiKey,
    pub(crate) current_frame: u64,
    pub(crate) layout: Clay<'a>,
    pub(crate) button_id: u32,
    pub(crate) renderer: Box<dyn Renderer>,
    pub(crate) bg_worker: WorkSystem,
    pub(crate) active_font: FontHandle,
}

#[allow(dead_code)]
pub struct Ui<'a> {
    state: UnsafeCell<State<'a>>,
}

impl<'a> Ui<'a> {
    pub fn new(renderer: Box<dyn Renderer>) -> Box<Self> {
        let vfs = Fileorama::new(2);
        let io_handler = IoHandler::new(&vfs);
        crate::image_api::install_image_loader(&vfs);
        let bg_worker = WorkSystem::new(2);

        let reserve_size = 1024 * 1024 * 1024;
        let state = State {
            vfs,
            io_handler,
            text_generator: font::TextGenerator::new(&bg_worker),
            hot_item: 0,
            input: input::Input::new(),
            current_frame: 0,
            primitives: Arena::new(reserve_size).unwrap(),
            layout: Clay::new(Dimensions::new(1920.0, 1080.0)),
            button_id: 0,
            renderer,
            bg_worker,
            active_font: 0,
        };

        let data = Box::new(Ui {
            state: UnsafeCell::new(state),
        });

        // This is a hack. To be fixed later
        unsafe {
            let raw_ptr = Box::into_raw(data);
            clay_layout::Clay::set_measure_text_function_unsafe(
                Self::measure_text_trampoline,
                raw_ptr as usize,
            );
            Box::from_raw(raw_ptr)
        }
    }

    unsafe extern "C" fn measure_text_trampoline(
        text_slice: Clay_StringSlice,
        config: *mut Clay_TextElementConfig,
        user_data: usize,
    ) -> Clay_Dimensions {
        let text = core::str::from_utf8_unchecked(core::slice::from_raw_parts(
            text_slice.chars as *const u8,
            text_slice.length as _,
        ));

        let text_config = Text::from(*config);
        let ui = &*(user_data as *const Ui);

        ui.measure_text(text, &text_config).into()
    }

    fn measure_text(&self, text: &str, _config: &Text) -> Dimensions {
        let state = unsafe { &mut *self.state.get() };
        // TODO: Proper error handling
        let size = state
            .text_generator
            .measure_text_size(text, state.active_font)
            .unwrap();
        Dimensions::new(size.0 as _, size.1 as _)
    }

    pub fn begin(&mut self, _delta_time: f32, width: usize, height: usize) {
        let state = unsafe { &mut *self.state.get() };
        state
            .layout
            .layout_dimensions(Dimensions::new(width as f32, height as f32));
        state.layout.begin();
        state.io_handler.update();
        state.primitives.rewind();
        state.button_id = 0;
    }

    pub fn with_layout<F: FnOnce(&Ui), const N: usize>(
        &self,
        id: Option<&'a str>,
        configs: [TypedConfig; N],
        f: F,
    ) {
        let state = unsafe { &mut *self.state.get() };

        state.layout.with(id, configs, |_clay| {
            f(self);
        });
    }

    fn bounding_box(render_command: &ClayRenderCommand) -> [f32; 4] {
        let bounding_box = render_command.bounding_box;
        [
            bounding_box.x,
            bounding_box.y,
            bounding_box.x + bounding_box.width,
            bounding_box.y + bounding_box.height,
        ]
    }

    fn color(color: ClayColor) -> flowi_renderer::Color {
        flowi_renderer::Color { r: color.r, g: color.g, b: color.b, a: color.a }
    }

    fn translate_clay_render_commands(commands: impl Iterator<Item = ClayRenderCommand<'a>>) -> Vec<RenderCommand<'a>> { 
        // TODO: Arena
        let mut primitives = Vec::with_capacity(1024);
        for command in commands {
            match command.config {
                RenderCommandConfig::Rectangle(config) => {
                    let rect = RenderCommand::DrawRect {
                        bounding_box: Self::bounding_box(&command), 
                        color: Self::color(config.color), 
                    };

                    primitives.push(rect)
                }
                RenderCommandConfig::Text(text, config) => {
                    let text = RenderCommand::DrawText {
                        bounding_box: Self::bounding_box(&command),
                        text,
                        font_size: config.font_size,
                        font_handle: config.font_id as _,
                        color: Self::color(config.color),
                    };

                    primitives.push(text)
                }
                /*
                RenderCommandConfig::Image(image) => {
                    let image = RenderCommand::DrawImage {
                        bounding_box: Self::bounding_box(&command),
                        rounded_corners: [0.0, 0.0, 0.0, 0.0],
                        color: Self::color(image.color),
                        width: image.width,
                        height: image.height,
                        handle: image.handle as _,
                        rounding: false,
                    };

                    primitives.push(image)
                }
                */
                _ => { },
            }
        }

        Vec::new()
    }

    pub fn end(&mut self) {
        let state = unsafe { &mut *self.state.get() };

        // TODO: Fix me
        let primitives = Self::translate_clay_render_commands(state.layout.end());
        state.renderer.render(&primitives);

        // Generate primitives from all boxes
        //state.generate_primitives();
        state.current_frame += 1;
    }

    pub fn load_font(&mut self, path: &str, size: i32) -> InternalResult<FontHandle> {
        let state = unsafe { &mut *self.state.get() };
        state.text_generator.load_font(path, size, &state.bg_worker)
    }

    pub fn queue_generate_text(
        &mut self,
        text: &str,
        font_id: FontHandle,
    ) -> Option<font::CachedString> {
        let state = unsafe { &mut *self.state.get() };
        state
            .text_generator
            .queue_generate_text(text, font_id, &state.bg_worker)
    }

    #[rustfmt::skip]
    pub fn button(&self, text: &str) -> Signal {
        let state = unsafe { &mut *self.state.get() };

        state.layout.with_id_index(Some(("TestButton", state.button_id)), [
            Layout::new()
                .width(fixed!(160.0))
                .height(fixed!(40.0))
                .padding(Padding::all(8)).end(),
             Rectangle::new()
                .color(ClayColor::rgba(244.0, 200.0, 200.0, 255.0))
                .corner_radius(CornerRadius::All(8.0))
                .end()], |_ui|
            {
                let font_id = state.active_font;
                state.text_generator.queue_generate_text(text, font_id, &state.bg_worker);

                state.layout.text(text, Text::new()
                    .font_id(font_id as u16)
                    .font_size(16)
                    .color(ClayColor::rgba(255.0, 255.0, 255.0, 255.0))
                    .line_height(16)
                    .end());
            },
        );

        state.button_id += 1;

        Signal::new()
    }

    #[allow(dead_code)]
    fn signal(&mut self) -> Signal {
        Signal::new()
        /*
        let mut signal = Signal::new();
        let box_area = box_area.as_mut_unsafe();

        dbg!(&box_area.rect);

        if box_area.rect.contains(self.input.mouse_position) {
            signal.flags.insert(signal::SignalFlags::HOVERING);
        }

        signal
        */
    }

    pub fn renderer(&mut self) -> &Box<dyn Renderer> {
        let state = unsafe { &mut *self.state.get() };
        &state.renderer
    }

    pub fn update(&mut self) {
        let state = unsafe { &mut *self.state.get() };
        state.text_generator.update();
    }

    pub fn get_text(&self, text: &str, handle: FontHandle) -> Option<&CachedString> {
        let state = unsafe { &mut *self.state.get() };
        state.text_generator.get_text(text, handle)
    }

    /*
    pub fn input(&mut self) -> &mut input::Input {
        &mut self.input
    }

    pub fn primitives(&self) -> &[Primitive] {
        self.primitives.get_array_by_type::<Primitive>()
    }
    */
}

#[derive(Debug, Clone, Copy)]
pub struct ApplicationSettings {
    pub width: usize,
    pub height: usize,
}
