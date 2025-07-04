pub mod backreference;
mod hash;
pub use hash::LzssHashTable;

use std::collections::HashMap;

use backreference::{
    DISTANCE_TO_CODE, DISTANCE_TO_EXTRA_BITS, LENGTH_TO_CODE, LENGTH_TO_EXTRA_BITS,
};

use crate::png::{deflate::encode_block_type_two, CompressionLevel};

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
        lzss_symbols = encode_lzss_iteration(bytes, ll_code_lengths, distance_code_lengths);
        // print!("Round {_i} ");
        // symbol_stats(&lzss_symbols);
    }

    lzss_symbols
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
    ll_code_lengths: HashMap<u16, u32>,
    distance_code_lengths: HashMap<u16, u32>,
) -> Vec<LzssSymbol> {
    //this is in reverse order
    let mut best_symbol_costs: Vec<(u32, LzssSymbol)> = Vec::new();
    let mut lzss_table = LzssHashTable::new(CompressionLevel::Best);
    for i in (bytes.len().max(LZSS_WINDOW_SIZE) - LZSS_WINDOW_SIZE)..(bytes.len() - 2) {
        lzss_table.insert(i, bytes[i], bytes[i + 1], bytes[i + 2], 0, bytes.len());
    }
    let ll_default: u32 =
        ll_code_lengths.iter().map(|(_code, l)| *l).sum::<u32>() / (ll_code_lengths.len() as u32);
    let distance_default: u32 = (distance_code_lengths
        .iter()
        .map(|(_code, l)| *l)
        .sum::<u32>()
        .checked_div(distance_code_lengths.len() as u32)
        .unwrap_or(0))
        * 2;
    let (mut chose_bf, mut chose_lit) = (0, 0);

    for (cost_list_index, byte) in bytes.iter().rev().enumerate() {
        let bytes_index = bytes.len() - cost_list_index - 1;
        let literal_cost = *ll_code_lengths.get(&(*byte as u16)).unwrap_or(&ll_default)
            + cost_list_index
                .checked_sub(1)
                .map(|idx| best_symbol_costs[idx].0)
                .unwrap_or(0);
        let backreferences: Vec<(u16, u16)> = lzss_table
            .get_all_backreferences(&bytes, bytes_index)
            .unwrap_or(Vec::new());

        let (bf, bf_cost) = backreferences
            .into_iter()
            .map(|bf| {
                let bf_encode_cost = cost_of_encoding_backreference(
                    bf,
                    &ll_code_lengths,
                    &distance_code_lengths,
                    ll_default,
                    distance_default,
                );
                // println!(
                //     "(char {}) Currently lookin at bf {:?}",
                //     char::from_u32(*byte as u32).unwrap(),
                //     bf,
                // );
                let bf_end_cost = cost_list_index
                    .checked_sub(bf.1 as usize)
                    .map(|idx| best_symbol_costs.get(idx).map(|(cost, _sym)| *cost))
                    .flatten()
                    .unwrap_or(0);
                // println!(
                //     "{} {:?} - cost {}",
                //     best_symbol_costs.len(),
                //     bf,
                //     bf_encode_cost + bf_end_cost
                // );

                // (bf, bf_encode_cost + bf_end_cost)
                (bf, bf_encode_cost + bf_end_cost)
            })
            .min_by_key(|(_bf, cost)| *cost)
            .unwrap_or(((0, 0), literal_cost));

        if bf_cost < literal_cost {
            // println!("chose bf {chose_bf}");
            chose_bf += 1;
            let symbol = LzssSymbol::Backreference(bf.0, bf.1);
            best_symbol_costs.push((bf_cost, symbol));
        } else {
            // println!("chose lit {chose_lit}");
            chose_lit += 1;
            best_symbol_costs.push((literal_cost, LzssSymbol::Literal(*byte)));
        }

        if bytes_index >= LZSS_WINDOW_SIZE {
            let first_byte_index_in_window = bytes_index - LZSS_WINDOW_SIZE;
            lzss_table.insert(
                first_byte_index_in_window,
                bytes[first_byte_index_in_window],
                bytes[first_byte_index_in_window + 1],
                bytes[first_byte_index_in_window + 2],
                first_byte_index_in_window,
                bytes_index,
            );
        }
    }
    // println!("Costs {:?}", best_symbol_costs);
    // println!("Chose bf {chose_bf}, chose lit {chose_lit}");
    let mut lzss_symbols: Vec<LzssSymbol> = Vec::new();
    let mut i = 0;
    let best_symbols: Vec<_> = best_symbol_costs.into_iter().rev().collect();

    while i < bytes.len() {
        let symbol = &best_symbols[i].1;
        lzss_symbols.push(symbol.clone());
        // println!("{i}");
        i += match symbol {
            LzssSymbol::Backreference(_, len) => *len as usize,
            _ => 1,
        };
    }

    lzss_symbols

    // best_symbol_costs
    //     .into_iter()
    //     .rev()
    //     .map(|(_cost, symbol)| symbol)
    //     .collect()
}

pub fn cost_of_encoding_backreference(
    (distance, length): (u16, u16),
    ll_code_lengths: &HashMap<u16, u32>,
    distance_code_lengths: &HashMap<u16, u32>,
    ll_default: u32,
    distance_defualt: u32,
) -> u32 {
    let length_code = LENGTH_TO_CODE[length as usize];
    let length_code_bits = *ll_code_lengths.get(&length_code).unwrap_or(&ll_default);
    let length_extra_bits = LENGTH_TO_EXTRA_BITS[length as usize].1 as u32;

    let distance_code = DISTANCE_TO_CODE[distance as usize];
    let dist_code_bits = *distance_code_lengths
        .get(&distance_code)
        .unwrap_or(&distance_defualt);
    let dist_extra_bits = DISTANCE_TO_EXTRA_BITS[distance as usize].1 as u32;

    length_code_bits + length_extra_bits + dist_code_bits + dist_extra_bits
}

fn find_backreference_with_table(
    bytes: &[u8],
    cursor: usize,
    window_size: usize,
    table: &mut LzssHashTable,
) -> Option<(u16, u16)> {
    let best_match = table.search(bytes, cursor, cursor.max(window_size) - window_size);

    if cursor + 2 < bytes.len() {
        table.insert(
            cursor,
            bytes[cursor],
            bytes[cursor + 1],
            bytes[cursor + 2],
            cursor.saturating_sub(LZSS_WINDOW_SIZE),
            cursor,
        );
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
                            "Invalid backreference for lzss symbol {}, distance {} is too big",
                            i, distance
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
                target.extend(&encoded_distance_code);

                let (dist_extra_bits, dist_num_extra_bits) = DISTANCE_TO_EXTRA_BITS[*dist as usize];
                target.push_u16_rtl(dist_extra_bits, dist_num_extra_bits);
            }
            LzssSymbol::EndOfBlock => {
                target.extend(ll_table.get(&END_OF_BLOCK_MARKER_VALUE).unwrap())
            }
        }
    }
}
