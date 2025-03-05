use std::hash::{Hash, Hasher};
use fxhash::FxHasher;
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

impl Hash for StringSlice {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
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
    pub stride: u32,
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
    DrawBackground(DrawImage),
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
    /// so it's placed outside each specific type
    pub color: Color,
    /// The type of render command.
    pub render_type: RenderType,
}


// Helper function to convert f32 to a stable integer representation
// Multiplying by a large factor preserves precision while converting to integer
fn float_to_stable_int(value: f32) -> i32 {
    let precision_factor = 100.0; // 3 decimal places of precision
    (value * precision_factor) as i32
}

// Helper trait to hash float arrays in a stable way
trait StableFloatHash {
    fn hash_stable<H: Hasher>(&self, state: &mut H);
}

impl StableFloatHash for [f32; 4] {
    fn hash_stable<H: Hasher>(&self, state: &mut H) {
        for &value in self {
            float_to_stable_int(value).hash(state);
        }
    }
}

// Implement Hash for the structs
impl Hash for DrawRectRoundedData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.corners.hash_stable(state);
    }
}

impl Hash for DrawBorderData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.outer_radius.hash_stable(state);
        self.inner_radius.hash_stable(state);
    }
}

impl Hash for DrawImage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.width.hash(state);
        self.height.hash(state);
        self.stride.hash(state);
        // Hash the pointer address as usize
        (self.handle as usize).hash(state);
        self.rounded_corners.hash_stable(state);
        self.rounding.hash(state);
    }
}

// Assuming StringSlice implements Hash
impl Hash for DrawTextData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text.hash(state);
        self.font_size.hash(state);
        // Assuming FontHandle implements Hash or can be converted to a hashable type
        self.font_handle.hash(state);
    }
}

impl Hash for DrawTextBufferData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the pointer address as usize
        (self.data.0 as usize).hash(state);
        self.handle.hash(state);
        self.width.hash(state);
        self.height.hash(state);
    }
}

impl Hash for RenderType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash a discriminant value first
        std::mem::discriminant(self).hash(state);

        // Hash the variant-specific data
        match self {
            RenderType::DrawRect => {
                // No additional data to hash
            },
            RenderType::DrawBackground(image) => {
                image.hash(state);
            },
            RenderType::DrawRectRounded(data) => {
                data.hash(state);
            },
            RenderType::DrawBorder(data) => {
                data.hash(state);
            },
            RenderType::DrawTextBuffer(data) => {
                data.hash(state);
            },
            RenderType::DrawImage(image) => {
                image.hash(state);
            },
            RenderType::ScissorStart |
            RenderType::ScissorEnd |
            RenderType::Custom |
            RenderType::None => {
                // No additional data to hash
            },
        }
    }
}

impl Hash for RenderCommand {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bounding_box.hash_stable(state);
        (self.color.r as i32).hash(state);
        (self.color.g as i32).hash(state);
        (self.color.b as i32).hash(state);
        (self.color.a as i32).hash(state);
        self.render_type.hash(state);
    }
}

