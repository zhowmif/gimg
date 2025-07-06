pub mod backreference;
mod hash;
// use crate::png::deflate::lzss::hash::LAST_KEY_HITS;
// use crate::png::deflate::lzss::hash::TOTAL_ATTEMPS;
use hash::first_byte_repeat_count;
pub use hash::LzssHashTable;

use std::{collections::HashMap, iter::repeat_n};

use backreference::{
    DISTANCE_CODE_TO_EXTRA_BITS, DISTANCE_TO_CODE, DISTANCE_TO_EXTRA_BITS,
    LENGTH_CODE_TO_EXTRA_BITS, LENGTH_TO_CODE, LENGTH_TO_EXTRA_BITS, LZSS_DISTANCE_CODES,
    LZSS_NUMBER_OF_DISTANCES, LZSS_NUMBER_OF_LENGHTS, LZSS_NUMBER_OF_LENGTH_CODES,
    LZSS_NUMBER_OF_LITERALS,
};

use crate::png::CompressionLevel;

use super::{
    append_end_of_block,
    bitstream::WriteBitStream,
    consts::{END_OF_BLOCK_MARKER_VALUE, LZSS_WINDOW_SIZE},
    decode::DeflateDecodeError,
    generate_prefix_codes_from_lzss_stream,
};

#[derive(Debug, Clone)]
pub enum LzssSymbol {
    Literal(u8),
    Backreference(u16, u16),
    EndOfBlock,
}

pub fn encode_lzss_greedy(bytes: &[u8], compression_level: CompressionLevel) -> Vec<LzssSymbol> {
    let mut table = LzssHashTable::new(compression_level);
    let mut cursor = 0;
    let mut stream = Vec::with_capacity(bytes.len() / 2);

    while cursor < bytes.len() {
        stream.push(
            match find_backreference_with_table(bytes, cursor, LZSS_WINDOW_SIZE, &mut table) {
                Some(backreference) => {
                    let symbol = LzssSymbol::Backreference(backreference.0, backreference.1);
                    cursor += backreference.1 as usize;

                    symbol
                }
                None => {
                    let symbol = LzssSymbol::Literal(bytes[cursor]);
                    cursor += 1;

                    symbol
                }
            },
        );
    }

    stream
}

pub fn encode_lzss_optimized(bytes: &[u8]) -> Vec<LzssSymbol> {
    let mut lzss_symbols = encode_lzss_greedy(bytes, CompressionLevel::Best);
    // print!("Initial ");
    // symbol_stats(&lzss_symbols);
    // println!("initial lzss symbols {:?}", lzss_symbols);
    for _i in 0..1 {
        let (ll_code_lengths, distance_code_lengths) =
            generate_prefix_codes_from_lzss_stream(append_end_of_block(&lzss_symbols));
        // let compressed = encode_block_type_two(&lzss_symbols, 0, true);
        // println!(
        //     "len {} {} - {}",
        //     ll_code_lengths.len(),
        //     distance_code_lengths.len(),
        //     compressed.bitstream.len() / 8
        // );
        let literal_encoding_costs = construct_literal_encoding_costs(&ll_code_lengths);
        let lengths_encoding_costs = construct_length_encoding_costs(&ll_code_lengths);
        let distance_encoding_costs = construct_distance_encoding_costs(&distance_code_lengths);
        lzss_symbols = encode_lzss_iteration(
            bytes,
            literal_encoding_costs,
            lengths_encoding_costs,
            distance_encoding_costs,
        );
        // print!("Round {_i} ");
        // symbol_stats(&lzss_symbols);
    }

    lzss_symbols
}

fn construct_literal_encoding_costs(
    ll_code_lengths: &HashMap<u16, u32>,
) -> [u32; LZSS_NUMBER_OF_LITERALS] {
    let mut literal_encode_costs = [0; LZSS_NUMBER_OF_LITERALS];
    let mut average_literal_size: u32 = 0;
    let mut number_of_literals_in_table = 0;

    for literal in 0..LZSS_NUMBER_OF_LITERALS {
        if let Some(literal_size) = ll_code_lengths.get(&(literal as u16)) {
            average_literal_size += literal_size;
            number_of_literals_in_table += 1;
        }
    }
    average_literal_size /= number_of_literals_in_table;

    for literal in 0..LZSS_NUMBER_OF_LITERALS {
        let literal_encode_cost = ll_code_lengths
            .get(&(literal as u16))
            .unwrap_or(&average_literal_size);

        literal_encode_costs[literal] = *literal_encode_cost;
    }

    literal_encode_costs
}

