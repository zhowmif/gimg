use std::{
    collections::HashMap,
    fs,
    process::{self, exit},
};

use crate::{
    is_tree_prefix_free,
    png::deflate::{
        huffman::construct_canonical_tree_from_lengths,
        lzss::backreference::{
            DISTANCE_CODE_TO_BASE_DISTANCE, DISTANCE_CODE_TO_EXTRA_BITS,
            LENGTH_CODE_TO_BASE_LENGTH, LENGTH_CODE_TO_EXTRA_BITS,
        },
        prefix_table::CLCode,
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
    let literal_length_table = reverse_hashmap(generate_static_lit_len_table());
    let distance_table = reverse_hashmap(generate_static_distance_table());

    decode_compressed_block(reader, target, &literal_length_table, &distance_table);
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

        if current_cl_length != 0 {
            cl_codes_lengths.insert(CL_ALPHABET[i as usize], current_cl_length as u32);
        }
    }

    let cl_codes = reverse_hashmap(construct_canonical_tree_from_lengths(&cl_codes_lengths));

    // print!("Decode ");
    // for (cl_code, code) in cl_codes.iter() {
    //     print!("({code},{}), ", *cl_code);
    // }
    // println!();

    let mut ll_and_distance_lengths = Vec::new();
    let mut current_code = 0;
    let mut current_code_length = 0;
    println!("***************************");
    while (ll_and_distance_lengths.len() as u16) < ll_table_length + distance_table_length {
        current_code <<= 1;
        current_code_length += 1;
        current_code = match reader.read_bit() {
            0 => current_code,
            _ => current_code | 1,
        };
        let code = NewBitStream::from_u32_msb(current_code, current_code_length);
        // println!("looking for code {}", code);

        if let Some(cl_code) = cl_codes.get(&code) {
            let cl_code = CLCode::parse_from_bitstream(*cl_code, reader);
            // println!("[DECODE] {:?} - {} - {}", cl_code, code, reader.bit_index);
            ll_and_distance_lengths
                .extend_from_slice(&cl_code.expand(*ll_and_distance_lengths.last().unwrap_or(&0)));
            current_code = 0;
            current_code_length = 0;
        }
    }

    let distance_code_lengths = ll_and_distance_lengths.split_off(ll_table_length as usize);
    let ll_code_lengths = ll_and_distance_lengths;

    // print!("ll code lengths {:?}", ll_code_lengths);
    let literal_length_table = get_code_table_from_lengths(ll_code_lengths);
    println!("Decode ll codes {:?}", literal_length_table.iter());
    // print!("Decode ");
    // is_tree_prefix_free(
    //     &literal_length_table
    //         .clone()
    //         .into_iter()
    //         .map(|(k, v)| (v as u32, k))
    //         .collect(),
    // );
    // process::exit(1);
    // print!("decode ll codes ");
    // for (cl_code, code) in literal_length_table.iter() {
    //     print!("({cl_code},{code}), ");
    // }
    // println!();
    let distance_table = get_code_table_from_lengths(distance_code_lengths);

    decode_compressed_block(reader, target, &literal_length_table, &distance_table);
}

pub fn reverse_bitstream_map(table: HashMap<u16, NewBitStream>) -> HashMap<u16, NewBitStream> {
    table
        .into_iter()
        .map(|(k, mut v)| {
            let mut inverted = NewBitStream::new();
            inverted.extend_reverse(&mut v);

            (k, v)
        })
        .collect()
}

pub fn decode_compressed_block(
    reader: &mut BitStreamReader,
    target: &mut Vec<u8>,
    literal_length_table: &HashMap<NewBitStream, u16>,
    distance_table: &HashMap<NewBitStream, u16>,
) {
    // for (k, v) in literal_length_table.iter() {
    //     if *v == 65 {
    //         println!("{} {}", v, k);
    //     }
    // }
    let mut lzss_stream: Vec<LzssSymbol> = Vec::new();
    let mut current_length = 0;
    let mut read_distance = false;
    let mut code = NewBitStream::new();
    println!("Starting with reader at {}", reader.bit_index);
    loop {
        // if lzss_stream.len() == 1 {
        //     println!("First {:?}", lzss_stream);
        // }
        // if lzss_stream.len() == 14349 {
        //     fs::write("decode.txt", format!("{:?}", lzss_stream));
        // }
        match reader.read_bit() {
            0 => code.push_zero(),
            _ => code.push_one(),
        };
        // println!("Looking for code {}", code);

        if read_distance {
            if let Some(distance_code) = distance_table.get(&code) {
                code.reset();
                let base_distance = DISTANCE_CODE_TO_BASE_DISTANCE[*distance_code as usize];
                let num_extra_bits = DISTANCE_CODE_TO_EXTRA_BITS[*distance_code as usize];
                // println!(
                //     "Dist code {distance_code} Reading {} extra bits",
                //     num_extra_bits
                // );
                let extra_bits = reader.read_number_lsb(num_extra_bits);

                // println!(
                //     "Pushing backreference ({}, {}) (len {})",
                //     base_distance + extra_bits,
                //     current_length,
                //     reader.bit_index
                // );
                //
                lzss_stream.push(LzssSymbol::Backreference(
                    base_distance + extra_bits,
                    current_length,
                ));
                read_distance = false;
            }
        } else {
            if let Some(value) = literal_length_table.get(&code) {
                if *value < END_OF_BLOCK_MARKER_VALUE {
                    // println!("Pushing literal {}", value);
                    // if *value == 85 {
                    //     println!("DEC {value} is {code}");
                    // }
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
                    // println!(
                    //     "length code {}, length {current_length}, code {}, (reader len {})",
                    //     value, code, reader.bit_index
                    // );

                    // if current_length == 28 {
                    //     println!("here {} {code}", lzss_stream.len());
                    // }
                }
                code.reset();
            }
        }
    }

    let data = decode_lzss(&lzss_stream);
    target.extend_from_slice(&data);
}

fn get_code_table_from_lengths(table_lengths: Vec<u32>) -> HashMap<NewBitStream, u16> {
    // println!("Table lengths {:?}", table_lengths);
    let frequency_map: HashMap<u16, u32> = table_lengths
        .into_iter()
        .enumerate()
        .filter(|(_i, l)| *l != 0)
        .map(|(i, l)| (i as u16, l))
        .collect();
    // println!("Frequency map {:?}", frequency_map);

    reverse_hashmap(construct_canonical_tree_from_lengths(&frequency_map))
}
