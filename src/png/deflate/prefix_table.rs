use std::{collections::HashMap, hash::Hash, iter::repeat_n};

use super::new_bitsream::{BitStreamReader, NewBitStream};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CLCode {
    SingleLength(u32),
    Sixteen { repeat_count: usize },
    Seventeen { repeat_count: usize },
    Eighteen { repeat_count: usize },
}

impl CLCode {
    pub fn to_number(&self) -> u32 {
        match self {
            CLCode::SingleLength(length) => *length,
            CLCode::Sixteen { repeat_count: _ } => 16,
            CLCode::Seventeen { repeat_count: _ } => 17,
            CLCode::Eighteen { repeat_count: _ } => 18,
        }
    }

    pub fn parse_from_bitstream(number: u32, bitstream: &mut BitStreamReader) -> Self {
        match number {
            0..=15 => CLCode::SingleLength(number),
            16 => {
                let repeat_count = bitstream.read_number_lsb(2).into();
                CLCode::Sixteen { repeat_count }
            }
            17 => {
                let repeat_count = bitstream.read_number_lsb(3).into();
                CLCode::Seventeen { repeat_count }
            }
            18 => {
                let repeat_count = bitstream.read_number_lsb(7).into();
                CLCode::Eighteen { repeat_count }
            }

            _ => panic!("Unrecognized cl code"),
        }
    }

    pub fn encode(&self, cl_codes: &HashMap<u32, NewBitStream>, target: &mut NewBitStream) {
        // print!(" {} ", cl_codes.get(&self.to_number()).unwrap());
        target.deprecated_extend_reverse(cl_codes.get(&self.to_number()).unwrap());

        match self {
            CLCode::SingleLength(_) => {}
            CLCode::Sixteen { repeat_count } => target.push_u8_lsb(*repeat_count as u8, 2),
            CLCode::Seventeen { repeat_count } => target.push_u8_lsb(*repeat_count as u8, 3),
            CLCode::Eighteen { repeat_count } => target.push_u8_lsb(*repeat_count as u8, 7),
        }
    }

    pub fn expand(&self, previous_value: u32) -> Vec<u32> {
        match self {
            CLCode::SingleLength(length) => vec![*length],
            CLCode::Sixteen { repeat_count } => {
                repeat_n(previous_value, *repeat_count + 3).collect()
            }
            CLCode::Seventeen { repeat_count } => repeat_n(0, *repeat_count + 3).collect(),
            CLCode::Eighteen { repeat_count } => repeat_n(0, *repeat_count + 11).collect(),
        }
    }
}

pub fn get_cl_codes_for_code_lengths<T: Eq + Hash>(
    sorted_alphabet: &[T],
    symbol_code_lengths: &HashMap<T, u32>,
) -> Vec<CLCode> {
    let all_symbol_lengths: Vec<_> = sorted_alphabet
        .into_iter()
        .map(|symbol| symbol_code_lengths.get(&symbol).map(|l| *l).unwrap_or(0))
        .collect();
    let mut cl_codes = Vec::new();

    let mut i = 0;
    while i < all_symbol_lengths.len() {
        let current_symbol_length = all_symbol_lengths[i];
        let mut current_length_run_length = 1;

        while current_length_run_length + i < all_symbol_lengths.len()
            && ((current_symbol_length == 0 && current_length_run_length < 138)
                || current_length_run_length < 6)
            && all_symbol_lengths[i] == all_symbol_lengths[i + current_length_run_length]
        {
            current_length_run_length += 1;
        }

        if current_symbol_length == 0 && current_length_run_length >= 11 {
            cl_codes.push(CLCode::Eighteen {
                repeat_count: current_length_run_length - 11,
            })
        } else if current_symbol_length == 0 && current_length_run_length >= 3 {
            cl_codes.push(CLCode::Seventeen {
                repeat_count: current_length_run_length - 3,
            })
        } else if current_length_run_length >= 4 {
            cl_codes.push(CLCode::SingleLength(all_symbol_lengths[i]));
            cl_codes.push(CLCode::Sixteen {
                repeat_count: current_length_run_length - 4,
            });
        } else {
            for _i in 0..current_length_run_length {
                cl_codes.push(CLCode::SingleLength(all_symbol_lengths[i]))
            }
        }

        i += current_length_run_length;
    }

    cl_codes
}

pub fn number_of_zero_symbols_at_end<T: Eq + Hash>(
    sorted_alphabet: &[T],
    symbol_code_lengths: &HashMap<T, u32>,
) -> usize {
    let mut result = 0;

    for symbol in sorted_alphabet.iter().rev() {
        match symbol_code_lengths.get(symbol) {
            Some(_) => {
                break;
            }
            None => result += 1,
        }
    }

    return result;
}

pub fn reverse_hashmap<K, V: Eq + Hash>(map: HashMap<K, V>) -> HashMap<V, K> {
    map.into_iter().map(|(k, v)| (v, k)).collect()
}

pub fn generate_static_lit_len_table() -> HashMap<u16, NewBitStream> {
    let lengths = generate_bitstream_from_range(48, 191, 8)
        .into_iter()
        .chain(generate_bitstream_from_range(400, 511, 9))
        .chain(generate_bitstream_from_range(0, 23, 7))
        .chain(generate_bitstream_from_range(192, 199, 8))
        .enumerate()
        .map(|(i, val)| (i as u16, val))
        .collect();

    lengths
}

pub fn generate_static_distance_table() -> HashMap<u16, NewBitStream> {
    (0..30)
        .zip(0..30)
        .map(|(i, val)| (i as u16, NewBitStream::from_u32_ltr(val, 5)))
        .collect()
}

fn generate_bitstream_from_range(start: usize, end: usize, len: u8) -> Vec<NewBitStream> {
    (start..=end)
        .map(|n| NewBitStream::from_u32_ltr(n as u32, len))
        .collect()
}
