use rgb24::RGB24;

use crate::{colors::YCbCr, image::Resolution};

pub mod rgb24;
mod utils;

pub trait PixelFormat {
    fn get_name(&self) -> &str;
    fn parse_bytestream(&self, bytes: Vec<u8>, resolution: Resolution) -> Vec<Vec<YCbCr>>;
    fn to_bytestream(&self, pixels: Vec<Vec<YCbCr>>) -> Vec<u8>;
}

pub fn get_pixel_format(name: &str) -> Box<dyn PixelFormat> {
    let pixels_formats: Vec<Box<dyn PixelFormat>> = vec![Box::new(RGB24)];

    pixels_formats
        .into_iter()
        .find(|pix_fmt| pix_fmt.get_name() == name)
        .expect(&format!("Unrecognized pixel format {}", name))
}
