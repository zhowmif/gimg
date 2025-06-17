mod bitstream;
mod consts;
pub mod huffman;
pub mod lzss;
pub mod new_bitsream;
pub mod zlib;

use std::collections::HashMap;

use consts::{LZSS_WINDOW_SIZE, MAX_UNCOMPRESSED_BLOCK_SIZE};
use lzss::{
    backreference::{
        DISTANCE_TO_CODE, DISTANCE_TO_EXTRA_BITS, LENGTH_TO_CODE, LENGTH_TO_EXTRA_BITS,
    },
    encode_lzss,
};
use new_bitsream::NewBitStream;
use zlib::zlib_encode;

pub fn compress_scanlines(scanlines: &Vec<Vec<u8>>) -> Vec<u8> {
    let mut encoder = DeflateEncoder::new(BlockType::None);

    for scanline in scanlines {
        encoder.write_bytes(&scanline);
    }

    let mut zlib_encoded = zlib_encode(encoder);
    let compressed = zlib_encoded.flush_to_bytes();

    compressed
}

// pub fn compress_scanlines(scanlines: &Vec<Vec<u8>>) -> Vec<u8> {
//     let mut e = ZlibEncoder::new(Vec::new(), Compression::none());
//
//     for scanline in scanlines {
//         e.write_all(&scanline).expect("Deflate writing failed");
//     }
//
//     let compressed = e.finish().unwrap();
//
//     compressed
// }

pub enum BlockType {
    None,
    FixedHuffman,
    DynamicHuffman,
}

impl BlockType {
    fn to_number(&self) -> u8 {
        match self {
            BlockType::None => 0,
            BlockType::FixedHuffman => 1,
            BlockType::DynamicHuffman => 2,
        }
    }
}

pub struct DeflateEncoder {
    block_type: BlockType,
    bytes: Vec<u8>,
}

impl DeflateEncoder {
    pub fn new(block_type: BlockType) -> Self {
        if matches!(block_type, BlockType::DynamicHuffman) {
            println!("Dynamic huffman trees are not supported yet :)");
        }

        Self {
            block_type,
            bytes: vec![],
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    pub fn uncompreseed(&self) -> &[u8] {
        &self.bytes
    }

    pub fn finish(&mut self) -> NewBitStream {
        match self.block_type {
            BlockType::None => self.encode_block_type_zero(),
            BlockType::FixedHuffman => self.encode_block_type_one(true),
            BlockType::DynamicHuffman => todo!(),
        }
    }

    fn encode_block_type_zero(&mut self) -> NewBitStream {
        let mut bitstream = NewBitStream::new();

        for (block_index, block_bytes) in self
            .bytes
            .chunks(MAX_UNCOMPRESSED_BLOCK_SIZE.into())
            .enumerate()
        {
            let is_last = block_index == self.bytes.len() / MAX_UNCOMPRESSED_BLOCK_SIZE as usize;
            Self::push_is_last(&mut bitstream, is_last);

            bitstream.push_u8_lsb(BlockType::None.to_number().into(), 2);
            //padding
            bitstream.push_u8_lsb(0, 5);

            let len = block_bytes.len() as u16;
            bitstream.push_u16_lsb(len);
            bitstream.push_u16_lsb(!len);

            for byte in block_bytes {
                bitstream.push_byte_lsb(*byte);
            }
        }

        bitstream
    }

    fn encode_block_type_one(&mut self, is_last: bool) -> NewBitStream {
        let mut result = NewBitStream::new();
        Self::push_is_last(&mut result, is_last);
        result.push_u8_lsb(BlockType::FixedHuffman.to_number().into(), 2);

        let mut lzss = encode_lzss(&self.bytes, LZSS_WINDOW_SIZE);
        lzss.push(lzss::LzssSymbol::EndOfBlock);
        let literal_length_table = self.generate_static_lit_len_table();
        let distance_table = self.generate_static_distance_table();

        for lzss_symbol in lzss {
            match lzss_symbol {
                lzss::LzssSymbol::Literal(lit) => {
                    result.extend(literal_length_table.get(&lit.into()).unwrap())
                }
                lzss::LzssSymbol::Backreference(dist, len) => {
                    let length_code = LENGTH_TO_CODE[len as usize];
                    let encoded_length_code = literal_length_table.get(&length_code).unwrap();
                    result.extend(encoded_length_code);

                    let (len_extra_bits, len_num_extra_bits) = LENGTH_TO_EXTRA_BITS[len as usize];
                    result.push_u16_msb_le(len_extra_bits, len_num_extra_bits);

                    let distance_code = DISTANCE_TO_CODE[dist as usize];
                    let encoded_distance_code = distance_table.get(&distance_code).unwrap();
                    result.extend(&encoded_distance_code);

                    let (dist_extra_bits, dist_num_extra_bits) =
                        DISTANCE_TO_EXTRA_BITS[dist as usize];
                    result.push_u16_msb_le(dist_extra_bits, dist_num_extra_bits);
                }
                lzss::LzssSymbol::EndOfBlock => {
                    result.extend(literal_length_table.get(&256).unwrap())
                }
            }
        }

        result
    }

    fn generate_static_lit_len_table(&self) -> HashMap<u16, NewBitStream> {
        let lengths = Self::generate_bitstream_from_range(48, 191, 8)
            .into_iter()
            .chain(Self::generate_bitstream_from_range(400, 511, 9))
            .chain(Self::generate_bitstream_from_range(0, 23, 7))
            .chain(Self::generate_bitstream_from_range(192, 199, 8))
            .enumerate()
            .map(|(i, val)| (i as u16, val))
            .collect();

        lengths
    }

    fn generate_static_distance_table(&self) -> HashMap<u16, NewBitStream> {
        (0..30)
            .zip(0..30)
            .map(|(i, val)| (i as u16, NewBitStream::from_u32_lsb(val, 5)))
            .collect()
    }

    fn generate_bitstream_from_range(start: usize, end: usize, len: u8) -> Vec<NewBitStream> {
        (start..=end)
            .map(|n| NewBitStream::from_u32_lsb(n as u32, len))
            .collect()
    }

    fn push_is_last(bitstream: &mut NewBitStream, is_last: bool) {
        if is_last {
            bitstream.push_one();
        } else {
            bitstream.push_zero();
        }
    }
}
