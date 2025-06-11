use crate::{algebra::align_up, png::adler32::Adler32Calculator};

use super::{bitstream::BitStream, new_bitsream::NewBitStream, DeflateEncoder};

const DEFLATE_CM: u16 = 8;

pub fn zlib_encode(mut deflate_encoder: DeflateEncoder) -> NewBitStream {
    let mut adler32 = Adler32Calculator::new();
    adler32.update_adler32(&deflate_encoder.bytes);
    let adler32 = adler32.get_adler32();
    let mut encoded = NewBitStream::new();

    let cm = 8;
    let cminfo = 0;
    let cmf = (cminfo << 4) as u8 + cm as u8;
    let fdict = 0;
    let flevel = 0;
    let flg = (cmf as u32) << 8 + (flevel << 6) + (fdict << 5);
    let fcheck = (align_up(flg as usize, 31) - flg as usize) as u8;

    encoded.push_u8_lsb(cm, 4);
    encoded.push_u8_lsb(cminfo, 4);
    encoded.push_u8_lsb(fcheck, 5);
    encoded.push_u8_lsb(fdict, 1);
    encoded.push_u8_lsb(flevel, 2);
    encoded.extend_aligned(&deflate_encoder.finish());
    encoded.push_bytes_lsb(&adler32.to_be_bytes());

    encoded
}
