use crate::png::{consts::IDAT_CHUNK_TYPE, crc::CrcCalculator};

use super::Chunk;

pub struct Idat;

impl Idat {
    pub fn encode_bytes(bytes: &[u8], crc_calculator: &mut CrcCalculator) -> Vec<u8> {
        let chunk = Chunk::new(IDAT_CHUNK_TYPE, bytes, crc_calculator);

        chunk.to_bytes()
    }
}
