use rgb24::RGB24;
use yuv420p::YUV420p;

use crate::{colors::YCbCr, image::Resolution};

pub mod rgb24;
mod utils;
pub mod yuv420p;

pub trait PixelFormat {
    fn get_name(&self) -> &str;
    fn parse_bytestream(&self, bytes: &[u8], resolution: Resolution) -> Vec<Vec<YCbCr>>;
    fn to_bytestream(&self, pixels: Vec<Vec<YCbCr>>) -> Vec<u8>;
}

pub fn get_pixel_format(name: &str) -> Box<dyn PixelFormat> {
    let pixels_formats: Vec<Box<dyn PixelFormat>> = vec![Box::new(RGB24), Box::new(YUV420p)];

    pixels_formats
        .into_iter()
        .find(|pix_fmt| pix_fmt.get_name() == name)
        .unwrap_or_else(|| panic!("Unrecognized pixel format {name}"))
}
