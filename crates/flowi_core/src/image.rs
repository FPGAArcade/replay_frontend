use crate::primitives::{Color, IVec2, Vec2};
use image_scaler::Color16;

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
#[allow(dead_code)]
pub enum ImageLoadStatus {
    /// The image is still loading
    Loading = 0,
    /// The image has finished loading
    Loaded = 1,
    /// The image failed to load
    Failed = 2,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ImageInfo {
    pub data: Vec<Color16>,
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

#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ImageMode {
    /// The image will be scaled to fit the target size while maintaining the aspect ratio
    /// of the image. Will only scale the image in steps of 2x, 3x, 4x, etc. 
    ScaleToTargetInteger,
}

impl Default for ImageMode {
    fn default() -> Self {
        ImageMode::ScaleToTargetInteger
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[allow(dead_code)]
pub struct ImageOptions {
    /// Mode of the scaling
    pub mode: ImageMode,
    /// The scale of the image. This is useful for loading SVGs at different sizes.
    pub scale: Vec2,
    pub size: IVec2,
    pub color: Color,
}
