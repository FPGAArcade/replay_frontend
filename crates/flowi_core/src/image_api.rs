use crate::image::{ImageFormat, ImageInfo, ImageOptions, ImageMode};
use crate::io_handler::IoHandle;
use crate::State;
use resvg::{tiny_skia, usvg};

use fileorama::{Driver, DriverType, Error, Fileorama, LoadStatus, Progress};
use thiserror::Error as ThisError;

use zune_core::{
    bit_depth::BitDepth, colorspace::ColorSpace as ZuneColorSpace,
    options::DecoderOptions as ZuneDecoderOptions,
};

use crate::primitives::IVec2;

use zune_image::{errors::ImageErrors as ZuneError, image::Image as ZuneImage};
use color16::Color16;

//use zune_jpeg::zune_core::colorspace::ColorSpace;

#[derive(ThisError, Debug)]
pub enum ImageErrors {
    #[error("Zune Error")]
    ZuneError(#[from] ZuneError),
    #[error("Generic")]
    Generic(String),
}

#[derive(Default, Debug)]
enum ImageType {
    ZuneImage(Box<[u8]>, Option<ImageOptions>),
    SvgData((Box<[u8]>, Option<ImageOptions>)),
    #[default]
    None,
}

#[derive(Default, Debug)]
struct ImageLoader {
    image_type: ImageType,
}

const LINEAR_BIT_COUNT: i32 = 15;

fn srgb_to_linear(x: f32) -> f32 {
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

// TODO: Move to a common place
fn build_srgb_to_linear_table() -> [i16; 1 << 8] {
    let mut table = [0; 1 << 8];

    for (i, entry) in table.iter_mut().enumerate().take(1 << 8) {
        let srgb = i as f32 / 255.0;
        let linear = srgb_to_linear(srgb);
        *entry = (linear * ((1 << LINEAR_BIT_COUNT) - 1) as f32).round() as i16;
    }

    table
}

fn box_to_vec_u8<T>(b: Box<T>) -> Vec<u8> {
    let num_bytes = std::mem::size_of::<T>();
    let ptr = Box::into_raw(b) as *mut u8;
    unsafe { Vec::from_raw_parts(ptr, num_bytes, num_bytes) }
}

fn decode_zune(data: &[u8], image_options: Option<ImageOptions>) -> Result<Vec<u8>, ImageErrors> {
    let image = ZuneImage::read(data, ZuneDecoderOptions::default())?;

    // TODO: Pass this in as state
    let srgb_to_linear = build_srgb_to_linear_table();

    let depth = image.depth();
    let color_space = image.colorspace();
    let dimensions = image.dimensions();

    // Only supporting 8 bit depth for now
    if depth != BitDepth::Eight {
        return Err(ImageErrors::Generic(format!(
            "Unsupported depth: {:?}",
            depth
        )));
    }

    // Only deal with one frame for now
    // TODO: Optimize
    let frames = image.flatten_frames();
    let output_size = frames.iter().map(|f| f.len()).sum::<usize>();
    let mut image_data = vec![0u8; output_size]; // TODO: uninit

    assert_eq!(frames.len(), 1);

    for frame in frames {
        image_data.copy_from_slice(&frame);
    }

    let mut color16_output = Vec::with_capacity(output_size);

    // TODO: Optimize
    for v in image_data.chunks(3) {
        let r = v[0] as usize;
        let g = v[1] as usize;
        let b = v[2] as usize;

        let r = srgb_to_linear[r];
        let g = srgb_to_linear[g];
        let b = srgb_to_linear[b];
        let a = 255 << 7;

        color16_output.push(Color16::new(r, g, b, a));
    }

    let format = match color_space {
        ZuneColorSpace::RGB => ImageFormat::Rgb,
        ZuneColorSpace::RGBA => ImageFormat::Rgba,
        ZuneColorSpace::BGR => ImageFormat::Bgr,
        ZuneColorSpace::BGRA => ImageFormat::Bgra,
        ZuneColorSpace::Luma => ImageFormat::Alpha,
        ZuneColorSpace::LumaA => ImageFormat::Alpha,
        _ => {
            return Err(ImageErrors::Generic(format!(
                "Unknown colorspace: {:?}",
                color_space
            )))
        }
    };

    let image = if let Some(image_opts) = image_options {
       if image_opts.mode == ImageMode::ScaleToTargetInteger {
           image_scaler::upscale_image_integer(&color16_output, dimensions.0, dimensions.1,
                                               image_opts.size.x as _, image_opts.size.y as _)
       } else {
           unimplemented!("Unsupported mode");
       }
    } else {
        image_scaler::scale_image(&color16_output, dimensions.0 as _, dimensions.1 as _, 200, 200)
    };

    // TODO: handle multiple frames
    let image_info = Box::new(ImageInfo {
        data: image.data,
        format: format as u32,
        width: image.width as i32,
        height: image.height as i32,
        frame_delay: 0,
        frame_count: 1,
    });

    Ok(box_to_vec_u8(image_info))
}

fn render_svg(data: &[u8], image_options: Option<ImageOptions>) -> Result<Vec<u8>, ImageErrors> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(data, &opt).unwrap();
    //let rtree = resvg::Tree::from_usvg(&tree);

    let pixmap_size = tree.size().to_int_size();
    let mut width = pixmap_size.width() as i32;
    let mut height = pixmap_size.height() as i32;
    let mut scale_x = 1.0;
    let mut scale_y = 1.0;

    if let Some(options) = image_options {
        if options.size.x > 0 && options.size.y == 0 {
            let width_ratio = options.size.x as f32 / width as f32;
            width = options.size.x;
            height = (height as f32 * width_ratio) as i32;
            scale_x = width_ratio;
            scale_y = width_ratio;
        } else if options.size.x == 0 && options.size.y > 0 {
            let height_ratio = options.size.y as f32 / height as f32;
            height = options.size.y;
            width = (width as f32 * height_ratio) as i32;
            scale_x = height_ratio;
            scale_y = height_ratio;
        } else if options.size.x > 0 && options.size.y > 0 {
            width = options.size.x;
            height = options.size.y;
        }
    }

    let mut pixmap = tiny_skia::Pixmap::new(width as _, height as _).unwrap();
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale_x, scale_y),
        &mut pixmap.as_mut(),
    );

    // TODO: fix this up
    let svg_data = pixmap.as_ref().data();
    let image_info_offset = std::mem::size_of::<ImageInfo>();

    let _image_info = ImageInfo {
        data: Vec::new(),
        format: ImageFormat::Rgba as u32,
        width,
        height,
        frame_count: 1,
        frame_delay: 0,
    };

    let mut output_data = vec![0u8; svg_data.len()]; // TODO: uninit

    // Write header at the start of the data
    //let write_image_info: &mut ImageInfo = unsafe { std::mem::transmute(&mut output_data[0]) };
    //*write_image_info = image_info;

    //output_data[image_info_offset..].copy_from_slice(svg_data);

    if let Some(options) = image_options {
        if options.color.r > 0.0 || options.color.g > 0.0 || options.color.b > 0.0 {
            let r = (options.color.r * 255.0) as u8;
            let g = (options.color.g * 255.0) as u8;
            let b = (options.color.b * 255.0) as u8;

            // TODO: Optimize
            for i in 0..svg_data.len() / 4 {
                output_data[image_info_offset + (i * 4)] = r;
                output_data[image_info_offset + ((i * 4) + 1)] = g;
                output_data[image_info_offset + ((i * 4) + 2)] = b;
            }
        }
    }

    Ok(output_data)
}

