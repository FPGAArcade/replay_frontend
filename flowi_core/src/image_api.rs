use crate::image::{ImageFormat, ImageInfo};
use crate::manual::FlString;
use crate::InternalState;
use fileorama::{MemoryDriver, MemoryDriverType,  Error, FilesDirs, LoadStatus, Progress, Fileorama};
use std::collections::HashMap;

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

#[derive(Default, Debug)]
enum ImageType {
    PngData(Box<[u8]>),
    //JpegData(Box<[u8]>),
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

fn load_png_from_memory(data: &[u8]) -> Result<Vec<u8>, Error> {
    match decode_png(data) {
        Ok(data) => Ok(data),
        Err(e) => Err(Error::Generic(format!("Error loading png: {:?}", e))),
    }
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

impl MemoryDriver for ImageLoader {
    fn name(&self) -> &'static str {
        dbg!();
        "flowi_image_loader"
    }

    // Create a new instance given data. The Driver will take ownership of the data
    fn create_instance(&self) -> MemoryDriverType {
        Box::<ImageLoader>::default()
    }

    // Get some data in and returns true if driver can be mounted from it
    fn can_create_from_data(&self, data: &[u8]) -> bool {
        let mut png_decoder = PngDecoder::new(data);
        let headers = png_decoder.decode_headers();
        headers.is_ok()
    }

    // Create a new instance given data. The Driver will take ownership of the data
    fn create_from_data(&self, data: Box<[u8]>) -> Option<MemoryDriverType> {
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

    /// Returns a handle which updates the progress and returns the loaded data. This will try to
    fn load(&mut self, _path: &str, progress: &mut Progress) -> Result<LoadStatus, Error> {
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

#[repr(C)]
pub(crate) struct ImageHandler {
    /// Images that are currently being loaded (i.e async)
    inflight: Vec<(u64, fileorama::Handle)>,
    /// Images that have been loaded
    loaded: HashMap<u64, Vec<u8>>,
    id_counter: u64,
}

impl ImageHandler {
    pub fn new(vfs: &Fileorama) -> Self {
        vfs.add_memory_driver(Box::new(ImageLoader::default()));

        Self {
            inflight: Vec::new(),
            loaded: HashMap::new(),
            id_counter: 1,
        }
    }

    pub fn is_loaded(&self, id: u64) -> bool {
        self.loaded.contains_key(&id)
    }

    /*
    pub fn update(&mut self) {
        for (id, handle) in self.inflight {
            match handle.recv() {
                RecvMsg::Progress(progress) => {
                    println!("progress: {}", progress);
                }

                RecvMsg::Data(data) => {
                    self.loaded.insert(id, data);
                }

                RecvMsg::Error(e) => {
                    println!("error: {:?}", e);
                }

                RecvMsg::Done => {
                    println!("done");
                }
            }
        }
    }
    */
}

fn load_sync(url: &str) -> Result<Vec<u8>, fileorama::Error> {
    let data = match std::fs::read(url) {
        Ok(data) => data,
        Err(e) => return Err(Error::Generic(format!("{:?}", e))),
    };

    load_png_from_memory(&data)
}

#[inline]
fn create_from_file_sync(state: &mut InternalState, filename: &str) -> Result<u64, Error> {
    let data = load_sync(filename)?;
    let id = state.image_handler.id_counter;
    state.image_handler.loaded.insert(id, data);
    state.image_handler.id_counter += 1;
    Ok(id)
}

#[inline]
fn create_from_file(state: &mut InternalState, filename: &str) -> Result<u64, Error> {
    let handle = state.vfs.load_url(filename);
    let id = state.image_handler.id_counter;
    state.image_handler.inflight.push((id, handle));
    state.image_handler.id_counter += 1;
    Ok(id)
}

struct WrapState<'a> {
    s: &'a mut crate::InternalState,
}

// FFI functions
#[no_mangle]
pub fn fl_image_create_from_file_block_impl(data: *mut core::ffi::c_void, filename: FlString) -> u64 {
    let state = &mut unsafe { &mut *(data as *mut WrapState) }.s;
    let name = filename.as_str();
    create_from_file_sync(state, name).unwrap_or_else(|e| {
        panic!("{:?}", e);
    })
}

// FFI functions
#[no_mangle]
pub fn fl_image_create_from_file_impl(data: *mut core::ffi::c_void, filename: FlString) -> u64 {
    let state = &mut unsafe { &mut *(data as *mut WrapState) }.s;
    let name = filename.as_str();
    create_from_file(state, name).unwrap_or_else(|e| {
        panic!("{:?}", e);
    })
}

#[no_mangle]
pub fn fl_image_get_info_impl(_data: *const core::ffi::c_void, _image: u64) -> *const ImageInfo {
    std::ptr::null()
}
