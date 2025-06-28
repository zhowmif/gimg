use std::collections::HashMap;

use crate::png::deflate::{
    huffman::construct_canonical_tree_from_lengths,
    lzss::backreference::{
        DISTANCE_CODE_TO_BASE_DISTANCE, DISTANCE_CODE_TO_EXTRA_BITS, LENGTH_CODE_TO_BASE_LENGTH,
        LENGTH_CODE_TO_EXTRA_BITS,
    },
    prefix_table::CLCode,
};

use super::{
    bitsream::{ReadBitStream, WriteBitStream},
    consts::{CL_ALPHABET, END_OF_BLOCK_MARKER_VALUE},
    lzss::{decode_lzss, LzssSymbol},
    prefix_table::{
        generate_static_distance_table, generate_static_lit_len_table, reverse_hashmap,
    },
    DeflateBlockType,
};

#[derive(Debug)]
pub struct DeflateDecodeError(pub String);

#[macro_export]
macro_rules! deflate_read_bits {
    ($read_value:expr, $msg:expr) => {
        match $read_value {
            Some(value) => value,
            None => {
                return Err(DeflateDecodeError(format!(
                    "DEFLATE bitstream ended unexpectedly: {}",
                    $msg
                )));
            }
        }
    };
}

pub fn decode_deflate(bytes: &[u8]) -> Result<Vec<u8>, DeflateDecodeError> {
    let mut bitsream = ReadBitStream::new(bytes);
    let mut result = Vec::new();

    loop {
        let is_last = deflate_read_bits!(bitsream.read_bit_boolean(), "expected new block");
        let btype = deflate_read_bits!(bitsream.read_number_lsb(2), "expected btype") as u8;

        match DeflateBlockType::from_number(btype)? {
            DeflateBlockType::None => parse_block_type_zero(&mut bitsream, &mut result)?,
            DeflateBlockType::FixedHuffman => parse_block_type_one(&mut bitsream, &mut result)?,
            DeflateBlockType::DynamicHuffman => parse_block_type_two(&mut bitsream, &mut result)?,
        }

        if is_last {
            break;
        }
    }

    Ok(result)
}

fn parse_block_type_zero(
    reader: &mut ReadBitStream,
    target: &mut Vec<u8>,
) -> Result<(), DeflateDecodeError> {
    reader.align_to_next_byte();
    let len = deflate_read_bits!(reader.read_u16_lsb_le(), "expected block type 0 LEN");
    let _nlen = deflate_read_bits!(reader.read_u16_lsb_le(), "expected block type 0 NLEN");
    let bytes = deflate_read_bits!(
        reader.read_bytes_aligned(len as usize),
        format!(
            "block with type 0 was too short, tried to read specified length: {}",
            len
        )
    );

    target.extend_from_slice(bytes);

    Ok(())
}

fn parse_block_type_one(
    reader: &mut ReadBitStream,
    target: &mut Vec<u8>,
) -> Result<(), DeflateDecodeError> {
    let literal_length_table = reverse_hashmap(generate_static_lit_len_table());
    let distance_table = reverse_hashmap(generate_static_distance_table());

    decode_compressed_block(reader, target, &literal_length_table, &distance_table)
}

