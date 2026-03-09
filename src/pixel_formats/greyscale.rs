use crate::colors::Rgb;

use super::{PixelFormat, RGB24P};

pub struct Greyscale {
    bit_depth: u8,
    pixels: Vec<Vec<u8>>,
}

impl From<Greyscale> for RGB24P {
    fn from(value: Greyscale) -> Self {
        Self {
            pixels: value
                .pixels
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|grey| Rgb::new(grey, grey, grey))
                        .collect()
                })
                .collect(),
        }
    }
}

impl From<RGB24P> for Greyscale {
    fn from(value: RGB24P) -> Self {
        Self {
            bit_depth: 8,
            pixels: value
                .pixels
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|Rgb { r, g, b }| {
                            let grey = r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114;

                            grey.round() as u8
                        })
                        .collect()
                })
                .collect(),
        }
    }
}

impl PixelFormat for Greyscale {}
