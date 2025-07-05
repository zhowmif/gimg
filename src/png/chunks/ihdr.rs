use crate::{
    png::{
        binary_utils::{read_byte, read_u32},
        color_type::ColorType,
        consts::{CHUNK_METADATA_LENGTH, IHDR_CHUNK_TYPE, IHDR_DATA_LENGTH},
        crc::CrcCalculator,
        interlace::InterlaceMethod,
        PngParseError,
    },
    png_assert,
};

use super::Chunk;

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

impl From<&CompressionMethod> for u8 {
    fn from(value: &CompressionMethod) -> Self {
        match value {
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

impl From<&FilterMethod> for u8 {
    fn from(value: &FilterMethod) -> Self {
        match value {
            FilterMethod::Adaptive => 0,
            FilterMethod::NoFilter => 1,
        }
    }
}

#[derive(Debug)]
pub struct Ihdr {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: ColorType,
    pub interlace_method: InterlaceMethod,
    compression_method: CompressionMethod,
    filter_method: FilterMethod,
}

impl Ihdr {
    pub fn new(
        width: u32,
        height: u32,
        color_type: ColorType,
        bit_depth: u8,
        interlace_method: InterlaceMethod,
    ) -> Self {
        Self {
            width,
            height,
            bit_depth,
            color_type,
            compression_method: CompressionMethod::Deflate,
            filter_method: FilterMethod::Adaptive,
            interlace_method,
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

    pub fn check_bit_depth_validity(&self) -> Result<(), PngParseError> {
        self.color_type
            .check_bit_depth_validty(self.bit_depth)
            .map_err(|err| {
                PngParseError(format!(
                    "Invalid bit depth {}, for color type {}",
                    err.1,
                    Into::<u8>::into(&err.0),
                ))
            })
    }

    pub fn check_compatibility(&self) -> Result<(), PngParseError> {
        png_assert!(
            matches!(self.compression_method, CompressionMethod::Deflate),
            "Unsupported compression method".to_string()
        );

        png_assert!(
            matches!(self.filter_method, FilterMethod::Adaptive),
            "Only adaptive filtering is supported".to_string()
        );

        self.check_bit_depth_validity()?;

        Ok(())
    }

    pub fn get_bits_per_pixel(&self) -> usize {
        self.bit_depth as usize * self.color_type.samples_per_pixel()
    }
}