fn parse_block_type_two(
    reader: &mut ReadBitStream,
    target: &mut Vec<u8>,
) -> Result<(), DeflateDecodeError> {
    let hlit = deflate_read_bits!(reader.read_number_lsb(5), "expected HLIT");
    let ll_table_length = hlit + 257;

    let hdist = deflate_read_bits!(reader.read_number_lsb(5), "expected HDIST");
    let distance_table_length = hdist + 1;
    let hclen = deflate_read_bits!(reader.read_number_lsb(4), "expected HLEN");
    let cl_table_length = hclen + 4;

    let mut cl_codes_lengths: HashMap<u32, u32> = HashMap::new();
    for i in 0..cl_table_length {
        let current_cl_length = deflate_read_bits!(reader.read_number_lsb(3), "expected CL code");

        if current_cl_length != 0 {
            cl_codes_lengths.insert(CL_ALPHABET[i as usize], current_cl_length as u32);
        }
    }

    let cl_codes = reverse_hashmap(construct_canonical_tree_from_lengths(&cl_codes_lengths));

    let mut ll_and_distance_lengths = Vec::new();
    let mut current_code = 0;
    let mut current_code_length = 0;
    while (ll_and_distance_lengths.len() as u16) < ll_table_length + distance_table_length {
        current_code <<= 1;
        current_code_length += 1;
        current_code =
            match deflate_read_bits!(reader.read_bit(), "ll/distance CL codes ended abruptly") {
                0 => current_code,
                _ => current_code | 1,
            };
        let code = WriteBitStream::from_u32_ltr(current_code, current_code_length);

        if let Some(cl_code) = cl_codes.get(&code) {
            let cl_code = CLCode::parse_from_bitstream(*cl_code, reader)?;
            ll_and_distance_lengths
                .extend_from_slice(&cl_code.expand(*ll_and_distance_lengths.last().unwrap_or(&0)));
            current_code = 0;
            current_code_length = 0;
        }
    }

    let distance_code_lengths = ll_and_distance_lengths.split_off(ll_table_length as usize);
    let ll_code_lengths = ll_and_distance_lengths;

    let literal_length_table = get_code_table_from_lengths(ll_code_lengths);
    let distance_table = get_code_table_from_lengths(distance_code_lengths);

    decode_compressed_block(reader, target, &literal_length_table, &distance_table)
}

pub fn decode_compressed_block(
    reader: &mut ReadBitStream,
    target: &mut Vec<u8>,
    literal_length_table: &HashMap<WriteBitStream, u16>,
    distance_table: &HashMap<WriteBitStream, u16>,
) -> Result<(), DeflateDecodeError> {
    let mut lzss_stream: Vec<LzssSymbol> = Vec::new();
    let mut current_length = 0;
    let mut read_distance = false;
    let mut code = WriteBitStream::new();
    loop {
        match deflate_read_bits!(reader.read_bit(), "data ended before end of block marker") {
            0 => code.push_zero(),
            _ => code.push_one(),
        };

        if read_distance {
            if let Some(distance_code) = distance_table.get(&code) {
                code.reset();
                let base_distance = DISTANCE_CODE_TO_BASE_DISTANCE[*distance_code as usize];
                let num_extra_bits = DISTANCE_CODE_TO_EXTRA_BITS[*distance_code as usize];
                let extra_bits = deflate_read_bits!(
                    reader.read_number_lsb(num_extra_bits),
                    "data ended before end of block marker"
                );
                lzss_stream.push(LzssSymbol::Backreference(
                    base_distance + extra_bits,
                    current_length,
                ));
                read_distance = false;
            }
        } else {
            if let Some(value) = literal_length_table.get(&code) {
                if *value < END_OF_BLOCK_MARKER_VALUE {
                    lzss_stream.push(LzssSymbol::Literal(*value as u8));
                } else if *value == END_OF_BLOCK_MARKER_VALUE {
                    break;
                } else {
                    let base_length = LENGTH_CODE_TO_BASE_LENGTH[*value as usize];
                    let num_extra_bits = LENGTH_CODE_TO_EXTRA_BITS[*value as usize];
                    let extra_bits = deflate_read_bits!(
                        reader.read_number_lsb(num_extra_bits),
                        "data ended before end of block marker"
                    );
                    read_distance = true;
                    current_length = base_length + extra_bits;
                }
                code.reset();
            }
        }
    }

    decode_lzss(target, &lzss_stream)?;

    Ok(())
}

fn get_code_table_from_lengths(table_lengths: Vec<u32>) -> HashMap<WriteBitStream, u16> {
    let frequency_map: HashMap<u16, u32> = table_lengths
        .into_iter()
        .enumerate()
        .filter(|(_i, l)| *l != 0)
        .map(|(i, l)| (i as u16, l))
        .collect();

    reverse_hashmap(construct_canonical_tree_from_lengths(&frequency_map))
}
