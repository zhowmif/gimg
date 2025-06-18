use std::{collections::HashMap, hash::Hash};

#[derive(Debug)]
pub enum CLCode {
    SingleLength(u32),
    Sixteen { repeat_count: usize },
    Seventeen { repeat_count: usize },
    Eighteen { repeat_count: usize },
}

pub fn get_cl_codes_for_code_lengths<T: Eq + Hash>(
    sorted_alphabet: &[T],
    symbol_code_lengths: HashMap<T, u32>,
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
