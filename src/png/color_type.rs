use crate::colors::{YCbCr, RGBA};

use super::PngParseError;

#[derive(Debug, Clone, Copy)]
pub enum ColorType {
    Greyscale,
    Truecolor,
    IndexedColor,
    GreyscaleAlpha,
    TrueColorAlpha,
}

impl ColorType {
    pub fn create_scanlines(&self, pixels: &[Vec<RGBA>]) -> Vec<Vec<u8>> {
        let mut scanlines: Vec<Vec<u8>> = Vec::with_capacity(pixels.len());

        for row in pixels {
            let mut scanline: Vec<u8> = Vec::with_capacity(row.len() * 4);

            for pixel in row {
                match self {
                    ColorType::IndexedColor => todo!(),
                    ColorType::Greyscale => {
                        let ycbcr = YCbCr::from(pixel.clone());

                        scanline.push(ycbcr.y);
                    }
                    ColorType::GreyscaleAlpha => {
                        let ycbcr = YCbCr::from(pixel.clone());

                        scanline.push(ycbcr.y);
                        scanline.push(pixel.a);
                    }
                    ColorType::Truecolor => {
                        scanline.push(pixel.r);
                        scanline.push(pixel.g);
                        scanline.push(pixel.b);
                    }
                    ColorType::TrueColorAlpha => {
                        scanline.push(pixel.r);
                        scanline.push(pixel.g);
                        scanline.push(pixel.b);
                        scanline.push(pixel.a);
                    }
                }
            }

            scanlines.push(scanline);
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
