use crate::{algebra::align_up, png::adler32::Adler32Calculator};

use super::{bitsream::WriteBitStream, DeflateBlockType, DeflateEncoder};

pub struct ZlibEncoder {
    deflate_encoder: DeflateEncoder,
    adler32_calculator: Adler32Calculator,
}

impl ZlibEncoder {
    pub fn new() -> Self {
        Self {
            deflate_encoder: DeflateEncoder::new(DeflateBlockType::DynamicHuffman),
            adler32_calculator: Adler32Calculator::new(),
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.deflate_encoder.write_bytes(bytes);
        self.adler32_calculator.update_adler32(bytes);
    }

    pub fn flush(&mut self) -> Vec<u8> {
        let adler32 = self.adler32_calculator.get_adler32();
        self.adler32_calculator.reset();

        let mut zlib_bitstream = WriteBitStream::new();
        let cm = 8;
        let cminfo = 7;
        let cmf = (cminfo << 4) as u8 + cm as u8;
        let fdict = 0;
        let flevel = 0;
        let flg = (cmf as u32) << 8 + (flevel << 6) + (fdict << 5);
        let fcheck = (align_up(flg as usize, 31) - flg as usize) as u8;

        zlib_bitstream.push_u8_rtl(cm, 4);
        zlib_bitstream.push_u8_rtl(cminfo, 4);
        zlib_bitstream.push_u8_rtl(fcheck, 5);
        zlib_bitstream.push_u8_rtl(fdict, 1);
        zlib_bitstream.push_u8_rtl(flevel, 2);

        let mut result = zlib_bitstream.flush_to_bytes();
        result.extend_from_slice(&self.deflate_encoder.finish().flush_to_bytes());
        result.extend_from_slice(&adler32.to_be_bytes());

        result
    }
}