fn construct_length_encoding_costs(
    ll_code_lengths: &HashMap<u16, u32>,
) -> [u32; LZSS_NUMBER_OF_LENGHTS] {
    let mut average_code_size: u32 = 0;
    let mut number_of_codes_in_table = 0;
    let mut lengths_encoding_costs = [0; LZSS_NUMBER_OF_LENGHTS];

    for code in LZSS_NUMBER_OF_LITERALS..(LZSS_NUMBER_OF_LITERALS + LZSS_NUMBER_OF_LENGTH_CODES) {
        if let Some(code_size) = ll_code_lengths.get(&(code as u16)) {
            average_code_size += code_size;
            number_of_codes_in_table += 1;
        }
    }

    average_code_size /= number_of_codes_in_table;

    for length in 0..LZSS_NUMBER_OF_LENGHTS {
        let code = LENGTH_TO_CODE[length];
        let code_encoding_cost = ll_code_lengths.get(&code).unwrap_or(&average_code_size);
        let extra_bits = LENGTH_CODE_TO_EXTRA_BITS[code as usize];

        let total_encode_cost = *code_encoding_cost + extra_bits as u32;

        lengths_encoding_costs[length] = total_encode_cost;
    }

    lengths_encoding_costs
}

fn construct_distance_encoding_costs(
    distance_code_lengths: &HashMap<u16, u32>,
) -> [u32; LZSS_NUMBER_OF_DISTANCES] {
    let mut average_code_size: u32 = 0;
    let mut number_of_codes_in_table = 0;

    for code in LZSS_DISTANCE_CODES {
        if let Some(code_size) = distance_code_lengths.get(&code) {
            average_code_size += code_size;
            number_of_codes_in_table += 1;
        }
    }

    average_code_size /= number_of_codes_in_table;

    let mut distance_encoding_costs = [0; LZSS_NUMBER_OF_DISTANCES];
    for distance in 0..LZSS_NUMBER_OF_DISTANCES {
        let code = DISTANCE_TO_CODE[distance];
        let code_encoding_cost = distance_code_lengths
            .get(&code)
            .unwrap_or(&average_code_size);
        let extra_bits = DISTANCE_CODE_TO_EXTRA_BITS[code as usize];

        let total_encode_cost = *code_encoding_cost + extra_bits as u32;

        distance_encoding_costs[distance] = total_encode_cost;
    }

    distance_encoding_costs
}

fn table_to_vec(t: &HashMap<u16, u32>, size: usize) -> Vec<Option<u32>> {
    let mut result: Vec<Option<u32>> = repeat_n(None, size).collect();

    for (k, v) in t {
        result[*k as usize] = Some(*v);
    }

    result
}

pub fn symbol_stats(lzss_symbols: &[LzssSymbol]) {
    let mut lits = 0;
    let mut bfs = 0;
    for sym in lzss_symbols {
        match sym {
            LzssSymbol::Literal(_) => lits += 1,
            LzssSymbol::Backreference(_, _) => bfs += 1,
            LzssSymbol::EndOfBlock => {}
        }
    }
    println!("total {} lits {lits}, bfs {bfs}", lzss_symbols.len());
}

pub fn encode_lzss_iteration(
    bytes: &[u8],
    literal_encoding_costs: [u32; LZSS_NUMBER_OF_LITERALS],
    lengths_encoding_costs: [u32; LZSS_NUMBER_OF_LENGHTS],
    distance_encoding_costs: [u32; LZSS_NUMBER_OF_DISTANCES],
) -> Vec<LzssSymbol> {
    //this is in reverse order
    let mut best_symbol_costs: Vec<(u32, LzssSymbol)> = Vec::new();
    let mut lzss_table = LzssHashTable::new(CompressionLevel::Best);
    for i in ((bytes.len().max(LZSS_WINDOW_SIZE) - LZSS_WINDOW_SIZE)..(bytes.len() - 2)).rev() {
        lzss_table.insert(i, bytes, first_byte_repeat_count(&bytes[i..]));
    }

    for cost_list_index in 0..bytes.len() {
        let bytes_index = bytes.len() - cost_list_index - 1;
        let byte = &bytes[bytes_index];

        let literal_cost = literal_encoding_costs[*byte as usize]
            + cost_list_index
                .checked_sub(1)
                .map(|idx| best_symbol_costs[idx].0)
                .unwrap_or(0);

        let backreferences: Vec<(u16, u16)> = lzss_table
            .get_all_backreferences(bytes, bytes_index)
            .unwrap_or_default();

        let mut best_bf = (0, 0);
        let mut best_bf_cost = literal_cost;
        for bf in backreferences {
            let bf_end_cost = cost_list_index
                .checked_sub(bf.1 as usize)
                .map(|idx| best_symbol_costs[idx].0)
                .unwrap_or(0);

            if bf_end_cost > best_bf_cost {
                continue;
            }

            let bf_encode_cost =
                cost_of_encoding_backreference(bf, lengths_encoding_costs, distance_encoding_costs);

            let bf_cost = bf_end_cost + bf_encode_cost;

            if bf_cost < best_bf_cost {
                best_bf = bf;
                best_bf_cost = bf_cost;
            }
        }

        if best_bf_cost < literal_cost {
            let symbol = LzssSymbol::Backreference(best_bf.0, best_bf.1);
            best_symbol_costs.push((best_bf_cost, symbol));
        } else {
            best_symbol_costs.push((literal_cost, LzssSymbol::Literal(*byte)));
        }

        if bytes_index >= LZSS_WINDOW_SIZE {
            let first_byte_index_in_window = bytes_index - LZSS_WINDOW_SIZE;
            lzss_table.insert(
                first_byte_index_in_window,
                bytes,
                first_byte_repeat_count(&bytes[first_byte_index_in_window..]),
            );
        }
    }
    let mut lzss_symbols: Vec<LzssSymbol> = Vec::new();
    let mut i = best_symbol_costs.len() - 1;

    loop {
        let symbol = &best_symbol_costs[i].1;
        lzss_symbols.push(symbol.clone());
        let jump = match symbol {
            LzssSymbol::Backreference(_, len) => *len as usize,
            _ => 1,
        };

        if jump > i {
            break;
        } else {
            i -= jump;
        }
    }

    // unsafe {
    //     println!("LAST KEY HITS {LAST_KEY_HITS}, TOTAL {TOTAL_ATTEMPS}");
    // }

    lzss_symbols
}

