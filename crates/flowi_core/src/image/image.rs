#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Format {
    /// 8-bit per channel Red, Green and Blue
    Rgb,
    /// 8-bit per channel Red, Green, Blue and Alpha
    Rgba,
    /// 8-bit per channel Blue, Green and Red
    Bgr,
    /// 8-bit per channel Blue, Green and Red and Alpha
    Bgra,
    /// 8-bit per channel Alpha only
    Alpha,
    /// 16-bit per channel Red, Green and Blue
    Rgb16,
    /// 16-bit per channel Red, Green and Blue and Alpha
    Rgba16,
    /// 16-bit per channel Alpha only
    Alpha16,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BorderType {
    /// No border
    None,
    /// A single pixel border
    Black(usize),
    /// Repeat the edge pixels
    Repeat(usize),
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ImageInfo {
    pub data: Vec<u8>,
    /// Format of the image.
    pub format: Format,
    /// width of the image
    pub width: i32,
    /// height of the Image
    pub height: i32,
    /// Number of frames. This is 1 for static images and > 1 for animated images
    pub frame_count: i32,
    /// How long each frame should be displayed for in milliseconds
    pub frame_delay: i32,
    /// Border type of the image
    pub border_type: BorderType,
    /// Start of the data excluding the border
    pub start_offset_ex_borders: usize,
    /// Full width of the image including the border
    pub stride: usize,
}

impl ImageInfo {
    pub fn new(
        data: Vec<u8>,
        format: Format,
        width: i32,
        height: i32,
        frame_count: i32,
        frame_delay: i32,
        border_type: BorderType,
        start_offset_ex_borders: usize,
        stride: usize,
    ) -> Self {
        ImageInfo {
            data,
            format,
            width,
            height,
            frame_count,
            frame_delay,
            border_type,
            start_offset_ex_borders,
            stride,
        }
    }

    pub fn vec_to_u8<T>(v: Vec<T>) -> Vec<u8> {
        let element_size = std::mem::size_of::<T>();
        let len = v.len();
        let capacity = v.capacity();
        let ptr = v.as_ptr() as *mut u8;

        // Prevent the original vector from being dropped
        std::mem::forget(v);

        // Create a new Vec<u8> from the raw parts
        unsafe { Vec::from_raw_parts(ptr, len * element_size, capacity * element_size) }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Resize {
    /// No image resizing
    None,
    /// Resize image to 2x,3x,etc
    Integer,
    /// Resize image to 2x,3x,etc with a vignette effect
    IntegerVignette,
    /// Resize using sharp bilinear filter
    SharpBilinear,
}

#[derive(Copy, Clone, Debug)]
pub enum ColorDepth {
    /// 8-bit per channel storage of data
    Depth8,
    /// 16-bit per channel storage of data (Rgba8 -> Rgba16, Alpha8 -> Alpha16, etc)
    Depth16,
}

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub struct LoadOptions {
    /// Resize the image
    pub resize: Resize,
    /// Color depth of the image
    pub color_depth: ColorDepth,
    /// Target size of the image (0, 0) means no resizing
    pub target_size: (i32, i32),
}

impl Default for LoadOptions {
    fn default() -> Self {
        LoadOptions {
            resize: Resize::None,
            color_depth: ColorDepth::Depth16,
            target_size: (0, 0),
        }
    }
}