static IMAGE_LOADER_NAME: &str = "flowi_image_loader";

impl Driver for ImageLoader {
    fn name(&self) -> &'static str {
        IMAGE_LOADER_NAME
    }

    fn get_directory_list(
        &mut self,
        _path: &str,
        _progress: &mut Progress,
    ) -> Result<fileorama::FilesDirs, Error> {
        Ok(fileorama::FilesDirs::default())
    }

    // Create a new instance given data. The Driver will take ownership of the data
    fn create_instance(&self) -> DriverType {
        Box::<ImageLoader>::default()
    }

    // Get some data in and returns true if driver can be mounted from it
    fn can_create_from_data(&self, data: &[u8], file_ext_hint: &str) -> bool {
        // we use the file_ext_hint to try to speed up the process
        match file_ext_hint {
            "jpg" | "jpeg" | "png" => return true,

            "svg" => {
                let opt = usvg::Options::default();
                let svg = usvg::Tree::from_data(data, &opt);
                if svg.is_ok() {
                    return true;
                }
            }

            _ => (),
        }

        false
    }

    // Create a new instance given data. The Driver will take ownership of the data
    fn create_from_data(
        &self,
        data: Box<[u8]>,
        file_ext_hint: &str,
        driver_data: &Option<Box<[u8]>>,
    ) -> Option<DriverType> {
        let options = if let Some(input_data) = driver_data {
            let io: &ImageOptions = unsafe { &*(input_data.as_ptr() as *const ImageOptions) };
            Some(*io)
        } else {
            None
        };

        // we use the file_ext_hint to try to speed up the process
        match file_ext_hint {
            "jpg" | "jpeg" | "png" => {
                return Some(Box::new(ImageLoader {
                    image_type: ImageType::ZuneImage(data, options),
                }));
            }

            "svg" => {
                let opt = usvg::Options::default();
                let svg = usvg::Tree::from_data(data.as_ref(), &opt);
                if svg.is_ok() {
                    return Some(Box::new(ImageLoader {
                        image_type: ImageType::SvgData((data, options)),
                    }));
                }
            }
            _ => (),
        }

        None
    }

    /// Returns a handle which updates the progress and returns the loaded data. This will try to
    fn load(&mut self, _path: &str, progress: &mut Progress) -> Result<LoadStatus, Error> {
        //println!("loading url: {} for image loader", _path);
        //progress.set_step(1);

        let decoded_data = match self.image_type {
            ImageType::ZuneImage(ref data, opts) => decode_zune(data, opts),
            ImageType::SvgData((ref data, opts)) => render_svg(data, opts),
            ImageType::None => return Err(Error::Generic("Unknown image type".to_owned())),
        };

        match decoded_data {
            Ok(data) => {
                progress.step()?;
                Ok(LoadStatus::Data(data.into_boxed_slice()))
            }

            Err(e) => {
                progress.step()?;
                Err(Error::Generic(format!("Error loading image: {:?}", e)))
            }
        }
    }
}