fn cost_of_encoding_backreference(
    (distance, length): (u16, u16),
    lengths_encoding_costs: [u32; LZSS_NUMBER_OF_LENGHTS],
    distance_encoding_costs: [u32; LZSS_NUMBER_OF_DISTANCES],
) -> u32 {
    let length_cost = lengths_encoding_costs[length as usize];
    let distance_cost = distance_encoding_costs[distance as usize];

    length_cost + distance_cost
}

fn find_backreference_with_table(
    bytes: &[u8],
    cursor: usize,
    window_size: usize,
    table: &mut LzssHashTable,
) -> Option<(u16, u16)> {
    let best_match = table.search(bytes, cursor, cursor.max(window_size) - window_size);

    if cursor + 2 < bytes.len() {
        table.insert(cursor, bytes, first_byte_repeat_count(&bytes[cursor..]));
    }

    best_match
}

pub fn decode_lzss(
    target: &mut Vec<u8>,
    lzss_symbols: &[LzssSymbol],
) -> Result<(), DeflateDecodeError> {
    for (i, symbol) in lzss_symbols.iter().enumerate() {
        match symbol {
            LzssSymbol::Literal(literal) => target.push(*literal),
            LzssSymbol::Backreference(distance, length) => {
                let backreference_data_start = match target.len().checked_sub(*distance as usize) {
                    Some(n) => n,
                    None => {
                        return Err(DeflateDecodeError(format!(
                        "Invalid backreference for lzss symbol {i}, distance {distance} is too big"
                    )))
                    }
                };

                //we must do this byte by byte in case there are repetitions
                for i in backreference_data_start..backreference_data_start + *length as usize {
                    target.push(target[i]);
                }
            }
            LzssSymbol::EndOfBlock => {
                break;
            }
        }
    }

    Ok(())
}

pub fn encode_lzss_to_bitstream<'a>(
    lzss_stream: impl Iterator<Item = &'a LzssSymbol>,
    ll_table: &HashMap<u16, WriteBitStream>,
    distance_table: &HashMap<u16, WriteBitStream>,
    target: &mut WriteBitStream,
) {
    for lzss_symbol in lzss_stream {
        match lzss_symbol {
            LzssSymbol::Literal(lit) => {
                target.extend(ll_table.get(&(*lit as u16)).unwrap());
            }
            LzssSymbol::Backreference(dist, len) => {
                let length_code = LENGTH_TO_CODE[*len as usize];
                let encoded_length_code = ll_table.get(&length_code).unwrap();
                target.extend(encoded_length_code);

                let (len_extra_bits, len_num_extra_bits) = LENGTH_TO_EXTRA_BITS[*len as usize];
                target.push_u16_rtl(len_extra_bits, len_num_extra_bits);

                let distance_code = DISTANCE_TO_CODE[*dist as usize];
                let encoded_distance_code = distance_table.get(&distance_code).unwrap();
                target.extend(encoded_distance_code);

                let (dist_extra_bits, dist_num_extra_bits) = DISTANCE_TO_EXTRA_BITS[*dist as usize];
                target.push_u16_rtl(dist_extra_bits, dist_num_extra_bits);
            }
            LzssSymbol::EndOfBlock => {
                target.extend(ll_table.get(&END_OF_BLOCK_MARKER_VALUE).unwrap())
            }
        }
    }
}
