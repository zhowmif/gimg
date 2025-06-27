use std::collections::HashMap;

use crate::colors::{YCbCr, RGB, RGBA};

use super::{deflate::bitsream::WriteBitStream, palette, PngParseError};

#[derive(Debug, Clone, Copy)]
pub enum ColorType {
    Greyscale,
    Truecolor,
    IndexedColor,
    GreyscaleAlpha,
    TrueColorAlpha,
}

impl ColorType {
    pub fn create_scanlines(
        &self,
        pixels: &[Vec<RGBA>],
        bit_depth: u8,
        palette: &Option<HashMap<RGB, (usize, RGB)>>,
    ) -> Vec<Vec<u8>> {
        if bit_depth < 8 {
            return self.create_scanlines_bit_aligned(pixels, bit_depth, palette);
        }

        let mut scanlines: Vec<Vec<u8>> = Vec::with_capacity(pixels.len());

        for row in pixels {
            let mut scanline: Vec<u8> = Vec::with_capacity(row.len() * 4);

            for pixel in row {
                match self {
                    ColorType::IndexedColor => match palette {
                        Some(palette) => {
                            let rgb: RGB = pixel.into();
                            let idx = palette
                                .get(&rgb)
                                .expect("all unique image rgb values must be present in palette")
                                .0 as u8;

                            Self::push_channel_value(&mut scanline, idx, bit_depth);
                        }
                        None => panic!("Palette must be created to encode indexed color image"),
                    },
                    ColorType::Greyscale => {
                        let ycbcr = YCbCr::from(pixel.clone());

                        Self::push_channel_value(&mut scanline, ycbcr.y, bit_depth);
                    }
                    ColorType::GreyscaleAlpha => {
                        let ycbcr = YCbCr::from(pixel.clone());

                        Self::push_channel_value(&mut scanline, ycbcr.y, bit_depth);
                        Self::push_channel_value(&mut scanline, pixel.a, bit_depth);
                    }
                    ColorType::Truecolor => {
                        Self::push_channel_value(&mut scanline, pixel.r, bit_depth);
                        Self::push_channel_value(&mut scanline, pixel.g, bit_depth);
                        Self::push_channel_value(&mut scanline, pixel.b, bit_depth);
                    }
                    ColorType::TrueColorAlpha => {
                        Self::push_channel_value(&mut scanline, pixel.r, bit_depth);
                        Self::push_channel_value(&mut scanline, pixel.g, bit_depth);
                        Self::push_channel_value(&mut scanline, pixel.b, bit_depth);
                        Self::push_channel_value(&mut scanline, pixel.a, bit_depth);
                    }
                }
            }

            scanlines.push(scanline);
        }

        scanlines
    }

    fn push_channel_value(scanline: &mut Vec<u8>, value: u8, bit_depth: u8) {
        if bit_depth == 8 {
            scanline.push(value);
        } else {
            scanline.push(value);
            scanline.push(value);
        }
    }

    fn create_scanlines_bit_aligned(
        &self,
        pixels: &[Vec<RGBA>],
        bit_depth: u8,
        palette: &Option<HashMap<RGB, (usize, RGB)>>,
    ) -> Vec<Vec<u8>> {
        let mut scanlines: Vec<Vec<u8>> = Vec::with_capacity(pixels.len());

        for row in pixels {
            let mut scanline = WriteBitStream::new();

            for pixel in row {
                match self {
                    ColorType::Greyscale => {
                        let ycbcr = YCbCr::from(pixel.clone());
                        let greyscale_adjusted_to_depths = ycbcr.y >> (8 - bit_depth);

                        scanline.push_u8_rtl(greyscale_adjusted_to_depths, bit_depth);
                    },
                    ColorType::IndexedColor => match palette {
                        Some(palette) => {
                            let rgb: RGB = pixel.into();
                            let idx = palette
                                .get(&rgb)
                                .expect("all unique image rgb values must be present in palette")
                                .0 as u8;

                            scanline.push_u8_rtl(idx, bit_depth)
                        }
                        None => panic!("Palette must be created to encode indexed color image"),
                    },
                    _ => panic!("Only greyscale and indexed color should be used with less than 8 bit depth")
                }
            }

            scanlines.push(scanline.flush_to_bytes());
        }

        scanlines
    }
}

impl TryFrom<u8> for ColorType {
    type Error = PngParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Greyscale),
            2 => Ok(Self::Truecolor),
            3 => Ok(Self::IndexedColor),
            4 => Ok(Self::GreyscaleAlpha),
            6 => Ok(Self::TrueColorAlpha),
            _ => Err(PngParseError(format!("Unrecognized color type {value}"))),
        }
    }
}

impl Into<u8> for &ColorType {
    fn into(self) -> u8 {
        match self {
            ColorType::Greyscale => 0,
            ColorType::Truecolor => 2,
            ColorType::IndexedColor => 3,
            ColorType::GreyscaleAlpha => 4,
            ColorType::TrueColorAlpha => 6,
        }
    }
}
