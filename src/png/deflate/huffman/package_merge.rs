use std::{collections::HashMap, fmt::Debug, hash::Hash, mem};

pub struct PackageMergeEncoder<T: Eq + Hash + Clone> {
    symbol_frequencies: HashMap<T, u32>,
}

impl<T: Eq + Hash + Clone + Debug + Ord> PackageMergeEncoder<T> {
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

    pub fn get_symbol_lengths(&mut self, max_code_length: usize) -> HashMap<T, u32> {
        if self.symbol_frequencies.len() > (1 << max_code_length) {
            panic!(
                "Cannot produce {max_code_length} for {} different values",
                self.symbol_frequencies.len()
            )
        }

        let mut symbol_frequencies: Vec<(T, u32)> = mem::take(&mut self.symbol_frequencies)
            .into_iter()
            .collect();
        symbol_frequencies.sort();

        if symbol_frequencies.len() == 1 {
            let mut result = HashMap::new();
            result.insert(symbol_frequencies.remove(0).0, 1);

            return result;
        }

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
            .flat_map(|queue| queue.0)
            .collect();
        let symbol_lengths = symbol_frequencies
            .into_iter()
            .map(|(symbol, _f)| {
                let count = all_used_coins
                    .iter()
                    .filter(|coin| **coin == symbol)
                    .count() as u32;

                (symbol, count)
            })
            .collect();

        symbol_lengths
    }
}
