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
use clay_layout::{color::Color, math::Dimensions, Clay, TypedConfig};
use fileorama::Fileorama;
pub use io_handler::IoHandler;
use primitives::Primitive;
use signal::Signal;
use std::cell::UnsafeCell;

use font::FontHandle;

pub use clay_layout::{
    //color::Color,
    elements::{rectangle::Rectangle, CornerRadius},
    fixed,
    grow,
    id::Id,
    layout::{alignment::Alignment, padding::Padding, sizing::Sizing, Layout, LayoutDirection},
    render_commands::{RenderCommandType, RenderCommandConfig, RenderCommand},
};

pub use render::FlowiRenderer as Renderer;

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

        let reserve_size = 1024 * 1024 * 1024;
        let state = State {
            vfs,
            io_handler,
            text_generator: font::TextGenerator::new(),
            hot_item: 0,
            input: input::Input::new(),
            current_frame: 0,
            primitives: Arena::new(reserve_size).unwrap(),
            layout: Clay::new(Dimensions::new(1920.0, 1080.0)),
            button_id: 0,
            renderer,
        };

        Box::new(Ui {
            state: UnsafeCell::new(state),
        })
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

    pub fn with_layout<F: FnOnce(&Ui), const N: usize>(&self, configs: [TypedConfig; N], f: F) {
        let state = unsafe { &mut *self.state.get() };

        state.layout.with(configs, |_clay| {
            f(self);
        });
    }

    pub fn end(&mut self) {
        let state = unsafe { &mut *self.state.get() };

        // TODO: Fix me
        let primitives = state.layout.end().collect::<Vec<_>>();
        state.renderer.render(&primitives);

        // Generate primitives from all boxes
        //state.generate_primitives();
        state.current_frame += 1;
    }

    fn generate_primitives(&mut self) {}

    pub fn load_font(&mut self, path: &str, size: i32) -> FontHandle {
        let state = unsafe { &mut *self.state.get() };

        state.text_generator.load_font_async(path, size)
    }

    #[rustfmt::skip]
    pub fn button(&self, _text: &str) -> Signal {
        let state = unsafe { &mut *self.state.get() };

        state.layout.with([
            Id::new_index("TestButton", state.button_id),
            Layout::new()
                .width(fixed!(160.0))
                .height(fixed!(40.0))
                .padding(Padding::all(8)).end(),
             Rectangle::new()
                .color(Color::rgba(244.0, 200.0, 200.0, 255.0))
                .corner_radius(CornerRadius::All(8.0))
                .end()], |_ui|
            {
                //dbg!(state.layout.get_bounding_box(Id::new_index("TestButton", state.button_id)));
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
