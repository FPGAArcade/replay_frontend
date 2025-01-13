mod image;
mod image_api;
pub mod input;
mod internal_error;
mod io_handler;
pub mod primitives;
pub mod signal;
pub mod widgets;

use arena_allocator::{Arena};
use fileorama::Fileorama;
pub use io_handler::IoHandler;
use primitives::{Primitive};
use signal::Signal;

type FlowiKey = u64;

pub(crate) struct Flowi {
    pub(crate) vfs: Fileorama,
    pub(crate) io_handler: IoHandler,
    pub(crate) input: input::Input,
    pub(crate) primitives: Arena,
    pub(crate) hot_item: FlowiKey,
    pub(crate) current_frame: u64,
}

impl Flowi {
    pub fn new() -> Box<Self> {
        let vfs = Fileorama::new(2);
        let io_handler = IoHandler::new(&vfs);
        crate::image_api::install_image_loader(&vfs);

        let reserve_size = 1024 * 1024 * 1024;

        Box::new(Flowi {
            vfs,
            io_handler,
            hot_item: 0,
            input: input::Input::new(),
            current_frame: 0,
            primitives: Arena::new(reserve_size).unwrap(),
        })
    }

    pub fn begin(&mut self, _delta_time: f32, _width: usize, _height: usize) {

        self.io_handler.update();
        self.primitives.rewind();
    }

    pub fn end(&mut self) {
        // Generate primitives from all boxes
        self.generate_primitives();
        self.current_frame += 1;
    }


    fn generate_primitives(&mut self) {
    }

    pub fn button(&mut self, _text: &str) -> Signal {
        /*
        let box_area = self.create_box_with_string(text);
        self.signal(box_area)
        */
        Signal::new()
    }

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
