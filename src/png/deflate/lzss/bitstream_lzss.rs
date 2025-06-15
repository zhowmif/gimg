//This module is to encode/decode a complete lzss stream, encoding each symbol as 9 bits
//(first bit being 0 for a literal and 1 for a backreference)
use crate::{bits::Bit, png::deflate::bitstream::BitStream};

use super::hash::LzssHashTable;

struct Backreference(u16, u8);

impl Backreference {
    fn distance(&self) -> u16 {
        self.0
    }

    fn length(&self) -> u8 {
        self.1
    }
}

pub fn encode_lzss(bytes: &[u8], window_size: usize) -> BitStream {
    let mut cursor = 0;
    let mut bitstream = BitStream::new();
    let mut table = LzssHashTable::new();

    while cursor < bytes.len() {
        match find_backreference_with_table(bytes, cursor, window_size, &mut table) {
            Some(backreference) => {
                bitstream.push_one();
                bitstream.push_byte(backreference.length());
                bitstream.push_bytes(&backreference.distance().to_be_bytes());
                cursor += backreference.length() as usize;
            }
            None => {
                bitstream.push_zero();
                bitstream.push_byte(bytes[cursor]);
                cursor += 1;
            }
        }
    }

    bitstream
}

fn find_backreference_with_table(
    bytes: &[u8],
    cursor: usize,
    window_size: usize,
    table: &mut LzssHashTable,
) -> Option<Backreference> {
    let best_match = table.search(bytes, cursor, cursor.max(window_size) - window_size)?;

    if cursor + 2 < bytes.len() {
        let key = (bytes[cursor], bytes[cursor + 1], bytes[cursor + 2]);
        table.insert(key, cursor);
    }

    Some(Backreference(best_match.0, best_match.1))
}

pub fn decode_lzss(bitstream: &BitStream) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    let mut idx = 0;

    while idx < bitstream.len() {
        let is_literal = matches!(bitstream.read_bit(&mut idx), Bit::Zero);

        if is_literal {
            let byte = bitstream.read_byte(&mut idx);
            result.push(byte);
        } else {
            let length = bitstream.read_byte(&mut idx) as usize;
            let d1 = bitstream.read_byte(&mut idx) as u16;
            let d2 = bitstream.read_byte(&mut idx) as u16;
            let distance = (d1 << 8) + d2;
            let backrefrence_data_start = result.len() - distance as usize;

            for i in backrefrence_data_start..backrefrence_data_start + length {
                result.push(result[i]);
            }
        }
    }

    result
}
