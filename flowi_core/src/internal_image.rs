use bgfx_rs::bgfx;
use bgfx_rs::bgfx::{Memory, Texture};
use png::Decoder;
use std::{collections::HashMap, fs::File};

use crate::{
    image::{ImageFfiApi, ImageFormat},
    internal_error::InternalError as Error,
    internal_error::InternalResult as Result,
    manual::FlString,
};

struct InternalImage {
    pub(crate) _texture: Texture,
    pub(crate) _format: ImageFormat,
}

pub(crate) struct ImageHandler {
    images: HashMap<u64, InternalImage>,
    image_counter: u64,
}

impl ImageHandler {
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
            image_counter: 1,
        }
    }

    pub(crate) fn load(&mut self, path: &str) -> Result<u64> {
        let file = File::open(path)?;
        let decoder = Decoder::new(file);
        let mut reader = decoder.read_info()?;
        let mut img_data = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut img_data)?;

        if info.bit_depth != png::BitDepth::Eight {
            return Err(Error::GenericError {
                text: format!("Png: Only images with 8-bit supported: {}", path),
            });
        }

        let format = match info.color_type {
            png::ColorType::Rgba => ImageFormat::Rgba8,
            png::ColorType::Rgb => ImageFormat::Rgb8,
            png::ColorType::Grayscale => ImageFormat::Alpha8,
            _ => {
                return Err(Error::GenericError {
                    text: format!("Png: Unsupported color type: {}", path),
                })
            }
        };

        self.create_from_memory(info.width, info.height, format, &img_data)
    }

    pub fn get_ffi_api(&self) -> ImageFfiApi {
        ImageFfiApi {
            data: self as *const ImageHandler as *const std::ffi::c_void,
            load: fl_image_load_impl,
            create_from_memory: fl_image_create_from_memory_impl,
        }
    }

    pub fn create_from_memory(
        &mut self,
        width: u32,
        height: u32,
        format: ImageFormat,
        data: &[u8],
    ) -> Result<u64> {
        // TODO: Validate that data is the correct size
        let texture = match format {
            ImageFormat::Rgba8 => bgfx::create_texture_2d(
                width as _,
                height as _,
                false,
                1,
                bgfx::TextureFormat::RGBA8,
                0,
                &Memory::copy(&data),
            ),

            ImageFormat::Rgb8 => bgfx::create_texture_2d(
                width as _,
                height as _,
                false,
                1,
                bgfx::TextureFormat::RGB8,
                0,
                &Memory::copy(&data),
            ),

            ImageFormat::Alpha8 => bgfx::create_texture_2d(
                width as _,
                height as _,
                false,
                1,
                bgfx::TextureFormat::R8,
                0,
                &Memory::copy(&data),
            ),
        };

        self.create_from_texture(texture, format)
    }

    fn create_from_texture(&mut self, _texture: Texture, _format: ImageFormat) -> Result<u64> {
        let image_id = self.image_counter;

        self.images
            .insert(image_id, InternalImage { _texture, _format });
        self.image_counter += 1;

        Ok(image_id)
    }
}

#[no_mangle]
pub unsafe extern "C" fn fl_image_load_impl(
    ctx: *const core::ffi::c_void,
    filename: FlString,
) -> u64 {
    let image_handler = unsafe { &mut *(ctx as *mut ImageHandler) };
    // TODO: correct handling
    image_handler.load(filename.as_str()).unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn fl_image_create_from_memory_impl(
    ctx: *const core::ffi::c_void,
    width: u32,
    height: u32,
    format: ImageFormat,
    data: *const u8,
    data_size: u32,
) -> u64 {
    let image_handler = unsafe { &mut *(ctx as *mut ImageHandler) };
    let mem = std::slice::from_raw_parts(data, data_size as usize);
    // TODO: correct handling
    image_handler
        .create_from_memory(width, height, format, mem)
        .unwrap()
}

#[no_mangle]
pub extern "C" fn fl_get_image_api(
    app_state: *const core::ffi::c_void,
    _version: u32,
) -> *const ImageFfiApi {
    let app_state = unsafe { &*(app_state as *const crate::application::ApplicationState) };
    &app_state.image_ffi_api
}
