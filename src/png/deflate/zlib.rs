use crate::{
    algebra::align_up,
    deflate_read_bits,
    png::{adler32::Adler32Calculator, CompressionLevel},
};

use super::{
    bitstream::{ReadBitStream, WriteBitStream},
    decode::{decode_deflate, DeflateDecodeError},
    DeflateEncoder,
};

pub struct ZlibEncoder {
    compression_level: CompressionLevel,
    deflate_encoder: DeflateEncoder,
    adler32_calculator: Adler32Calculator,
}

impl ZlibEncoder {
    pub fn new(compression_level: CompressionLevel) -> Self {
        Self {
            deflate_encoder: DeflateEncoder::new(compression_level),
            adler32_calculator: Adler32Calculator::new(),
            compression_level,
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
        let flevel = self.compression_level.to_zlib_u8();
        let flg = ((cmf as u32) << 8) + ((flevel as u32) << 6) + ((fdict as u32) << 5);
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

pub fn decode_zlib(bytes: &[u8]) -> Result<Vec<u8>, DeflateDecodeError> {
    let data_start_index = decode_zlib_header(bytes)?;
    let uncompressed_data = decode_deflate(&bytes[data_start_index..bytes.len() - 4])?;
    let _adler32 = u32::from_be_bytes(bytes[bytes.len() - 4..].try_into().unwrap());
    //TODO: validate adler 32

    Ok(uncompressed_data)
}

pub fn decode_zlib_header(bytes: &[u8]) -> Result<usize, DeflateDecodeError> {
    let mut header_bitstream = ReadBitStream::new(bytes);
    let cm = deflate_read_bits!(
        header_bitstream.read_number_lsb(4),
        "expected ZLIB compression method (CM)"
    );

    if cm != 8 {
        return Err(DeflateDecodeError(format!(
            "Unsupported compression method in ZLIB header {}",
            cm
        )));
    }

    let cminfo = deflate_read_bits!(
        header_bitstream.read_number_lsb(4),
        "expected ZLIB compression info (CMINFO)"
    );

    if cminfo > 7 {
        return Err(DeflateDecodeError(format!(
            "Unsupported compression info value in ZLIB header: {}",
            cminfo
        )));
    }
    let cmf = (cminfo << 4) as u8 + cm as u8;

    let fcheck = deflate_read_bits!(header_bitstream.read_number_lsb(5), "expected ZLIB fcheck");
    let fdict = deflate_read_bits!(header_bitstream.read_bit(), "expected ZLIB FDICT");
    let flevel = deflate_read_bits!(header_bitstream.read_number_lsb(2), "expected ZLIB FLEVEL");
    let flg = (flevel << 6) + ((fdict as u16) << 5) + fcheck;

    if (((cmf as u16) << 8) + flg) % 31 != 0 {
        return Err(DeflateDecodeError(format!(
            "ZLIB CMF + FLG is not a multiple of 31"
        )));
    }

    let data_start_index = 2 + if fdict == 1 { 4 } else { 0 };

    Ok(data_start_index)
}