pub(crate) fn install_image_loader(vfs: &Fileorama) {
    vfs.add_driver(Box::<ImageLoader>::default());
}

#[inline]
pub fn load(state: &mut State, filename: &str) -> IoHandle {
    state
        .io_handler
        .load_with_driver(filename, IMAGE_LOADER_NAME)
}

#[inline]
#[allow(dead_code)]
fn load_with_options(state: &mut State, filename: &str, options: &ImageOptions) -> IoHandle {
    let data = [*options];

    state
        .io_handler
        .load_with_driver_data(filename, IMAGE_LOADER_NAME, &data)
}

#[inline]
pub fn load_background(state: &mut State, filename: &str, target_size: (u32, u32)) -> IoHandle {
    let image_options = ImageOptions {
        mode: ImageMode::ScaleToTargetInteger,
        size: IVec2::new(target_size.0 as i32, target_size.1 as i32), 
        ..Default::default()
    };

    let data = [image_options];

    state
        .io_handler
        .load_with_driver_data(filename, IMAGE_LOADER_NAME, &data)
}

/*
#[inline]
fn image_status(state: &InternalState, id: u64) -> ImageLoadStatus {
    if let Some(image) = state.io_handler.loaded.get(&id) {
        match image {
            LoadedData::Data(_e) => ImageLoadStatus::Loaded,
            LoadedData::Error(_e) => ImageLoadStatus::Failed,
        }
    } else {
        ImageLoadStatus::Loading
    }
}

#[inline]
fn image_data(state: &InternalState, id: u64) -> FlData {
    if let Some(image) = state.io_handler.loaded.get(&id) {
        match image {
            LoadedData::Data(data) => {
                let header_size = std::mem::size_of::<ImageInfo>();
                let data = &data[header_size..];
                FlData {
                    data: data.as_ptr() as *const core::ffi::c_void,
                    size: data.len() as u64,
                }
            }
            LoadedData::Error(_) => FlData::default(),
        }
    } else {
        FlData::default()
    }
}
*/

