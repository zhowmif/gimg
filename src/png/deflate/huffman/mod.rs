use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::mem;

use crate::queue::PriorityQueue;

use super::new_bitsream::NewBitStream;

pub mod package_merge;

pub struct HuffmanEncoder<T: Eq + Hash + Clone> {
    symbol_frequencies: HashMap<T, u32>,
}

impl<T: Eq + Hash + Clone> HuffmanEncoder<T> {
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

    fn build_priority_queue(&mut self) -> PriorityQueue<Vec<(T, u32)>> {
        let mut queue = PriorityQueue::new();
        let symbol_frequencies = mem::replace(&mut self.symbol_frequencies, HashMap::new());

        for (symbol, frequency) in symbol_frequencies.into_iter() {
            queue.enqueue(vec![(symbol, 0)], frequency);
        }

        queue
    }

    pub fn get_symbol_lengths(&mut self) -> Vec<(T, u32)> {
        let mut symbol_queue = self.build_priority_queue();

        while symbol_queue.len() > 1 {
            let (mut a, a_freq) = symbol_queue.dequeue_front().unwrap();
            let (b, b_freq) = symbol_queue.dequeue_front().unwrap();
            let new_priority = a_freq + b_freq;
            a.extend(b);
            for ele in a.iter_mut() {
                ele.1 += 1;
            }

            symbol_queue.enqueue(a, new_priority);
        }
        let mut symbol_lengths = symbol_queue.dequeue().unwrap().0;
        symbol_lengths.sort_by_key(|(_, len)| *len);

        symbol_lengths
    }
}

pub fn construct_canonical_tree_from_lengths<T: Eq + Hash + Clone>(
    symbol_lengths: &HashMap<T, u32>,
) -> HashMap<T, NewBitStream> {
    let mut symbol_lengths: Vec<_> = symbol_lengths.into_iter().collect();
    symbol_lengths.sort_by_key(|(_symbol, len)| **len);

    let mut symbol_codes = HashMap::new();
    let h = *symbol_lengths.last().unwrap().1;
    let mut b = 0;
    for (symbol, length) in symbol_lengths.into_iter() {
        let m = saturating_shl(b, (8 as u32).saturating_sub(h));
        let p = NewBitStream::from_u32_lsb(m, *length as u8);
        symbol_codes.insert(symbol.clone(), p);
        b += 1 << (h - length);
    }

    symbol_codes
}

fn saturating_shl(lhs: u32, rhs: u32) -> u32 {
    lhs.checked_shl(rhs as u32).unwrap_or(0)
}
