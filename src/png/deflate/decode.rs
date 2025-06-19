use std::collections::HashMap;

use crate::png::deflate::{
    huffman::construct_canonical_tree_from_lengths,
    lzss::backreference::{
        DISTANCE_CODE_TO_BASE_DISTANCE, DISTANCE_CODE_TO_EXTRA_BITS, LENGTH_CODE_TO_BASE_LENGTH,
        LENGTH_CODE_TO_EXTRA_BITS, LENGTH_TO_EXTRA_BITS,
    },
};

use super::{
    consts::{CL_ALPHABET, END_OF_BLOCK_MARKER_VALUE},
    lzss::{decode_lzss, LzssSymbol},
    new_bitsream::{BitStreamReader, NewBitStream},
    prefix_table::{
        generate_static_distance_table, generate_static_lit_len_table, reverse_hashmap,
    },
    BlockType,
};

pub fn decode_deflate(bytes: &[u8]) -> Vec<u8> {
    let mut bitsream = BitStreamReader::new(bytes);
    let mut result = Vec::new();

    loop {
        let is_last = bitsream.read_bit_boolean();
        let btype = BlockType::from_number(bitsream.read_number_lsb(2) as u8);

        match btype {
            BlockType::None => parse_block_type_zero(&mut bitsream, &mut result),
            BlockType::FixedHuffman => parse_block_type_one(&mut bitsream, &mut result),
            BlockType::DynamicHuffman => parse_block_type_two(&mut bitsream, &mut result),
        }

        if is_last {
            break;
        }
    }

    result
}

fn parse_block_type_zero(reader: &mut BitStreamReader, target: &mut Vec<u8>) {
    reader.align_to_next_byte();
    let len = reader.read_u16_lsb_le();
    let _nlen = reader.read_u16_lsb_le();
    let bytes = reader.read_bytes_aligned(len as usize);

    target.extend_from_slice(bytes);
}

fn parse_block_type_one(reader: &mut BitStreamReader, target: &mut Vec<u8>) {
    let mut lzss_stream: Vec<LzssSymbol> = Vec::new();
    let literal_length_table = reverse_hashmap(generate_static_lit_len_table());
    let distance_table = reverse_hashmap(generate_static_distance_table());

    let mut current_length = 0;
    let mut read_distance = false;
    let mut current_code = 0;
    let mut current_code_length = 0;
    loop {
        current_code <<= 1;
        current_code_length += 1;
        current_code = match reader.read_bit() {
            0 => current_code,
            _ => current_code | 1,
        };
        let code = NewBitStream::from_u32_lsb(current_code, current_code_length);

        if read_distance {
            if let Some(distance_code) = distance_table.get(&code) {
                current_code = 0;
                current_code_length = 0;
                let base_distance = DISTANCE_CODE_TO_BASE_DISTANCE[*distance_code as usize];
                let num_extra_bits = DISTANCE_CODE_TO_EXTRA_BITS[*distance_code as usize];
                let extra_bits = reader.read_number_lsb(num_extra_bits);
                lzss_stream.push(LzssSymbol::Backreference(
                    base_distance + extra_bits,
                    current_length,
                ));
                read_distance = false;
            }
        } else {
            if let Some(value) = literal_length_table.get(&code) {
                current_code = 0;
                current_code_length = 0;
                if *value < END_OF_BLOCK_MARKER_VALUE {
                    lzss_stream.push(LzssSymbol::Literal(*value as u8));
                } else if *value == END_OF_BLOCK_MARKER_VALUE {
                    // lzss_stream.push(LzssSymbol::EndOfBlock);
                    break;
                } else {
                    let base_length = LENGTH_CODE_TO_BASE_LENGTH[*value as usize];
                    let num_extra_bits = LENGTH_CODE_TO_EXTRA_BITS[*value as usize];
                    let extra_bits = reader.read_number_lsb(num_extra_bits);
                    read_distance = true;
                    current_length = base_length + extra_bits;
                }
            }
        }
    }

    let data = decode_lzss(&lzss_stream);
    target.extend_from_slice(&data);
}

fn parse_block_type_two(reader: &mut BitStreamReader, target: &mut Vec<u8>) {
    let hlit = reader.read_number_lsb(5);
    let ll_table_length = hlit + 257;

    let hdist = reader.read_number_lsb(5);
    let distance_table_length = hdist + 1;
    let hclen = reader.read_number_lsb(4);
    let cl_table_length = hclen + 4;

    let mut cl_codes_lengths: HashMap<u32, u32> = HashMap::new();
    for i in 0..cl_table_length {
        let current_cl_length = reader.read_number_lsb(3);

        cl_codes_lengths.insert(CL_ALPHABET[i as usize], current_cl_length as u32);
    }

    let cl_codes = construct_canonical_tree_from_lengths(&cl_codes_lengths);

    print!("Decode ");
    for (cl_code, code) in cl_codes.iter() {
        print!("({cl_code},{code}), ");
    }
    println!();
}
