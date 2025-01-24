use raw_window_handle::RawWindowHandle;

pub type ImageHandle = u64;
pub type FontHandle = u64;

pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// Enum representing different rendering commands.
/// The bounding box is represented as [x0, y0, x1, x1].
pub enum RenderCommand<'a> {
    /// Command to draw a rectangle.
    DrawRect {
        bounding_box: [f32; 4],
        color: Color,
    },

    /// Command to draw a rounded rectangle.
    DrawRectRounded {
        bounding_box: [f32; 4],
        corners: [f32; 4],
        color: Color,
    },

    /// Command to draw a border.
    DrawBorder {
        bounding_box: [f32; 4],
        outer_radius: [f32; 4],
        inner_radius: [f32; 4],
        color: Color,
    },

    /// Command to draw text.
    DrawText {
        bounding_box: [f32; 4],
        text: &'a str,
        font_size: u16,
        font_handle: FontHandle,
        color: Color,
    },

    /// Command to draw an image.
    DrawImage {
        bounding_box: [f32; 4],
        rounded_corners: [f32; 4],
        color: Color,
        width: u32,
        height: u32,
        handle: ImageHandle,
        rounding: bool,
    },
}

/// Supported images formats.
pub enum ImageFormat {
    RGB8,
    RGBA8,
    RGBA16,
    I16,
}

pub struct Image<'a> {
    pub data: &'a [u8],
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
}

/// Struct containing the data for a software renderer. This is used if the renderer generates an
/// internal buffer that the system then has to use to display. This info isn't needed if the
/// renderer draws directly to the window.
pub struct SoftwareRenderData<'a> {
    pub buffer: &'a [u8],
    pub width: u32,
    pub height: u32,
}

pub trait Renderer {
    /// Creates a new renderer. The window handle is optional and can be used to create a renderer.
    /// Some renderers needs access to the underlying Window data to setup the renderer correctly
    /// and this is used to provide this information.
    fn new(_window_handle: Option<&RawWindowHandle>) -> Self
    where
        Self: Sized;

    /// Sets the window size.
    fn set_window_size(&mut self, _width: u32, _height: u32) {}

    /// Sets the text buffer with the given parameters.
    ///
    /// This is a generated text matching the text, font and size as input. It's expected that the
    /// backend will save away this data as it sees fit and use it later when rendering text.
    /// 
    /// # Parameters
    /// - `_text`: The text to be set.
    /// - `_font_size`: The size of the font.
    /// - `_font_id`: The handle to the font.
    /// - `_text_buffer`: The buffer containing the text data.
    fn set_text_buffer(&mut self, _text: &str, _font_size: i16, _handle: FontHandle, _image: &Image) {}

    /// Sets the image with the given handle. The renderer needs to keep track of this image as the handle
    /// will later be refereed to during `[Renderer::render]`.
    fn set_image(&mut self, _handle: ImageHandle, _image: &Image) {}

    /// If the renderer returns this it's expected that it has filled this buffer.
    fn software_renderer_info<'a>(&'a self) -> Option<SoftwareRenderData<'a>> {
        None
    }

    fn render(&mut self, commands: &[RenderCommand]);
}
