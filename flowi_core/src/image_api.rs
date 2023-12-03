use crate::image::{ImageFormat, ImageInfo};
use crate::manual::FlString;
use crate::InternalState;
//use fileorama::{Driver, DriverType, Error, FilesDirs, LoadStatus, Progress, Fileorama, RecvMsg};
use std::collections::HashMap;
use std::{thread, time};

/*
use zune_jpeg::{
    JpegDecoder,
    zune_core::options::DecoderOptions as JpegDecoderOptions,
    zune_core::colorspace::ColorSpace as JpegColorSpace,
    errors::DecodeErrors as JpegDecodeErrors
};
*/

use zune_png::{
    error::PngDecodeErrors, zune_core::bit_depth::BitDepth as PngBitDepth,
    zune_core::colorspace::ColorSpace as PngColorSpace, PngDecoder,
};

struct Test {
    int: u8,
    string: String,
    option: Option<Vec<i32>>,
}

#[derive(Default, Debug)]
enum ImageType {
    PngData(Box<[u8]>),
    JpegData(Box<[u8]>),
    #[default]
    None,
}

#[derive(Default, Debug)]
struct ImageLoader {
    image_type: ImageType,
}

fn decode_png(data: &[u8]) -> Result<Vec<u8>, PngDecodeErrors> {
    let mut decoder = PngDecoder::new(data);
    decoder.decode_headers()?;

    // unwraping here is safe as we have already checked that the headers are ok
    let depth = decoder.get_depth().unwrap();
    let buffer_size = decoder.output_buffer_size().unwrap();
    let color_space = decoder.get_colorspace().unwrap();
    let dimensions = decoder.get_dimensions().unwrap();

    // Only supporting 8-bit PNGs for now
    if depth != PngBitDepth::Eight {
        return Err(PngDecodeErrors::Generic(format!(
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
            return Err(PngDecodeErrors::Generic(format!(
                "Unknown colorspace: {:?}",
                color_space
            )))
        }
    };

    let image_info = ImageInfo {
        image_format: format as u32,
        width: dimensions.0 as u32,
        height: dimensions.1 as u32,
        frame_count: 1,
    };

    // Write header at the start of the data
    let write_image_info: &mut ImageInfo = unsafe { std::mem::transmute(&mut output_data[0]) };
    *write_image_info = image_info;

    Ok(output_data)
}

/*
fn decode_jpeg(data: &[u8]) -> Result<LoadStatus, Error> {
    let opts = JpegDecoderOptions::new_fast()
        .set_color_space(JpegColorSpace::RGB);
    let mut decoder = JpegDecoder::new(data);
    decoder.set_options(opts);
    decoder.decode_headers()?;
    let image_data = decoder.decode()?;
}
*/

/*
impl Driver for ImageLoader {
    /// We return true here as we don't know
    fn is_remote(&self) -> bool {
        dbg!();
        true
    }

    fn name(&self) -> &'static str {
        dbg!();
        "flowi_image_loader"
    }

    fn supports_url(&self, url: &str) -> bool {
        url.ends_with(".png")
        //dbg!();
        // We always let other drivers load to memory
        //false
    }

    // Create a new instance given data. The Driver will take ownership of the data
    fn create_instance(&self) -> DriverType {
        Box::<ImageLoader>::default()
    }

    // Get some data in and returns true if driver can be mounted from it
    fn can_load_from_data(&self, data: &[u8]) -> bool {
        dbg!();
        let mut png_decoder = PngDecoder::new(data);
        let headers = png_decoder.decode_headers();
        headers.is_ok()
    }

    // Create a new instance given data. The Driver will take ownership of the data
    fn create_from_data(&self, data: Box<[u8]>) -> Option<DriverType> {
        dbg!();
        // Check if png or jpeg loader can open the data
        /*
        let jpeg_decoder = JpegDecoder::new(&data);
        let mut headers = jpeg_decoder.decode_headers();
        if headers.is_ok() {
            return Some(Box::new(ImageLoader {
                image_type: ImageType::JpegData(data),
            }));
        }
        */

        let mut png_decoder = PngDecoder::new(&data);
        let headers = png_decoder.decode_headers();
        if headers.is_ok() {
            return Some(Box::new(ImageLoader {
                image_type: ImageType::PngData(data),
            }));
        }

        None
    }

    fn can_load_from_url(&self, url: &str) -> bool {
        url.ends_with(".png")
        //dbg!();
        //false
    }

    /// Used when creating an instance of the driver with a path to load from
    fn create_from_url(&self, _url: &str) -> Option<DriverType> {
        dbg!();
        None
    }

    /// Returns a handle which updates the progress and returns the loaded data. This will try to
    fn load_url(&mut self, _path: &str, progress: &mut Progress) -> Result<LoadStatus, Error> {
        println!("loading url: {} for image loader", _path);
        //progress.set_step(1);

        match self.image_type {
            ImageType::PngData(ref data) => match decode_png(data) {
                Ok(data) => {
                    progress.step()?;
                    Ok(LoadStatus::Data(data.into_boxed_slice()))
                }

                Err(e) => {
                    progress.step()?;
                    Err(Error::Generic(format!("Error loading png: {:?}", e)))
                }
            },

            /*
            ImageType::JpegData(ref data) => {
                let png_decoder = PngDecoder::new(&data);
                png_decoder.decode_headers()?;

            }
            */
            _ => Ok(LoadStatus::NotFound),
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
*/

enum Image {
    //Async(fileorama::Handle),
    Data(Vec<u8>),
}

#[repr(C)]
pub(crate) struct ImageHandler {
    images: HashMap<u64, Image>,
    id_counter: u64,
}

impl ImageHandler {
    //pub fn new(vfs: &Fileorama) -> Self {
    pub fn new() -> Self {
        //vfs.add_driver(Box::new(ImageLoader::default()));

        Self {
            images: HashMap::new(),
            id_counter: 1,
        }
    }
}

/*
fn load_sync(url: &str) -> Result<Vec<u8>, Error> {
//fn load_sync(vfs: &Fileorama, url: &str) -> Result<Vec<u8>, Error> {
    /*
    let handle = vfs.load_url(url);

    for _ in 0..10_000 {
        match handle.recv.try_recv() {
            Ok(RecvMsg::ReadDone(data)) => return Ok(data.get().to_vec()),
            Err(e) => return Err(Error::Generic(format!("{:?}", e))),
            _ => (),
        }

        // sleep for 0.1 ms when waiting for loading to complete
        thread::sleep(std::time::Duration::from_millis(1));
    }

    Err(Error::Generic(format!("Loading of {} didn't complete within timeout", url)))
    */
    Err(Error::Generic(format!("Loading of {} didn't complete within timeout", url)))
}
*/


/*
#[inline]
fn create_from_file_sync(state: &mut InternalState, filename: &str) -> std::io::Result<u64> {
    let data = load_sync(filename)?;
    let id = state.image_handler.id_counter;
    state.image_handler.images.insert(id, Image::Data(data));
    state.image_handler.id_counter += 1;
    Ok(id)
}
*/

/*
#[inline]
fn create_from_file(state: &mut InternalState, filename: &str) -> Result<u64, Error> {
    let handle = state.vfs.load_url(filename);
    let id = state.image_handler.id_counter;
    state.image_handler.images.insert(id, Image::Async(handle));
    state.image_handler.id_counter += 1;
    Ok(id)
}
*/

struct WrapState<'a> {
    s: &'a mut crate::InternalState,
}

// FFI functions
#[no_mangle]
pub fn fl_image_create_from_file_block_impl(data: *mut core::ffi::c_void, filename: FlString) -> u64 {
    let state = &mut unsafe { &mut *(data as *mut WrapState) }.s;
    /*
    let name = filename.as_str();
    create_from_file_sync(state, name).unwrap_or_else(|e| {
        //panic!("{:?}", e);
        // TODO: set_last_error
        return 0;
    })
    */
    0
}

// FFI functions
#[no_mangle]
pub fn fl_image_create_from_file_impl(data: *mut core::ffi::c_void, filename: FlString) -> u64 {
    let state = &mut unsafe { &mut *(data as *mut WrapState) }.s;
    let name = filename.as_str();
    /*
    create_from_file(state, name).unwrap_or_else(|e| {
        panic!("{:?}", e);
        // TODO: set_last_error
        return 0;
    })
    */
    0
}

#[no_mangle]
pub fn fl_image_get_info_impl(_data: *const core::ffi::c_void, _image: u64) -> *const ImageInfo {
    std::ptr::null()
}