// Function to get a hash for a RenderCommand
pub fn hash_render_command(cmd: &RenderCommand) -> u64 {
    let mut hasher = FxHasher::default();
    cmd.hash(&mut hasher);
    hasher.finish()
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Test the float_to_stable_int function
    #[test]
    fn test_float_to_stable_int() {
        // Test basic conversion
        assert_eq!(float_to_stable_int(1.0), 100);
        assert_eq!(float_to_stable_int(0.5), 50);

        // Test that small differences are preserved
        assert_eq!(float_to_stable_int(0.01), 1);
        assert_ne!(float_to_stable_int(0.01), float_to_stable_int(0.02));

        // Test that very close values are considered the same (epsilon test)
        let epsilon = 0.001; // Smaller than our precision factor can differentiate
        assert_eq!(float_to_stable_int(1.0), float_to_stable_int(1.0 + epsilon));
    }

    // Test for stable hashing of float arrays
    #[test]
    fn test_stable_float_array_hashing() {
        let arr1 = [1.0, 2.0, 3.0, 4.0];
        let arr2 = [1.0, 2.0, 3.0, 4.0];
        let arr3 = [1.0, 2.0, 3.0, 4.1]; // Slightly different

        let hash1 = hash_float_array(&arr1);
        let hash2 = hash_float_array(&arr2);
        let hash3 = hash_float_array(&arr3);

        // Same values should produce same hashes
        assert_eq!(hash1, hash2);

        // Different values should produce different hashes
        assert_ne!(hash1, hash3);

        // Test with very close values
        let arr4 = [1.0, 2.0, 3.0, 4.0 + 0.0000001]; // Extremely close
        let hash4 = hash_float_array(&arr4);

        // Values within epsilon should hash the same
        assert_eq!(hash1, hash4);
    }

    // Helper function to hash a float array
    fn hash_float_array(arr: &[f32; 4]) -> u64 {
        let mut hasher = FxHasher::default();
        arr.hash_stable(&mut hasher);
        hasher.finish()
    }

    // Test consistent hashing for DrawRectRoundedData
    #[test]
    fn test_draw_rect_rounded_data_hash() {
        let data1 = DrawRectRoundedData { corners: [1.0, 2.0, 3.0, 4.0] };
        let data2 = DrawRectRoundedData { corners: [1.0, 2.0, 3.0, 4.0] };
        let data3 = DrawRectRoundedData { corners: [1.1, 2.0, 3.0, 4.0] };

        assert_eq!(hash_struct(&data1), hash_struct(&data2));
        assert_ne!(hash_struct(&data1), hash_struct(&data3));
    }

    // Test consistent hashing for DrawBorderData
    #[test]
    fn test_draw_border_data_hash() {
        let data1 = DrawBorderData {
            outer_radius: [1.0, 2.0, 3.0, 4.0],
            inner_radius: [0.5, 1.5, 2.5, 3.5]
        };

        let data2 = DrawBorderData {
            outer_radius: [1.0, 2.0, 3.0, 4.0],
            inner_radius: [0.5, 1.5, 2.5, 3.5]
        };

        let data3 = DrawBorderData {
            outer_radius: [1.0, 2.0, 3.0, 4.0],
            inner_radius: [0.6, 1.5, 2.5, 3.5] // Slightly different
        };

        assert_eq!(hash_struct(&data1), hash_struct(&data2));
        assert_ne!(hash_struct(&data1), hash_struct(&data3));
    }

    // Test RenderCommand hashing
    #[test]
    fn test_render_command_hash() {
        let cmd1 = create_test_render_command(1.0, 2.0);
        let cmd2 = create_test_render_command(1.0, 2.0);
        let cmd3 = create_test_render_command(1.1, 2.0);

        assert_eq!(hash_render_command(&cmd1), hash_render_command(&cmd2));
        assert_ne!(hash_render_command(&cmd1), hash_render_command(&cmd3));
    }

    // Test stability with small floating point variations
    #[test]
    fn test_float_stability() {
        // Values that should hash the same due to precision limits
        let cmd1 = create_test_render_command(1.0, 2.0);
        let cmd2 = create_test_render_command(1.0000001, 2.0);  // Negligible difference

        assert_eq!(hash_render_command(&cmd1), hash_render_command(&cmd2));

        // Values that should hash differently
        let cmd3 = create_test_render_command(1.01, 2.0);  // Noticeable difference

        assert_ne!(hash_render_command(&cmd1), hash_render_command(&cmd3));
    }

    // Test HashMap usage with our hashed types
    #[test]
    fn test_hashmap_with_render_commands() {
        let mut map = HashMap::new();

        let cmd1 = create_test_render_command(1.0, 2.0);
        let cmd2 = create_test_render_command(1.0, 2.0);  // Same values
        let cmd3 = create_test_render_command(3.0, 4.0);  // Different values

        map.insert(hash_render_command(&cmd1), "first");

        // Should find the same key for cmd2
        assert_eq!(map.get(&hash_render_command(&cmd2)), Some(&"first"));

        // Should not find the key for cmd3
        assert_eq!(map.get(&hash_render_command(&cmd3)), None);
    }

    // Helper function to create a test RenderCommand
    fn create_test_render_command(x: f32, y: f32) -> RenderCommand {
        RenderCommand {
            bounding_box: [x, y, x + 10.0, y + 10.0],
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            render_type: RenderType::DrawRectRounded(DrawRectRoundedData {
                corners: [x, y, x + 1.0, y + 1.0],
            }),
        }
    }

    // Helper function to hash a struct that implements Hash
    fn hash_struct<T: Hash>(value: &T) -> u64 {
        let mut hasher = FxHasher::default();
        value.hash(&mut hasher);
        hasher.finish()
    }

    // Test hashing with more complex structures
    #[test]
    fn test_complex_render_command_hash() {
        // Create a complex render command with different render types
        let cmd1 = RenderCommand {
            bounding_box: [10.0, 20.0, 30.0, 40.0],
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            render_type: RenderType::DrawImage(DrawImage {
                width: 100,
                height: 200,
                stride: 300,
                handle: std::ptr::null(),
                rounded_corners: [5.0, 5.0, 5.0, 5.0],
                rounding: true,
            }),
        };

        let cmd2 = RenderCommand {
            bounding_box: [10.0, 20.0, 30.0, 40.0],
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            render_type: RenderType::DrawImage(DrawImage {
                width: 100,
                height: 200,
                stride: 300,
                handle: std::ptr::null(),
                rounded_corners: [5.0, 5.0, 5.0, 5.0],
                rounding: true,
            }),
        };

        let cmd3 = RenderCommand {
            bounding_box: [10.0, 20.0, 30.0, 40.0],
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            render_type: RenderType::DrawRectRounded(DrawRectRoundedData {
                corners: [5.0, 5.0, 5.0, 5.0],
            }),
        };

        // Same render type and data should hash the same
        assert_eq!(hash_render_command(&cmd1), hash_render_command(&cmd2));

        // Different render types should hash differently
        assert_ne!(hash_render_command(&cmd1), hash_render_command(&cmd3));
    }
}
