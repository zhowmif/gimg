use greyscale::Greyscale;
use crate::colors::Rgb;

mod greyscale;

pub trait PixelFormat: From<RGB24P> + Into<RGB24P> {
    fn to_grayscale(self) -> Greyscale {
        let rgb: RGB24P = self.into();

        rgb.into()
    }
}

pub struct RGB24P {
    pixels: Vec<Vec<Rgb>>
}

impl PixelFormat for RGB24P {}

pub struct YCbCr420p {}

impl From<RGB24P> for YCbCr420p {
    fn from(value: RGB24P) -> Self {
        todo!()
    }
}

impl From<YCbCr420p> for RGB24P {
    fn from(value: YCbCr420p) -> Self {
        todo!()
    }
}

impl PixelFormat for YCbCr420p {}
