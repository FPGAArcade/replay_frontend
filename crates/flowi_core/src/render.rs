use flowi_renderer::{RenderCommand, Renderer};
use raw_window_handle::RawWindowHandle;

pub struct DummyRenderer {}

impl Renderer for DummyRenderer {
    fn new(_window: Option<&RawWindowHandle>) -> Self {
        Self {}
    }

    fn render(&mut self, _commands: &[RenderCommand]) {}
}
