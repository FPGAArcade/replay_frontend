use raw_window_handle::RawWindowHandle;

pub type ImageHandle = u64;
pub type FontHandle = u64;
pub type TextHandle = u64;

#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub struct RawVoidPtr(pub *const core::ffi::c_void);

impl Default for RawVoidPtr {
    fn default() -> Self {
        Self(core::ptr::null())
    }
}

unsafe impl Send for RawVoidPtr {}
unsafe impl Sync for RawVoidPtr {}

#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Debug)]
pub struct StringSlice {
    pub ptr: *const u8,
    pub len: u32,
}

impl StringSlice {
    pub fn new(s: &str) -> Self {
        Self {
            ptr: s.as_ptr(),
            len: s.len() as u32,
        }
    }

    pub fn as_str(&self) -> &str {
        unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(self.ptr, self.len as usize))
        }
    }
}

#[derive(Debug)]
pub struct DrawRectRoundedData {
    pub corners: [f32; 4],
}

#[derive(Debug)]
pub struct DrawBorderData {
    pub outer_radius: [f32; 4],
    pub inner_radius: [f32; 4],
}

#[derive(Debug)]
pub struct DrawImage {
    pub width: u32,
    pub height: u32,
    pub handle: *const i16,
    pub rounded_corners: [f32; 4],
    pub rounding: bool,
}

#[derive(Debug)]
pub struct DrawTextData {
    pub text: StringSlice,
    pub font_size: u16,
    pub font_handle: FontHandle,
}

#[derive(Debug, Default)]
pub struct DrawTextBufferData {
    pub data: RawVoidPtr,
    pub handle: TextHandle,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug)]
pub enum RenderType {
    DrawRect,
    DrawRectRounded(DrawRectRoundedData),
    DrawBorder(DrawBorderData),
    DrawTextBuffer(DrawTextBufferData),
    DrawImage(DrawImage),
    ScissorStart,
    ScissorEnd,
    Custom,
    None,
}

#[derive(Debug)]
pub struct RenderCommand {
    /// The bounding box is represented as [x0, y0, x1, x1].
    pub bounding_box: [f32; 4],
    /// Color of the render command. This may not apply to all commands, but is common enough
    /// so it's placed outside of each specific type
    pub color: Color,
    /// The type of render command.
    pub render_type: RenderType,
}

/// Supported images formats.
pub enum ImageFormat {
    RGB8,
    RGBA8,
    RGBA16,
    I16,
}

pub struct Image {
    // This is static because lifetime is held by other systems.
    pub data: core::ffi::c_void,
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
    fn new(window_size: (usize, usize), _window_handle: Option<&RawWindowHandle>) -> Self
    where
        Self: Sized;

    /// Sets the window size.
    fn set_window_size(&mut self, _width: u32, _height: u32) {}

    /// Sets the text buffer with the given parameters.
    ///
    /// This is a generated text matching the text, font and size as input. It's expected that the
    /// backend will save away this data as it sees fit and use it later when rendering text.
    fn set_text_buffer(&mut self, _handle: TextHandle, _image: &Image) {}

    /// Sets the image with the given handle. The renderer needs to keep track of this image as the handle
    /// will later be refereed to during `[Renderer::render]`.
    fn set_image(&mut self, _handle: ImageHandle, _image: &Image) {}

    /// If the renderer returns this it's expected that it has filled this buffer.
    fn software_renderer_info(&self) -> Option<SoftwareRenderData> {
        None
    }

    fn render(&mut self, commands: &[RenderCommand]);
}
