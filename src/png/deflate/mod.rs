pub mod bitsream;
mod consts;
pub mod decode;
pub mod huffman;
pub mod lzss;
pub mod prefix_table;
pub mod zlib;

use std::collections::HashMap;

use bitsream::WriteBitStream;
use consts::{
    CL_ALPHABET, END_OF_BLOCK_MARKER_VALUE, LZSS_WINDOW_SIZE, MAX_CL_CODE_LENGTH,
    MAX_SYMBOL_CODE_LENGTH, MAX_UNCOMPRESSED_BLOCK_SIZE,
};
use huffman::{construct_canonical_tree_from_lengths, package_merge::PackageMergeEncoder};
use lzss::{
    backreference::{
        DISTANCE_TO_CODE, DISTANCE_TO_EXTRA_BITS, LENGTH_TO_CODE, LENGTH_TO_EXTRA_BITS,
    },
    encode_lzss, LzssSymbol,
};
use prefix_table::{
    generate_static_distance_table, generate_static_lit_len_table, get_cl_codes_for_code_lengths,
    number_of_zero_symbols_at_end,
};
use zlib::ZlibEncoder;

pub fn compress_scanlines(scanlines: &Vec<Vec<u8>>) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new();

    for scanline in scanlines {
        encoder.write_bytes(&scanline);
    }

    encoder.flush()
}

#[derive(Debug)]
pub enum DeflateBlockType {
    None,
    FixedHuffman,
    DynamicHuffman,
}

impl DeflateBlockType {
    fn to_number(&self) -> u8 {
        match self {
            DeflateBlockType::None => 0,
            DeflateBlockType::FixedHuffman => 1,
            DeflateBlockType::DynamicHuffman => 2,
        }
    }

    fn from_number(n: u8) -> Self {
        match n {
            0 => DeflateBlockType::None,
            1 => DeflateBlockType::FixedHuffman,
            2 => DeflateBlockType::DynamicHuffman,
            n => panic!("Unrecognized deflate block type {}", n),
        }
    }
}

pub struct DeflateEncoder {
    block_type: DeflateBlockType,
    bytes: Vec<u8>,
}

