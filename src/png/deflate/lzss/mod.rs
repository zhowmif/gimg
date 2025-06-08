use hash::LzssHashTable;

use crate::{bits::Bit, png::deflate::bitstream::BitStream};
use std::collections::HashMap;

mod hash;

struct Backreference(u16, u8);

impl Backreference {
    fn distance(&self) -> u16 {
        self.0
    }

    fn length(&self) -> u8 {
        self.1
    }
}

pub fn encode_lzss_table(bytes: &[u8], window_size: usize) -> BitStream {
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

pub fn encode_lzss(bytes: &[u8], window_size: usize) -> BitStream {
    let mut cursor = 0;
    let mut bitstream = BitStream::new();

    while cursor < bytes.len() {
        let reference = find_backrefrence(bytes, cursor, window_size);

        if reference.length() > 3 {
            bitstream.push_one();
            bitstream.push_byte(reference.length());
            bitstream.push_bytes(&reference.distance().to_be_bytes());
            cursor += reference.length() as usize;
        } else {
            bitstream.push_zero();
            bitstream.push_byte(bytes[cursor]);
            cursor += 1;
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
    let best_match = table.search(bytes, cursor, cursor.max(window_size) - window_size);

    if cursor + 2 < bytes.len() {
        let key = (bytes[cursor], bytes[cursor + 1], bytes[cursor + 2]);
        table.insert(key, cursor);
    }

    best_match
}

fn find_backrefrence(bytes: &[u8], cursor: usize, window_size: usize) -> Backreference {
    let window_start = cursor.max(window_size) - window_size;
    let window = &bytes[window_start..];
    let mut longset_sequence = Backreference(0, 0);

    let mut index = 0;
    while window_start + index < cursor {
        let mut current_match_length: usize = 0;

        while current_match_length + 1 < u8::MAX.into()
            && cursor + current_match_length < bytes.len()
            && index + current_match_length < window.len()
            && window[index + current_match_length] == bytes[cursor + current_match_length]
        {
            current_match_length += 1;
        }

        if current_match_length > longset_sequence.length() as usize {
            longset_sequence = Backreference(
                (cursor - (window_start + index)) as u16,
                current_match_length as u8,
            );
        }

        index += 1;
    }

    longset_sequence
}

pub fn decode_lzss(bitstream: &BitStream) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    let mut idx = 0;
    let mut table: HashMap<(u8, u8, u8), u8> = HashMap::new();
    table.insert((7, 5, 3), 34);

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
