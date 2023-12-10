use crate::image::{ImageFormat, ImageInfo, ImageLoadStatus};
use crate::manual::{FlString, FlData};
use crate::InternalState;
use fileorama::{MemoryDriver, MemoryDriverType,  Error, FilesDirs, LoadStatus, Progress, Fileorama, RecvMsg};
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
        format: format as u32,
        width: dimensions.0 as u32,
        height: dimensions.1 as u32,
        frame_count: 1,
    };

    // Write header at the start of the data
    let write_image_info: &mut ImageInfo = unsafe { std::mem::transmute(&mut output_data[0]) };
    *write_image_info = image_info;

    Ok(output_data)
}

fn load_png_from_memory(data: &[u8]) -> Result<Vec<u8>, String> {
    match decode_png(data) {
        Ok(data) => Ok(data),
        Err(e) => Err(format!("Error loading png: {:?}", e)),
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

#[derive(Debug)]
enum LoadedData {
    Data(Vec<u8>),
    Error(String),
}

#[repr(C)]
pub(crate) struct ImageHandler {
    /// Images that are currently being loaded (i.e async)
    pub(crate) inflight: Vec<(u64, fileorama::Handle)>,
    /// Images that have been loaded
    pub(crate) loaded: HashMap<u64, LoadedData>,
    id_counter: u64,
}

impl ImageHandler {
    pub fn new(vfs: &Fileorama) -> Self {
        vfs.add_memory_driver(Box::<ImageLoader>::default());

        Self {
            inflight: Vec::new(),
            loaded: HashMap::new(),
            id_counter: 1,
        }
    }

    pub fn is_loaded(&self, id: u64) -> bool {
        self.loaded.contains_key(&id)
    }

    pub fn update(&mut self) {
        let len = self.inflight.len();

        for i in 0..len {
            let (id, handle) = &self.inflight[i];
            match handle.recv.try_recv() {
                Ok(RecvMsg::ReadProgress(_progress)) => { },

                Ok(RecvMsg::ReadDone(data)) => {
                    // TODO: handle ownership
                    self.loaded.insert(*id, LoadedData::Data(data.get().to_vec()));
                    self.inflight.remove(i);
                }

                Ok(RecvMsg::Error(e)) => {
                    self.loaded.insert(*id, LoadedData::Error(e));
                    self.inflight.remove(i);
                }

                Ok(RecvMsg::NotFound) => { },

                _ => { },
                Err(_) => { },
            }
        }
    }
}

fn load_sync(url: &str) -> Result<Vec<u8>, String> {
    let data = match std::fs::read(url) {
        Ok(data) => data,
        Err(e) => return Err(format!("{:?}", e)),
    };

    load_png_from_memory(&data)
}

#[inline]
fn create_from_file_sync(state: &mut InternalState, filename: &str) -> u64 {
    let id = state.image_handler.id_counter;

    match load_sync(filename) {
        Ok(data) => state.image_handler.loaded.insert(id, LoadedData::Data(data)),
        Err(e) => state.image_handler.loaded.insert(id, LoadedData::Error(e)),
    };

    state.image_handler.id_counter += 1;
    id
}

#[inline]
fn create_from_file(state: &mut InternalState, filename: &str) -> u64 {
    let handle = state.vfs.load_url(filename);
    let id = state.image_handler.id_counter;
    state.image_handler.inflight.push((id, handle));
    state.image_handler.id_counter += 1;
    id
}

#[inline]
fn image_status(state: &InternalState, id: u64) -> ImageLoadStatus {
    if let Some(image) = state.image_handler.loaded.get(&id) {
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
    if let Some(image) = state.image_handler.loaded.get(&id) {
        match image {
            LoadedData::Data(data) => {
                let header_size = std::mem::size_of::<ImageInfo>();
                let data = &data[header_size..];
                FlData {
                    data: data.as_ptr() as *const core::ffi::c_void,
                    size: data.len() as u64,
                }
            },
            LoadedData::Error(_) => FlData::default(),
        }
    } else {
        FlData::default()
    }
}

#[inline]
fn image_info(state: &InternalState, image_id: u64) -> *const ImageInfo {
    if let Some(image_data) = state.image_handler.loaded.get(&image_id) {
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
pub fn fl_image_create_from_file_block_impl(data: *mut core::ffi::c_void, filename: FlString) -> u64 {
    let state = &mut unsafe { &mut *(data as *mut WrapState) }.s;
    let name = filename.as_str();
    create_from_file_sync(state, name)
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
    use crate::ApplicationSettings;
    use super::*;

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
        let data = unsafe { std::slice::from_raw_parts(data.data as *const u8, data.size as usize) };
        assert_eq!(data[0], 255);
        assert_eq!(data[1], 0);
        assert_eq!(data[2], 0);
    }

    #[test]
    fn png_sync_fail_load() {
        let settings = ApplicationSettings { width: 0, height: 0 };
        let mut instance = crate::Instance::new(&settings);
        let handle = create_from_file_sync(&mut instance.state, "data/png/broken/xs1n0g01.png");
        assert!(image_status(&instance.state, handle) == ImageLoadStatus::Failed);
    }

    #[test]
    fn png_sync_load_ok() {
        let settings = ApplicationSettings { width: 0, height: 0 };
        let mut instance = crate::Instance::new(&settings);
        let handle = create_from_file_sync(&mut instance.state, "data/png/rgb.png");
        assert!(image_status(&instance.state, handle) == ImageLoadStatus::Loaded);
    }

    #[test]
    fn png_sync_red_image_ok() {
        let settings = ApplicationSettings { width: 0, height: 0 };
        let mut instance = crate::Instance::new(&settings);
        let handle = create_from_file_sync(&mut instance.state, "data/png/solid_red.png");
        validate_red_image(&instance.state, handle);
    }

    #[test]
    fn png_async_red_image_ok() {
        let settings = ApplicationSettings { width: 0, height: 0 };
        let mut instance = crate::Instance::new(&settings);
        let handle = create_from_file(&mut instance.state, "data/png/solid_red.png");

        // assume we haven't loaded it just yet
        assert_eq!(instance.state.image_handler.is_loaded(handle), false);

        for _ in 0..100 {
            instance.state.image_handler.update();

            if instance.state.image_handler.is_loaded(handle) {
                validate_red_image(&instance.state, handle);
                return;
            }

            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // should never get here
        assert!(false);
    }

    /*
    #[test]
    fn png_async_load_fail() {
        let settings = ApplicationSettings { width: 0, height: 0 };
        let mut instance = crate::Instance::new(&settings);
        let handle = create_from_file(&mut instance.state, "data/png/non_such_file.png");

        // assume we haven't loaded it just yet
        assert_eq!(instance.state.image_handler.is_loaded(handle), false);

        for _ in 0..100 {
            instance.state.image_handler.update();

            if instance.state.image_handler.is_loaded(handle) {
                assert!(image_status(&instance.state, handle) == ImageLoadStatus::Failed);
                return;
            }

            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // should never get here
        assert!(false);
    }
    */

    /*
    #[test]
    fn png_async_load_fail_broken() {
        let settings = ApplicationSettings { width: 0, height: 0 };
        let mut instance = crate::Instance::new(&settings);
        let handle = create_from_file(&mut instance.state, "data/png/broken/xs1n0g01.png");

        // assume we haven't loaded it just yet
        assert_eq!(instance.state.image_handler.is_loaded(handle), false);

        for _ in 0..100 {
            instance.state.image_handler.update();

            if instance.state.image_handler.is_loaded(handle) {
                dbg!(image_status(&instance.state, handle));
                assert!(image_status(&instance.state, handle) == ImageLoadStatus::Failed);
                return;
            }

            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // should never get here
        assert!(false);
    }
    */
}