impl DeflateEncoder {
    pub fn new(block_type: DeflateBlockType) -> Self {
        Self {
            block_type,
            bytes: vec![],
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    pub fn finish(&mut self) -> WriteBitStream {
        match self.block_type {
            DeflateBlockType::None => self.encode_block_type_zero(),
            DeflateBlockType::FixedHuffman => self.encode_block_type_one(true),
            DeflateBlockType::DynamicHuffman => self.encode_block_type_two(true),
        }
    }

    fn encode_block_type_zero(&mut self) -> WriteBitStream {
        let mut bitstream = WriteBitStream::new();

        for (block_index, block_bytes) in self
            .bytes
            .chunks(MAX_UNCOMPRESSED_BLOCK_SIZE.into())
            .enumerate()
        {
            let is_last = block_index == self.bytes.len() / MAX_UNCOMPRESSED_BLOCK_SIZE as usize;
            Self::push_is_last(&mut bitstream, is_last);

            bitstream.push_u8_rtl(DeflateBlockType::None.to_number().into(), 2);
            //padding
            bitstream.push_u8_rtl(0, 5);

            let len = block_bytes.len() as u16;
            bitstream.push_u16_ltr_le(len);
            bitstream.push_u16_ltr_le(!len);

            for byte in block_bytes {
                bitstream.push_byte_ltr(*byte);
            }
        }

        bitstream
    }

    fn encode_block_type_one(&mut self, is_last: bool) -> WriteBitStream {
        let mut result = WriteBitStream::new();
        Self::push_is_last(&mut result, is_last);
        result.push_u8_rtl(DeflateBlockType::FixedHuffman.to_number().into(), 2);

        let mut lzss = encode_lzss(&self.bytes, LZSS_WINDOW_SIZE);
        lzss.push(lzss::LzssSymbol::EndOfBlock);

        let literal_length_table = generate_static_lit_len_table();
        let distance_table = generate_static_distance_table();

        Self::encode_lzss_stream(&lzss, &literal_length_table, &distance_table, &mut result);

        result
    }

    fn encode_block_type_two(&mut self, is_last: bool) -> WriteBitStream {
        let mut result = WriteBitStream::new();
        Self::push_is_last(&mut result, is_last);
        result.push_u8_rtl(DeflateBlockType::DynamicHuffman.to_number().into(), 2);

        let mut lzss = encode_lzss(&self.bytes, LZSS_WINDOW_SIZE);
        lzss.push(lzss::LzssSymbol::EndOfBlock);
        let (ll_code_lengths, distance_code_lengths) =
            Self::generate_prefix_codes_from_lzss_stream(&lzss);
        let ll_codes = construct_canonical_tree_from_lengths(&ll_code_lengths);
        let distance_codes = construct_canonical_tree_from_lengths(&distance_code_lengths);

        let ll_alphabet: Vec<_> = (0..=285).collect();
        let ll_table_length =
            ll_alphabet.len() - number_of_zero_symbols_at_end(&ll_alphabet, &ll_code_lengths);
        let ll_table_cl_codes =
            get_cl_codes_for_code_lengths(&ll_alphabet[..ll_table_length], &ll_code_lengths);
        let hlit = ll_table_length - 257;
        result.push_u8_rtl(hlit as u8, 5);

        let distance_alphabet: Vec<_> = (0..=31).collect();
        let distance_table_length = (distance_alphabet.len()
            - number_of_zero_symbols_at_end(&distance_alphabet, &distance_code_lengths))
        .max(1);
        let distance_table_cl_codes = get_cl_codes_for_code_lengths(
            &distance_alphabet[..distance_table_length],
            &distance_code_lengths,
        );
        let hdist = distance_table_length - 1;
        result.push_u8_rtl(hdist as u8, 5);
        let mut cl_codes_encoder = PackageMergeEncoder::new();
        for cl_code in ll_table_cl_codes
            .iter()
            .chain(distance_table_cl_codes.iter())
        {
            cl_codes_encoder.add_symbol(&cl_code.to_number());
        }
        let cl_codes_lengths = cl_codes_encoder.get_symbol_lengths(MAX_CL_CODE_LENGTH);
        let cl_codes = construct_canonical_tree_from_lengths(&cl_codes_lengths);
        let cl_table_length =
            CL_ALPHABET.len() - number_of_zero_symbols_at_end(&CL_ALPHABET, &cl_codes_lengths);
        let hclen = cl_table_length - 4;
        result.push_u8_rtl(hclen as u8, 4);

        for i in 0..cl_table_length {
            let cl_code_length = cl_codes_lengths
                .get(&CL_ALPHABET[i])
                .map(|x| *x)
                .unwrap_or(0);

            result.push_u8_rtl(cl_code_length as u8, 3);
        }
        for cl_code in ll_table_cl_codes {
            cl_code.encode(&cl_codes, &mut result);
        }
        for cl_code in distance_table_cl_codes {
            cl_code.encode(&cl_codes, &mut result);
        }

        Self::encode_lzss_stream(&lzss, &ll_codes, &distance_codes, &mut result);

        result
    }

    pub fn encode_lzss_stream(
        lzss_stream: &[LzssSymbol],
        ll_table: &HashMap<u16, WriteBitStream>,
        distance_table: &HashMap<u16, WriteBitStream>,
        target: &mut WriteBitStream,
    ) {
        for lzss_symbol in lzss_stream {
            match lzss_symbol {
                lzss::LzssSymbol::Literal(lit) => {
                    target.extend(ll_table.get(&(*lit as u16)).unwrap());
                }
                lzss::LzssSymbol::Backreference(dist, len) => {
                    let length_code = LENGTH_TO_CODE[*len as usize];
                    let encoded_length_code = ll_table.get(&length_code).unwrap();
                    target.extend(encoded_length_code);

                    let (len_extra_bits, len_num_extra_bits) = LENGTH_TO_EXTRA_BITS[*len as usize];
                    target.push_u16_rtl(len_extra_bits, len_num_extra_bits);

                    let distance_code = DISTANCE_TO_CODE[*dist as usize];
                    let encoded_distance_code = distance_table.get(&distance_code).unwrap();
                    target.extend(&encoded_distance_code);

                    let (dist_extra_bits, dist_num_extra_bits) =
                        DISTANCE_TO_EXTRA_BITS[*dist as usize];
                    target.push_u16_rtl(dist_extra_bits, dist_num_extra_bits);
                }
                lzss::LzssSymbol::EndOfBlock => {
                    target.extend(ll_table.get(&END_OF_BLOCK_MARKER_VALUE).unwrap())
                }
            }
        }
    }

    fn generate_prefix_codes_from_lzss_stream(
        lzss_stream: &[LzssSymbol],
    ) -> (HashMap<u16, u32>, HashMap<u16, u32>) {
        let mut ll_encoder = PackageMergeEncoder::new();
        let mut distance_encoder = PackageMergeEncoder::new();

        for lzss_symbol in lzss_stream {
            match lzss_symbol {
                LzssSymbol::Literal(value) => {
                    ll_encoder.add_symbol(&(*value as u16));
                }
                LzssSymbol::Backreference(distance, length) => {
                    ll_encoder.add_symbol(&LENGTH_TO_CODE[*length as usize]);
                    distance_encoder.add_symbol(&DISTANCE_TO_CODE[*distance as usize]);
                }
                LzssSymbol::EndOfBlock => {
                    ll_encoder.add_symbol(&END_OF_BLOCK_MARKER_VALUE);
                }
            }
        }
        let ll_code_lengths = ll_encoder.get_symbol_lengths(MAX_SYMBOL_CODE_LENGTH);
        let distance_code_length = distance_encoder.get_symbol_lengths(MAX_SYMBOL_CODE_LENGTH);

        (ll_code_lengths, distance_code_length)
    }

    fn push_is_last(bitstream: &mut WriteBitStream, is_last: bool) {
        if is_last {
            bitstream.push_one();
        } else {
            bitstream.push_zero();
        }
    }
}
