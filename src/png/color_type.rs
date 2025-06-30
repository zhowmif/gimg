use std::{collections::HashMap, u8};

use crate::colors::{YCbCr, RGB, RGBA};

use super::{
    deflate::bitsream::{ReadBitStream, WriteBitStream},
    PngParseError,
};

#[derive(Debug)]
pub struct InvalidBitDepthError(pub ColorType, pub u8);

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
        palette: &Option<HashMap<RGBA, (usize, RGBA)>>,
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
                            let idx = palette
                                .get(&pixel)
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
        palette: &Option<HashMap<RGBA, (usize, RGBA)>>,
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
                            let idx = palette
                                .get(&pixel)
                                .expect("all unique image rgb values must be present in palette")
                                .0 as u8;
                            debug_assert!(idx < (1<<bit_depth));

                            scanline.push_u8_rtl(idx, bit_depth)
                        }
                        None => panic!("Palette must be created to encode indexed color image"),
                    },
                    _ => panic!("scanline_to_pixels called with less than 8 bits for non compatible color type")
                }
            }

            scanlines.push(scanline.flush_to_bytes());
        }

        scanlines
    }

    pub fn scanline_to_pixels(
        &self,
        scanlines: &[Vec<u8>],
        bit_depth: u8,
        width: usize,
        palette: &Option<Vec<RGBA>>,
    ) -> Result<Vec<Vec<RGBA>>, PngParseError> {
        if bit_depth < 8 {
            return self.scanline_to_pixels_bit_aligned(scanlines, bit_depth, width, palette);
        }

        let mut pixels = Vec::with_capacity(scanlines.len());
        let bytes_per_pixel = self.samples_per_pixel() * ((bit_depth as usize) >> 3);

        for scanline in scanlines {
            let mut pixel_row = Vec::with_capacity(scanline.len() / bytes_per_pixel);

            for pixel_bytes in scanline.chunks_exact(bytes_per_pixel) {
                let pixel = match self {
                    ColorType::Greyscale => {
                        let gamma = Self::read_channel_bytes(pixel_bytes, bit_depth, 0);
                        let ycbcr = YCbCr::new(gamma, 127, 127);

                        ycbcr.into()
                    }
                    ColorType::Truecolor => {
                        let r = Self::read_channel_bytes(pixel_bytes, bit_depth, 0);
                        let g = Self::read_channel_bytes(pixel_bytes, bit_depth, 1);
                        let b = Self::read_channel_bytes(pixel_bytes, bit_depth, 2);

                        RGBA::new(r, g, b, u8::MAX)
                    }
                    ColorType::IndexedColor => {
                        let color = &palette
                            .as_ref()
                            .expect("scanline_to_pixels called with indexed color but no palette")
                            [pixel_bytes[0] as usize];

                        color.clone()
                    }
                    ColorType::GreyscaleAlpha => {
                        let gamma = Self::read_channel_bytes(pixel_bytes, bit_depth, 0);
                        let alpha = Self::read_channel_bytes(pixel_bytes, bit_depth, 1);
                        let ycbcr = YCbCr::new(gamma, 127, 127);
                        let rgb: RGB = RGB::from(&ycbcr);

                        RGBA::new(rgb.r, rgb.g, rgb.b, alpha)
                    }
                    ColorType::TrueColorAlpha => {
                        let r = Self::read_channel_bytes(pixel_bytes, bit_depth, 0);
                        let g = Self::read_channel_bytes(pixel_bytes, bit_depth, 1);
                        let b = Self::read_channel_bytes(pixel_bytes, bit_depth, 2);
                        let a = Self::read_channel_bytes(pixel_bytes, bit_depth, 3);

                        RGBA::new(r, g, b, a)
                    }
                };

                pixel_row.push(pixel);
            }

            pixels.push(pixel_row);
        }

        Ok(pixels)
    }

    pub fn check_bit_depth_validty(&self, bit_depth: u8) -> Result<(), InvalidBitDepthError> {
        match self {
            ColorType::Greyscale => self.validate_bit_depth(&[1, 2, 4, 8, 16], bit_depth),
            ColorType::Truecolor => self.validate_bit_depth(&[8, 16], bit_depth),
            ColorType::IndexedColor => self.validate_bit_depth(&[1, 2, 4, 8], bit_depth),
            ColorType::GreyscaleAlpha => self.validate_bit_depth(&[8, 16], bit_depth),
            ColorType::TrueColorAlpha => self.validate_bit_depth(&[8, 16], bit_depth),
        }
    }

    fn validate_bit_depth(
        &self,
        valid_bit_depths: &[u8],
        bit_depth: u8,
    ) -> Result<(), InvalidBitDepthError> {
        if !valid_bit_depths.contains(&bit_depth) {
            return Err(InvalidBitDepthError(*self, bit_depth));
        }

        Ok(())
    }

    fn read_channel_bytes(pixel_bytes: &[u8], bit_depth: u8, value_idx: usize) -> u8 {
        if bit_depth == 8 {
            pixel_bytes[value_idx]
        } else {
            let first = pixel_bytes[value_idx * 2];
            let second = pixel_bytes[value_idx + 1];

            first.saturating_add(second >> 7)
        }
    }

    pub fn samples_per_pixel(&self) -> usize {
        match self {
            ColorType::Greyscale => 1,
            ColorType::Truecolor => 3,
            ColorType::IndexedColor => 1,
            ColorType::GreyscaleAlpha => 2,
            ColorType::TrueColorAlpha => 4,
        }
    }

    fn scanline_to_pixels_bit_aligned(
        &self,
        scanlines: &[Vec<u8>],
        bit_depth: u8,
        width: usize,
        palette: &Option<Vec<RGBA>>,
    ) -> Result<Vec<Vec<RGBA>>, PngParseError> {
        let mut pixels = Vec::with_capacity(scanlines.len());

        for scanline in scanlines {
            let row_size = (self.samples_per_pixel() * scanline.len() * 8) / bit_depth as usize;
            let mut bitstream = ReadBitStream::new(scanline);
            let mut pixel_row = Vec::with_capacity(width);

            while pixel_row.len() < row_size {
                let value = bitstream.read_number_lsb(bit_depth as usize).unwrap() as u8;

                let pixel = match self {
                    ColorType::Greyscale => {
                        let gamma = Self::repeat_value_in_byte(value, bit_depth);
                        let ycbcr = YCbCr::new(gamma, 127, 127);

                        ycbcr.into()
                    }
                    ColorType::IndexedColor => {
                        let color = &palette
                            .as_ref()
                            .expect("scanline_to_pixels called with indexed color but no palette")
                            [value as usize];

                        color.clone()
                    }
                    _ => panic!(
                    "scanline_to_pixels called with less than 8 bits for non compatible color type"
                ),
                };

                pixel_row.push(pixel);
            }

            pixels.push(pixel_row)
        }

        Ok(pixels)
    }

    fn repeat_value_in_byte(value: u8, bit_depth: u8) -> u8 {
        let mut byte = 0;

        for _i in 0..(8 / bit_depth) {
            byte += value;
            byte <<= bit_depth;
        }

        byte
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
