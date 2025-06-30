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
use decode::DeflateDecodeError;
use huffman::{construct_canonical_tree_from_lengths, package_merge::PackageMergeEncoder};
use lzss::{
    backreference::{DISTANCE_TO_CODE, LENGTH_TO_CODE},
    encode_lzss, encode_lzss_to_bitstream, LzssSymbol,
};
use prefix_table::{
    generate_static_distance_table, generate_static_lit_len_table, get_cl_codes_for_code_lengths,
    number_of_zero_symbols_at_end,
};
use zlib::{decode_zlib, ZlibEncoder};

use crate::png_assert;

use super::{CompressionLevel, PngParseError};

pub fn compress_scanlines(
    scanlines: &Vec<Vec<u8>>,
    compression_level: CompressionLevel,
) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(compression_level);

    for scanline in scanlines {
        encoder.write_bytes(&scanline);
    }

    encoder.flush()
}

pub fn uncompress_scanlines<'a>(
    data: &'a [u8],
    height: usize,
    width: usize,
    bits_per_pixel: usize,
) -> Result<Vec<Vec<u8>>, PngParseError> {
    let uncompressed_data =
        decode_zlib(data).map_err(|deflate_err| PngParseError(deflate_err.0))?;

    let filter_byte_size = 1;
    let bytes_per_scanline = filter_byte_size + ((width * bits_per_pixel) >> 3);
    let expected_data_size = height * bytes_per_scanline;
    png_assert!(
        uncompressed_data.len() == expected_data_size,
        format!(
            "Expected {} bytes for resolution {}x{} after decompressing, but received {}",
            expected_data_size,
            height,
            width,
            uncompressed_data.len()
        )
    );

    Ok(uncompressed_data
        .chunks(bytes_per_scanline)
        .map(|scanline| scanline.to_vec())
        .collect())
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

    fn from_number(n: u8) -> Result<Self, DeflateDecodeError> {
        Ok(match n {
            0 => DeflateBlockType::None,
            1 => DeflateBlockType::FixedHuffman,
            2 => DeflateBlockType::DynamicHuffman,
            n => {
                return Err(DeflateDecodeError(format!(
                    "Unrecognized deflate block type - {}",
                    n
                )))
            }
        })
    }
}

pub struct DeflateEncoder {
    compression_level: CompressionLevel,
    bytes: Vec<u8>,
}

impl DeflateEncoder {
    pub fn new(compression_level: CompressionLevel) -> Self {
        Self {
            bytes: vec![],
            compression_level,
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    pub fn finish(&mut self) -> WriteBitStream {
        match self.compression_level {
            CompressionLevel::None => encode_block_type_zero(&self.bytes, 0, true).bitstream,
            CompressionLevel::Best => {
                let mut compressed = WriteBitStream::new();
                let mut last_block = EncodedBlock {
                    start_index: 0,
                    block_type: DeflateBlockType::None,
                    bitstream: WriteBitStream::new(),
                };
                let chunk_size = self.bytes.len() / 100;

                for (chunk_num, chunk) in self.bytes.chunks(chunk_size).enumerate() {
                    let chunk_start_index = chunk_num * chunk_size;
                    let is_last_chunk = chunk_num == self.bytes.len() / chunk_size;
                    let chunk_end_index = chunk_start_index + chunk.len();
                    let previous_and_current_data = &self.bytes[..chunk_end_index];
                    let chunk_encoded_alone = smaller_block(
                        encode_block_type_two(
                            &previous_and_current_data,
                            chunk_start_index,
                            is_last_chunk,
                        ),
                        encode_block_type_one(
                            &previous_and_current_data,
                            chunk_start_index,
                            is_last_chunk,
                        ),
                    );
                    let added_to_last_block = match last_block.block_type {
                        DeflateBlockType::None => encode_block_type_zero(
                            &previous_and_current_data,
                            last_block.start_index,
                            is_last_chunk,
                        ),
                        DeflateBlockType::FixedHuffman => encode_block_type_one(
                            &previous_and_current_data,
                            last_block.start_index,
                            is_last_chunk,
                        ),
                        DeflateBlockType::DynamicHuffman => encode_block_type_two(
                            &previous_and_current_data,
                            last_block.start_index,
                            is_last_chunk,
                        ),
                    };

                    if added_to_last_block.bitstream.len() - last_block.bitstream.len()
                        > chunk_encoded_alone.bitstream.len()
                    {
                        compressed.extend(&last_block.bitstream);
                        last_block = chunk_encoded_alone;
                        last_block.start_index = chunk_start_index;
                    } else {
                        last_block = added_to_last_block;
                    }
                }

                compressed.extend(&last_block.bitstream);

                compressed
            }
            CompressionLevel::Fast => encode_block_type_two(&self.bytes, 0, true).bitstream,
        }
    }
}

fn encode_block_type_zero(bytes: &[u8], start_index: usize, is_last: bool) -> EncodedBlock {
    let mut bitstream = WriteBitStream::new();

    for (block_index, block_bytes) in bytes.chunks(MAX_UNCOMPRESSED_BLOCK_SIZE.into()).enumerate() {
        let is_last = is_last && block_index == bytes.len() / MAX_UNCOMPRESSED_BLOCK_SIZE as usize;
        push_is_last(&mut bitstream, is_last);

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

    EncodedBlock {
        block_type: DeflateBlockType::None,
        bitstream,
        start_index,
    }
}

fn encode_block_type_one(bytes: &[u8], start_index: usize, is_last: bool) -> EncodedBlock {
    let mut result = WriteBitStream::new();
    push_is_last(&mut result, is_last);
    result.push_u8_rtl(DeflateBlockType::FixedHuffman.to_number().into(), 2);

    let mut lzss = encode_lzss(bytes, start_index, LZSS_WINDOW_SIZE);
    lzss.push(lzss::LzssSymbol::EndOfBlock);

    let literal_length_table = generate_static_lit_len_table();
    let distance_table = generate_static_distance_table();

    encode_lzss_to_bitstream(&lzss, &literal_length_table, &distance_table, &mut result);

    EncodedBlock {
        block_type: DeflateBlockType::FixedHuffman,
        bitstream: result,
        start_index,
    }
}

fn encode_block_type_two(bytes: &[u8], start_index: usize, is_last: bool) -> EncodedBlock {
    let mut result = WriteBitStream::new();
    push_is_last(&mut result, is_last);
    result.push_u8_rtl(DeflateBlockType::DynamicHuffman.to_number().into(), 2);

    let mut lzss = encode_lzss(bytes, start_index, LZSS_WINDOW_SIZE);
    lzss.push(lzss::LzssSymbol::EndOfBlock);
    let (ll_code_lengths, distance_code_lengths) = generate_prefix_codes_from_lzss_stream(&lzss);
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

    encode_lzss_to_bitstream(&lzss, &ll_codes, &distance_codes, &mut result);

    EncodedBlock {
        block_type: DeflateBlockType::DynamicHuffman,
        bitstream: result,
        start_index,
    }
}

struct EncodedBlock {
    start_index: usize,
    block_type: DeflateBlockType,
    bitstream: WriteBitStream,
}

fn smaller_block(block1: EncodedBlock, block2: EncodedBlock) -> EncodedBlock {
    if block1.bitstream.len() < block2.bitstream.len() {
        block1
    } else {
        block2
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
