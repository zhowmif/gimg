use crate::{
    png::{
        binary_utils::{read_byte, read_u32},
        consts::{CHUNK_METADATA_LENGTH, IHDR_CHUNK_TYPE, IHDR_DATA_LENGTH},
        crc::CrcCalculator,
        PngParseError,
    },
    png_assert,
};

use super::Chunk;

#[derive(Debug)]
enum ColorType {
    Greyscale,
    Truecolor,
    IndexedColor,
    GreyscaleAlpha,
    TrueColorAlpha,
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

#[derive(Debug)]
enum CompressionMethod {
    Deflate,
    Other,
}

impl From<u8> for CompressionMethod {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Deflate,
            _ => Self::Other,
        }
    }
}

impl Into<u8> for &CompressionMethod {
    fn into(self) -> u8 {
        match self {
            CompressionMethod::Deflate => 0,
            CompressionMethod::Other => 1,
        }
    }
}

#[derive(Debug)]
enum FilterMethod {
    Adaptive,
    NoFilter,
}

impl From<u8> for FilterMethod {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Adaptive,
            _ => Self::NoFilter,
        }
    }
}

impl Into<u8> for &FilterMethod {
    fn into(self) -> u8 {
        match self {
            FilterMethod::Adaptive => 0,
            FilterMethod::NoFilter => 1,
        }
    }
}

#[derive(Debug)]
enum InterlaceMethod {
    NoInterlace,
    Adam7,
}

impl TryFrom<u8> for InterlaceMethod {
    type Error = PngParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NoInterlace),
            1 => Ok(Self::Adam7),
            _ => Err(PngParseError(format!(
                "Unrecognized interlace method {value}"
            ))),
        }
    }
}

impl Into<u8> for &InterlaceMethod {
    fn into(self) -> u8 {
        match self {
            InterlaceMethod::NoInterlace => 0,
            InterlaceMethod::Adam7 => 1,
        }
    }
}

#[derive(Debug)]
pub struct IHDR {
    pub width: u32,
    pub height: u32,
    bit_depth: u8,
    color_type: ColorType,
    compression_method: CompressionMethod,
    filter_method: FilterMethod,
    interlace_method: InterlaceMethod,
}

impl IHDR {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            bit_depth: 8,
            color_type: ColorType::TrueColorAlpha,
            compression_method: CompressionMethod::Deflate,
            filter_method: FilterMethod::Adaptive,
            interlace_method: InterlaceMethod::NoInterlace,
        }
    }

    pub fn to_bytes(&self, crc_calculator: &mut CrcCalculator) -> Vec<u8> {
        let mut data = Vec::with_capacity(CHUNK_METADATA_LENGTH + IHDR_DATA_LENGTH);
        data.extend_from_slice(&self.width.to_be_bytes());
        data.extend_from_slice(&self.height.to_be_bytes());
        data.push(self.bit_depth);
        data.push((&self.color_type).into());
        data.push((&self.compression_method).into());
        data.push((&self.filter_method).into());
        data.push((&self.interlace_method).into());
        let chunk = Chunk::new(IHDR_CHUNK_TYPE, &data, crc_calculator);

        chunk.to_bytes()
    }

    pub fn from_chunk(chunk: Chunk) -> Result<Self, PngParseError> {
        png_assert!(
            *chunk.chunk_type == *IHDR_CHUNK_TYPE,
            format!("Expected IHDR chunk, found {:?}", chunk.chunk_type)
        );

        png_assert!(
            chunk.chunk_data.len() == IHDR_DATA_LENGTH,
            format!(
                "Invalid IHDR chunk size, expected {}, received {}",
                IHDR_DATA_LENGTH,
                chunk.chunk_data.len()
            )
        );

        let mut offset = 0;
        let width = read_u32(&mut offset, chunk.chunk_data);
        let height = read_u32(&mut offset, chunk.chunk_data);
        let bit_depth = read_byte(&mut offset, chunk.chunk_data);
        let color_type = ColorType::try_from(read_byte(&mut offset, chunk.chunk_data))?;
        let compression_method = CompressionMethod::from(read_byte(&mut offset, chunk.chunk_data));
        let filter_method = FilterMethod::from(read_byte(&mut offset, chunk.chunk_data));
        let interlace_method = InterlaceMethod::try_from(read_byte(&mut offset, chunk.chunk_data))?;

        Ok(Self {
            width,
            height,
            bit_depth,
            color_type,
            compression_method,
            filter_method,
            interlace_method,
        })
    }

    pub fn check_compatibility(&self) -> Result<(), PngParseError> {
        png_assert!(
            matches!(self.color_type, ColorType::TrueColorAlpha),
            format!("Unsupported color type {:?}", self.color_type)
        );

        png_assert!(
            matches!(self.compression_method, CompressionMethod::Deflate),
            "Unsupported compression method".to_string()
        );

        png_assert!(
            matches!(self.filter_method, FilterMethod::Adaptive),
            "Only adaptive filtering is supported".to_string()
        );

        png_assert!(
            matches!(self.interlace_method, InterlaceMethod::NoInterlace),
            "Interlacing is not supported".to_string()
        );

        Ok(())
    }

    pub fn get_bits_per_pixel(&self) -> usize {
        let samples_per_pixel: usize = match self.color_type {
            ColorType::Greyscale => 1,
            ColorType::Truecolor => 3,
            ColorType::IndexedColor => 1,
            ColorType::GreyscaleAlpha => 1,
            ColorType::TrueColorAlpha => 4,
        };

        self.bit_depth as usize * samples_per_pixel
    }
}