/*
#[inline]
fn image_info(state: &InternalState, image_id: u64) -> *const ImageInfo {
    if let Some(image_data) = state.io_handler.loaded.get(&image_id) {
        match image_data {
            LoadedData::Data(data) => {
                let image_info: &ImageInfo = unsafe { std::mem::transmute(&data[0]) };
                image_info
            }
            LoadedData::Error(_) => std::ptr::null(),
        }
    } else {
        std::ptr::null()
    }
}
*/

/*
struct WrapState<'a> {
    s: &'a mut crate::InternalState,
}
*/

/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ApplicationSettings;

    fn validate_red_image(state: &InternalState, handle: u64) {
        assert_eq!(image_status(state, handle), ImageLoadStatus::Loaded);
        let info = image_info(state, handle);
        assert_ne!(info, std::ptr::null());
        let info = unsafe { &*(info as *const ImageInfo) };
        assert_eq!(info.format, ImageFormat::Rgb as u32);
        assert_eq!(info.width, 200);
        assert_eq!(info.height, 200);
        let data = image_data(state, handle);
        assert_ne!(data.data, std::ptr::null());
        let data =
            unsafe { std::slice::from_raw_parts(data.data as *const u8, data.size as usize) };
        assert_eq!(data[0], 255);
        assert_eq!(data[1], 0);
        assert_eq!(data[2], 0);
    }

    fn wait_for_image_to_load(state: &mut crate::Instance, handle: u64) {
        for _ in 0..200 {
            state.update();

            if state.state.io_handler.is_loaded(handle) {
                return;
            }

            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // should never get here
        assert!(false);
    }

    #[test]
    fn png_red_image_ok() {
        let settings = ApplicationSettings {
            width: 0,
            height: 0,
        };
        let mut instance = crate::Instance::new(&settings);
        let handle = load(&mut instance.state, "data/png/solid_red.png");

        wait_for_image_to_load(&mut instance, handle);
        validate_red_image(&instance.state, handle);
    }

    #[test]
    fn jpg_green_image_ok() {
        let settings = ApplicationSettings {
            width: 0,
            height: 0,
        };
        let mut instance = crate::Instance::new(&settings);
        let handle = load(&mut instance.state, "data/jpeg/green.jpg");

        wait_for_image_to_load(&mut instance, handle);

        assert_eq!(
            image_status(&instance.state, handle),
            ImageLoadStatus::Loaded
        );
        let info = image_info(&instance.state, handle);
        assert_ne!(info, std::ptr::null());
        let info = unsafe { &*(info as *const ImageInfo) };
        assert_eq!(info.format, ImageFormat::Rgb as u32);
        assert_eq!(info.width, 64);
        assert_eq!(info.height, 64);
        let data = image_data(&instance.state, handle);
        assert_ne!(data.data, std::ptr::null());
        let data =
            unsafe { std::slice::from_raw_parts(data.data as *const u8, data.size as usize) };
        assert_eq!(data[0], 0);
        assert_eq!(data[1], 255);
        assert_eq!(data[2], 1);
    }

    #[test]
    fn svg_load_ok() {
        let settings = ApplicationSettings {
            width: 0,
            height: 0,
        };
        let mut instance = crate::Instance::new(&settings);
        let handle = load(&mut instance.state, "data/home.svg");

        wait_for_image_to_load(&mut instance, handle);

        assert_eq!(
            image_status(&instance.state, handle),
            ImageLoadStatus::Loaded
        );
        let info = image_info(&instance.state, handle);
        assert_ne!(info, std::ptr::null());
        let info = unsafe { &*(info as *const ImageInfo) };

        assert_eq!(info.format, ImageFormat::Rgba as u32);
        assert_eq!(info.width, 22);
        assert_eq!(info.height, 16);
        assert_eq!(info.frame_count, 1);
    }

    #[test]
    fn png_load_broken_fail() {
        let settings = ApplicationSettings {
            width: 0,
            height: 0,
        };
        let mut instance = crate::Instance::new(&settings);
        let handle = load(&mut instance.state, "data/png/broken/xs1n0g01.png");

        wait_for_image_to_load(&mut instance, handle);
        assert!(image_status(&instance.state, handle) == ImageLoadStatus::Failed);
    }
}
*/
