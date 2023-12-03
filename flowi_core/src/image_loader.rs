use fileorama::{Driver, Error, FilesDirs, LoadStatus, Progress, DriverType};
use crate::image::{ImageInfo, ImageFormat};
/*
use zune_jpeg::{
    JpegDecoder, 
    zune_core::options::DecoderOptions as JpegDecoderOptions,
    zune_core::colorspace::ColorSpace as JpegColorSpace,
    errors::DecodeErrors as JpegDecodeErrors
};
*/

use zune_png::{
    PngDecoder, 
    error::PngDecodeErrors as PngDecodeErrors, 
    zune_core::colorspace::ColorSpace as PngColorSpace,
    zune_core::bit_depth::BitDepth as PngBitDepth,
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
    let mut decoder = PngDecoder::new(&data);
    decoder.decode_headers()?;

    let Some(depth) = decoder.get_depth() else {
        return Err(PngDecodeErrors::Generic(format!("Could not get depth")));
    };

   let Some(buffer_size) = decoder.output_buffer_size() else { 
        return Err(PngDecodeErrors::Generic(format!("Could not get output buffer size")));
    };

    let Some(color_space) = decoder.get_colorspace() else { 
        return Err(PngDecodeErrors::Generic(format!("Could not get colorspace")));
    };

    let Some(dimensions) = decoder.get_dimensions() else { 
        return Err(PngDecodeErrors::Generic(format!("Could not get dimensions")));
    };

    // Only supporting 8-bit PNGs for now
    if depth != PngBitDepth::Eight {
        return Err(PngDecodeErrors::Generic(format!("Unsupported depth: {:?}", depth)));
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
        _ => return Err(PngDecodeErrors::Generic(format!("Unknown colorspace: {:?}", color_space))),
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

impl Driver for ImageLoader {
    /// We return true here as we don't know
    fn is_remote(&self) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "flowi_image_loader"
    }

    fn supports_url(&self, _url: &str) -> bool {
        // We always let other drivers load to memory
        false
    }

    // Create a new instance given data. The Driver will take ownership of the data
    fn create_instance(&self) -> DriverType {
        Box::new(ImageLoader::default())
    }

    // Get some data in and returns true if driver can be mounted from it
    fn can_load_from_data(&self, data: &[u8]) -> bool {
        let mut png_decoder = PngDecoder::new(&data);
        let headers = png_decoder.decode_headers();
        headers.is_ok()
    }

    // Create a new instance given data. The Driver will take ownership of the data
    fn create_from_data(&self, data: Box<[u8]>) -> Option<DriverType> {
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

    fn can_load_from_url(&self, _url: &str) -> bool {
        false
    }

    /// Used when creating an instance of the driver with a path to load from
    fn create_from_url(&self, _url: &str) -> Option<DriverType> {
        None
    }

    /// Returns a handle which updates the progress and returns the loaded data. This will try to
    fn load_url(
        &mut self,
        _path: &str,
        progress: &mut Progress,
    ) -> Result<LoadStatus, Error> {
        //trace!("loading url: {} for image loader", path);
        progress.set_step(1);

        match self.image_type {
            ImageType::PngData(ref data) => {
                match decode_png(data) {
                    Ok(data) => {
                        progress.step()?;
                        Ok(LoadStatus::Data(data.into_boxed_slice()))
                    },

                    Err(e) => {
                        progress.step()?;
                        Err(Error::Generic(format!("Error loading png: {:?}", e)))
                    },
                }
            }

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

