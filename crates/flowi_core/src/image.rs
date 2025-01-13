use crate::primitives::{Color, IVec2, Vec2};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ImageFormat {
    /// 8-bit per channel Red, Green and Blue
    Rgb = 0,
    /// 8-bit per channel Red, Green, Blue and Alpha
    Rgba = 1,
    /// 8-bit per channel Blue, Green and Red
    Bgr = 2,
    /// 8-bit per channel Blue, Green and Red and Alpha
    Bgra = 3,
    /// 8-bit per channel Alpha only
    Alpha = 4,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ImageLoadStatus {
    /// The image is still loading
    Loading = 0,
    /// The image has finished loading
    Loaded = 1,
    /// The image failed to load
    Failed = 2,
}

#[derive(Debug)]
pub struct ImageInfo {
    /// Format of the image. See the ImageFormat enum
    pub format: u32,
    /// width of the image
    pub width: i32,
    /// height of the Image
    pub height: i32,
    /// Number of frames. This is 1 for static images and > 1 for animated images
    pub frame_count: i32,
    /// How long each frame should be displayed for in milliseconds
    pub frame_delay: i32,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ImageOptions {
    /// The scale of the image. This is useful for loading SVGs at different sizes.
    pub scale: Vec2,
    /// Set a size of the image (this will override the scale). if one component is set to 0 it will be calculated based on the aspect ratio of the image.
    pub size: IVec2,
    /// Set a size of the image (this will override the scale). if one component is set to 0 it will be calculated based on the aspect ratio of the image.
    pub color: Color,
}
