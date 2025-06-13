use std::{collections::HashMap, fmt::Debug, hash::Hash, mem};

pub struct PackageMergeEncoder<T: Eq + Hash + Clone> {
    symbol_frequencies: HashMap<T, u32>,
}

impl<T: Eq + Hash + Clone + Debug> PackageMergeEncoder<T> {
    pub fn new() -> Self {
        Self {
            symbol_frequencies: HashMap::new(),
        }
    }

    pub fn add_symbol(&mut self, symbol: &T) {
        match self.symbol_frequencies.get_mut(symbol) {
            Some(frequency) => *frequency += 1,
            None => {
                self.symbol_frequencies.insert(symbol.clone(), 1);
            }
        };
    }

    pub fn get_symbol_lengths(&mut self, max_code_length: usize) -> Vec<(T, u32)> {
        if self.symbol_frequencies.len() > (1 << max_code_length) {
            panic!("Cannot produce {max_code_length} for {} different values", self.symbol_frequencies.len())
        }
        let symbol_frequencies: Vec<(T, u32)> =
            mem::replace(&mut self.symbol_frequencies, HashMap::new())
                .into_iter()
                .collect();
        let starting_coin_queue: Vec<_> = symbol_frequencies
            .clone()
            .into_iter()
            .map(|(symbol, freq)| (vec![symbol], freq))
            .collect();

        let mut last_coin_queue_packages = Vec::new();
        for _i in 1..=max_code_length {
            let mut current_coin_queue = starting_coin_queue.clone();
            current_coin_queue.extend_from_slice(&last_coin_queue_packages);
            current_coin_queue.sort_by_key(|(_symbol, freq)| *freq);
            last_coin_queue_packages = Vec::new();
            while current_coin_queue.len() > 1 {
                let tmp: Vec<_> = current_coin_queue.drain(0..2).collect();
                let [a, b] = tmp.try_into().unwrap();
                last_coin_queue_packages
                    .push((a.0.into_iter().chain(b.0.into_iter()).collect(), a.1 + b.1));
            }
        }
        let all_used_coins: Vec<T> = last_coin_queue_packages
            .into_iter()
            .map(|queue| queue.0)
            .flatten()
            .collect();
        let mut symbol_lengths: Vec<_> = symbol_frequencies
            .into_iter()
            .map(|(symbol, _f)| {
                let count = all_used_coins
                    .iter()
                    .filter(|coin| **coin == symbol)
                    .count() as u32;

                (symbol, count)
            })
            .collect();
        symbol_lengths.sort_by_key(|(s, length)| *length);

        symbol_lengths
    }
}
