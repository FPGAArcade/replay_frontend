mod image;
mod image_api;
mod internal_error;
mod io_handler;
mod box_area;
pub mod layout;
pub mod primitives;
pub mod input;
pub mod widgets;

use fileorama::Fileorama;
pub use io_handler::IoHandler;

struct InternalState {
    pub(crate) vfs: Fileorama,
    pub(crate) io_handler: IoHandler,
}

pub struct Flowi {
    state: Box<InternalState>,
}

impl Flowi {
    pub fn new() -> Self {
        let vfs = Fileorama::new(2);
        let io_handler = IoHandler::new(&vfs);
        crate::image_api::install_image_loader(&vfs);

        let state = Box::new(InternalState { vfs, io_handler });

        Self { state }
    }
}
