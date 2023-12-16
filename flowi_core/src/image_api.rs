use crate::image::{ImageFormat, ImageInfo, ImageLoadStatus};
use crate::io_handler::LoadedData;
use crate::manual::{FlData, FlString};
use crate::InternalState;
use fileorama::{
    Error, Fileorama, FilesDirs, LoadStatus, MemoryDriver, MemoryDriverType, Progress,
};
use thiserror::Error as ThisError;

use zune_jpeg::{
    errors::DecodeErrors as JpegDecodeErrors, zune_core::colorspace::ColorSpace as JpegColorSpace,
    zune_core::options::DecoderOptions as JpegDecoderOptions, JpegDecoder,
};

use zune_png::{
    error::PngDecodeErrors, zune_core::bit_depth::BitDepth as PngBitDepth,
    zune_core::colorspace::ColorSpace as PngColorSpace, PngDecoder,
};

#[derive(ThisError, Debug)]
pub enum ImageErrors {
    #[error("Png Error")]
    ParseError(#[from] PngDecodeErrors),
    #[error("Jpeg Error")]
    JpegError(#[from] JpegDecodeErrors),
    #[error("Gif Error")]
    GifError(#[from] gif::DecodingError),
    #[error("Gif Disposal Error")]
    GifDisposeError(#[from] gif_dispose::Error),
    #[error("Generic")]
    Generic(String),
}

#[derive(Default, Debug)]
enum ImageType {
    PngData(Box<[u8]>),
    JpegData(Box<[u8]>),
    GifData(Box<[u8]>),
    #[default]
    None,
}

#[derive(Default, Debug)]
struct ImageLoader {
    image_type: ImageType,
}

fn decode_png(data: &[u8]) -> Result<Vec<u8>, ImageErrors> {
    let mut decoder = PngDecoder::new(data);
    decoder.decode_headers()?;

    // unwraping here is safe as we have already checked that the headers are ok
    let depth = decoder.get_depth().unwrap();
    let buffer_size = decoder.output_buffer_size().unwrap();
    let color_space = decoder.get_colorspace().unwrap();
    let dimensions = decoder.get_dimensions().unwrap();

    // Only supporting 8-bit PNGs for now
    if depth != PngBitDepth::Eight {
        return Err(ImageErrors::Generic(format!(
            "Unsupported depth: {:?}",
            depth
        )));
    }

    let image_info_offset = std::mem::size_of::<ImageInfo>();

    let output_size = buffer_size + image_info_offset;
    let mut output_data = vec![0u8; output_size]; // TODO: uninit

    decoder.decode_into(&mut output_data[image_info_offset..])?;

    let format = match color_space {
        PngColorSpace::RGB => ImageFormat::Rgb,
        PngColorSpace::RGBA => ImageFormat::Rgba,
        PngColorSpace::BGR => ImageFormat::Bgr,
        PngColorSpace::BGRA => ImageFormat::Bgra,
        PngColorSpace::Luma => ImageFormat::Alpha,
        PngColorSpace::LumaA => ImageFormat::Alpha,
        _ => {
            return Err(ImageErrors::Generic(format!(
                "Unknown colorspace: {:?}",
                color_space
            )))
        }
    };

    // TODO: handle multiple frames
    let image_info = ImageInfo {
        format: format as u32,
        width: dimensions.0 as u32,
        height: dimensions.1 as u32,
        frame_delay: 0,
        frame_count: 1,
    };

    // Write header at the start of the data
    let write_image_info: &mut ImageInfo = unsafe { std::mem::transmute(&mut output_data[0]) };
    *write_image_info = image_info;

    Ok(output_data)
}

fn decode_jpeg(data: &[u8]) -> Result<Vec<u8>, ImageErrors> {
    let opts = JpegDecoderOptions::new_fast().jpeg_set_out_colorspace(JpegColorSpace::RGB);

    let mut decoder = JpegDecoder::new(data);
    decoder.set_options(opts);
    decoder.decode_headers()?;

    let dimensions = decoder.dimensions().unwrap();
    let buffer_size = decoder.output_buffer_size().unwrap();

    let image_info_offset = std::mem::size_of::<ImageInfo>();

    let output_size = buffer_size + image_info_offset;
    let mut output_data = vec![0u8; output_size]; // TODO: uninit

    decoder.decode_into(&mut output_data[image_info_offset..])?;

    let image_info = ImageInfo {
        format: ImageFormat::Rgb as u32,
        width: dimensions.0 as u32,
        height: dimensions.1 as u32,
        frame_count: 1,
        frame_delay: 0,
    };

    // Write header at the start of the data
    let write_image_info: &mut ImageInfo = unsafe { std::mem::transmute(&mut output_data[0]) };
    *write_image_info = image_info;

    Ok(output_data)
}

fn decode_gif(data: &[u8]) -> Result<Vec<u8>, ImageErrors> {
    let mut gif_opts = gif::DecodeOptions::new();
    gif_opts.set_color_output(gif::ColorOutput::Indexed);

    let mut decoder = gif_opts.read_info(data)?;
    let mut screen = gif_dispose::Screen::new_decoder(&decoder);

    let image_info_offset = std::mem::size_of::<ImageInfo>();
    let width = screen.pixels.width();
    let height = screen.pixels.height();

    let mut frames = Vec::new();
    let mut buffer_size = 0;
    let mut frame_delay_ms = u32::MAX;

    while let Some(frame) = decoder.read_next_frame()? {
        screen.blit_frame(&frame)?;
        // we only handle a uniform delay for now
        frame_delay_ms = frame_delay_ms.min(frame.delay as u32 * 10);
        let f = screen.pixels.buf().to_vec();
        buffer_size += f.len() * 4;
        frames.push(f);
    }

    let output_size = buffer_size + image_info_offset;
    let mut output_data = vec![0u8; output_size]; // TODO: uninit

    let image_info = ImageInfo {
        format: ImageFormat::Rgba as u32,
        width: width as u32,
        height: height as u32,
        frame_count: frames.len() as u32,
        frame_delay: frame_delay_ms,
    };

    // Write header at the start of the data
    let write_image_info: &mut ImageInfo = unsafe { std::mem::transmute(&mut output_data[0]) };
    *write_image_info = image_info;

    for (i, frame) in frames.iter().enumerate() {
        let offset = image_info_offset + (i * frame.len());
        let frame: &[u8] = bytemuck::cast_slice(frame);
        output_data[offset..offset + frame.len()].copy_from_slice(frame);
    }

    Ok(output_data)
}

static IMAGE_LOADER_NAME: &'static str = "flowi_image_loader";

impl MemoryDriver for ImageLoader {
    fn name(&self) -> &'static str {
        IMAGE_LOADER_NAME
    }

    // Create a new instance given data. The Driver will take ownership of the data
    fn create_instance(&self) -> MemoryDriverType {
        Box::<ImageLoader>::default()
    }

    // Get some data in and returns true if driver can be mounted from it
    fn can_create_from_data(&self, data: &[u8], file_ext_hint: &str) -> bool {
        // we use the file_ext_hint to try to speed up the process
        match file_ext_hint {
            "png" => {
                let mut png_decoder = PngDecoder::new(data);
                let headers = png_decoder.decode_headers();
                if headers.is_ok() {
                    return true;
                }
            }
            "jpg" | "jpeg" => {
                let mut jpeg_decoder = JpegDecoder::new(data);
                let headers = jpeg_decoder.decode_headers();
                if headers.is_ok() {
                    return true;
                }
            }
            "gif" => {
                let mut decoder = gif::DecodeOptions::new();
                decoder.set_color_output(gif::ColorOutput::Indexed);
                if decoder.read_info(data).is_ok() {
                    return true;
                }
            }
            _ => {}
        }

        // fallback to trying all decoders
        let mut png_decoder = PngDecoder::new(data);
        let headers = png_decoder.decode_headers();
        if headers.is_ok() {
            return true;
        }

        let mut jpeg_decoder = JpegDecoder::new(data.as_ref());
        let headers = jpeg_decoder.decode_headers();
        if headers.is_ok() {
            return true;
        }

        let mut decoder = gif::DecodeOptions::new();
        decoder.set_color_output(gif::ColorOutput::Indexed);
        decoder.read_info(data.as_ref()).is_ok()
    }

    // Create a new instance given data. The Driver will take ownership of the data
    fn create_from_data(
        &self,
        data: Box<[u8]>,
        file_ext_hint: &str,
        _driver_data: &Option<Box<[u8]>>,
    ) -> Option<MemoryDriverType> {
        // we use the file_ext_hint to try to speed up the process
        match file_ext_hint {
            "png" => {
                let mut png_decoder = PngDecoder::new(data.as_ref());
                let headers = png_decoder.decode_headers();
                if headers.is_ok() {
                    return Some(Box::new(ImageLoader {
                        image_type: ImageType::PngData(data),
                    }));
                }
            }
            "jpg" | "jpeg" => {
                let mut jpeg_decoder = JpegDecoder::new(data.as_ref());
                let headers = jpeg_decoder.decode_headers();
                if headers.is_ok() {
                    return Some(Box::new(ImageLoader {
                        image_type: ImageType::JpegData(data),
                    }));
                }
            }
            "gif" => {
                let mut decoder = gif::DecodeOptions::new();
                decoder.set_color_output(gif::ColorOutput::Indexed);
                if decoder.read_info(data.as_ref()).is_ok() {
                    return Some(Box::new(ImageLoader {
                        image_type: ImageType::GifData(data),
                    }));
                }
            }
            _ => {}
        }

        // Check if png or jpeg loader can open the data
        let mut jpeg_decoder = JpegDecoder::new(data.as_ref());
        let headers = jpeg_decoder.decode_headers();
        if headers.is_ok() {
            return Some(Box::new(ImageLoader {
                image_type: ImageType::JpegData(data),
            }));
        }

        let mut png_decoder = PngDecoder::new(data.as_ref());
        let headers = png_decoder.decode_headers();
        if headers.is_ok() {
            return Some(Box::new(ImageLoader {
                image_type: ImageType::PngData(data),
            }));
        }

        let mut decoder = gif::DecodeOptions::new();
        decoder.set_color_output(gif::ColorOutput::Indexed);
        match decoder.read_info(data.as_ref()) {
            Ok(_) => {
                return Some(Box::new(ImageLoader {
                    image_type: ImageType::GifData(data),
                }));
            }
            Err(_) => {}
        }

        None
    }

    /// Returns a handle which updates the progress and returns the loaded data. This will try to
    fn load(&mut self, _path: &str, progress: &mut Progress) -> Result<LoadStatus, Error> {
        println!("loading url: {} for image loader", _path);

        //progress.set_step(1);

        let decoded_data = match self.image_type {
            ImageType::PngData(ref data) => decode_png(data),
            ImageType::JpegData(ref data) => decode_jpeg(data),
            ImageType::GifData(ref data) => decode_gif(data),
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

    fn get_directory_list(
        &mut self,
        _path: &str,
        _progress: &mut Progress,
    ) -> Result<FilesDirs, Error> {
        Ok(FilesDirs::default())
    }
}

pub(crate) fn install_image_loader(vfs: &Fileorama) {
    vfs.add_memory_driver(Box::new(ImageLoader::default()));
}

#[inline]
fn create_from_file(state: &mut InternalState, filename: &str) -> u64 {
    let handle = state
        .io_handler
        .load_with_driver(filename, IMAGE_LOADER_NAME);
    handle as u64
}

#[inline]
fn image_status(state: &InternalState, id: u64) -> ImageLoadStatus {
    if let Some(image) = state.io_handler.loaded.get(&id) {
        match image {
            LoadedData::Data(_) => ImageLoadStatus::Loaded,
            LoadedData::Error(_) => ImageLoadStatus::Failed,
        }
    } else {
        ImageLoadStatus::Loaded
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

struct WrapState<'a> {
    s: &'a mut crate::InternalState,
}

// FFI functions
#[no_mangle]
pub fn fl_image_create_from_file_impl(data: *mut core::ffi::c_void, filename: FlString) -> u64 {
    let state = &mut unsafe { &mut *(data as *mut WrapState) }.s;
    let name = filename.as_str();
    create_from_file(state, name)
}

#[no_mangle]
pub fn fl_image_get_info_impl(data: *const core::ffi::c_void, image: u64) -> *const ImageInfo {
    let state = &mut unsafe { &mut *(data as *mut WrapState) }.s;
    image_info(state, image)
}

#[no_mangle]
pub fn fl_image_get_data_impl(data: *const core::ffi::c_void, image: u64) -> FlData {
    let state = &mut unsafe { &mut *(data as *mut WrapState) }.s;
    image_data(state, image)
}

#[no_mangle]
pub fn fl_image_get_status_impl(data: *const core::ffi::c_void, image: u64) -> ImageLoadStatus {
    let state = &mut unsafe { &mut *(data as *mut WrapState) }.s;
    image_status(state, image)
}

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
        let handle = create_from_file(&mut instance.state, "data/png/solid_red.png");

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
        let handle = create_from_file(&mut instance.state, "data/jpeg/green.jpg");

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
    fn gif_animation_ok() {
        let settings = ApplicationSettings {
            width: 0,
            height: 0,
        };
        let mut instance = crate::Instance::new(&settings);
        let handle = create_from_file(&mut instance.state, "data/gif/test.gif");

        wait_for_image_to_load(&mut instance, handle);

        assert_eq!(
            image_status(&instance.state, handle),
            ImageLoadStatus::Loaded
        );
        let info = image_info(&instance.state, handle);
        assert_ne!(info, std::ptr::null());
        let info = unsafe { &*(info as *const ImageInfo) };
        assert_eq!(info.format, ImageFormat::Rgba as u32);
        assert_eq!(info.width, 142);
        assert_eq!(info.height, 142);
        assert_eq!(info.frame_count, 12);
    }

    /*
    TODO: Fix this broken test
    #[test]
    fn png_load_fail() {
        let settings = ApplicationSettings { width: 0, height: 0 };
        let mut instance = crate::Instance::new(&settings);
        let handle = create_from_file(&mut instance.state, "data/png/non_such_file.png");

        wait_for_image_to_load(&mut instance, handle);
        assert!(image_status(&instance.state, handle) == ImageLoadStatus::Failed);
    }
    */

    #[test]
    fn png_load_broken_fail() {
        let settings = ApplicationSettings {
            width: 0,
            height: 0,
        };
        let mut instance = crate::Instance::new(&settings);
        let handle = create_from_file(&mut instance.state, "data/png/broken/xs1n0g01.png");

        wait_for_image_to_load(&mut instance, handle);
        assert!(image_status(&instance.state, handle) == ImageLoadStatus::Failed);
    }
}
