use super::{
    binary_utils::{read_bytes, read_u32},
    consts::CHUNK_METADATA_LENGTH,
    crc::CrcCalculator,
    PngParseError,
};

pub mod ihdr;
pub mod idat;
pub mod iend;

#[derive(Debug)]
pub struct Chunk<'a> {
    chunk_type: &'a [u8],
    chunk_data: &'a [u8],
    crc: u32,
}

impl<'a> Chunk<'a> {
    pub fn new(
        chunk_type: &'a [u8],
        chunk_data: &'a [u8],
        crc_calculator: &mut CrcCalculator,
    ) -> Chunk<'a> {
        crc_calculator.reset();
        crc_calculator.update_crc(chunk_type);
        crc_calculator.update_crc(chunk_data);
        let crc = crc_calculator.get_crc();
        crc_calculator.reset();

        Self {
            chunk_type,
            chunk_data,
            crc,
        }
    }

    pub fn from_bytes(bytes: &'a [u8], offset: &mut usize) -> Result<Chunk<'a>, PngParseError> {
        if bytes.len() < CHUNK_METADATA_LENGTH {
            return Err(PngParseError(format!(
                "Cannot parse chunk as it is smaller than {CHUNK_METADATA_LENGTH} bytes"
            )));
        }

        let length: u32 = read_u32(offset, bytes);

        if bytes.len() < length as usize + CHUNK_METADATA_LENGTH {
            return Err(PngParseError(format!(
                "Cannot parse chunk as it is smaller than specified length {length}"
            )));
        }

        let chunk_type = read_bytes(offset, bytes, 4);
        let chunk_data = read_bytes(offset, bytes, length as usize);
        let crc = read_u32(offset, bytes);

        Ok(Chunk {
            chunk_type,
            chunk_data,
            crc,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let length = self.chunk_data.len();
        let mut result = Vec::with_capacity(CHUNK_METADATA_LENGTH + length);

        result.extend_from_slice(&(length as u32).to_be_bytes());
        result.extend_from_slice(self.chunk_type);
        result.extend_from_slice(self.chunk_data);
        result.extend_from_slice(&self.crc.to_be_bytes());

        result
    }
}
