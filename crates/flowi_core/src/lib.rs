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
use fileorama::Fileorama;
pub use io_handler::IoHandler;
use primitives::Primitive;
use signal::Signal;
use clay_layout::{
    TypedConfig,
    math::Dimensions,
    Clay,
};

use font::FontHandle;

pub use clay_layout::layout::{
    LayoutDirection,
    alignment::Alignment,
    sizing::Sizing,
    padding::Padding,
};
    
pub use clay_layout::elements::rectangle::Rectangle;

type FlowiKey = u64;

#[allow(dead_code)]
pub struct Ui<'a> {
    pub(crate) text_generator: font::TextGenerator, 
    pub(crate) vfs: Fileorama,
    pub(crate) io_handler: IoHandler,
    pub(crate) input: input::Input,
    pub(crate) primitives: Arena,
    pub(crate) hot_item: FlowiKey,
    pub(crate) current_frame: u64,
    pub(crate) layout: Clay<'a>,
}

impl<'a> Ui<'a> {
    pub fn new() -> Box<Self> {
        let vfs = Fileorama::new(2);
        let io_handler = IoHandler::new(&vfs);
        crate::image_api::install_image_loader(&vfs);

        let reserve_size = 1024 * 1024 * 1024;

        Box::new(Ui {
            vfs,
            io_handler,
            text_generator: font::TextGenerator::new(),
            hot_item: 0,
            input: input::Input::new(),
            current_frame: 0,
            primitives: Arena::new(reserve_size).unwrap(),
            layout: Clay::new(Dimensions::new(1920.0, 1080.0)),
        })
    }

    pub fn begin(&mut self, _delta_time: f32, width: usize, height: usize) {
        self.layout.layout_dimensions(Dimensions::new(width as f32, height as f32));
        self.io_handler.update();
        self.primitives.rewind();
    }

    pub fn with_layout<F: FnOnce(&Ui), const N: usize>(
        &self,
        configs: [TypedConfig; N],
        f: F,
    ) {
        self.layout.with(configs, |_clay| {
            f(self);
        });
    }

    pub fn end(&mut self) {
        // Generate primitives from all boxes
        self.generate_primitives();
        self.current_frame += 1;
    }

    fn generate_primitives(&mut self) {}

    pub fn load_font(&mut self, path: &str, size: i32) -> FontHandle {
        self.text_generator.load_font_async(path, size)
    }

    pub fn button(&mut self, _text: &str) -> Signal {
        /*
        let box_area = self.create_box_with_string(text);
        self.signal(box_area)
        */
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

    pub fn input(&mut self) -> &mut input::Input {
        &mut self.input
    }

    pub fn primitives(&self) -> &[Primitive] {
        self.primitives.get_array_by_type::<Primitive>()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ApplicationSettings {
    pub width: usize,
    pub height: usize,
}
