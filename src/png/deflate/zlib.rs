use crate::{algebra::align_up, png::adler32::Adler32Calculator};

use super::{new_bitsream::NewBitStream, DeflateEncoder};

const DEFLATE_CM: u16 = 8;

pub fn zlib_encode(mut deflate_encoder: DeflateEncoder) -> Vec<u8> {
    let mut adler32 = Adler32Calculator::new();
    adler32.update_adler32(&deflate_encoder.bytes);
    let adler32 = adler32.get_adler32();
    let mut encoded = NewBitStream::new();

    let cm = 8;
    let cminfo = 7;
    let cmf = (cminfo << 4) as u8 + cm as u8;
    let fdict = 0;
    let flevel = 0;
    let flg = (cmf as u32) << 8 + (flevel << 6) + (fdict << 5);
    let fcheck = (align_up(flg as usize, 31) - flg as usize) as u8;

    encoded.push_u8_rtl(cm, 4);
    encoded.push_u8_rtl(cminfo, 4);
    encoded.push_u8_rtl(fcheck, 5);
    encoded.push_u8_rtl(fdict, 1);
    encoded.push_u8_rtl(flevel, 2);
    let mut bytes = encoded.flush_to_bytes();
    bytes.extend_from_slice(&deflate_encoder.finish().flush_to_bytes());
    bytes.extend_from_slice(&adler32.to_be_bytes());

    bytes
}
