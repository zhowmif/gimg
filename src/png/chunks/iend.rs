use crate::png::{consts::IEND_CHUNK_TYPE, crc::CrcCalculator};

use super::Chunk;

pub struct Iend;

impl Iend {
    pub fn to_bytes(crc_calculator: &mut CrcCalculator) -> Vec<u8> {
        let chunk = Chunk::new(IEND_CHUNK_TYPE, &[], crc_calculator);

        chunk.to_bytes()
    }
}
