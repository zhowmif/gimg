use std::collections::{HashMap, VecDeque};

use crate::png::{deflate::consts::LZSS_WINDOW_SIZE, CompressionLevel};

#[derive(Clone)]
pub struct LzssHashTable {
    map: HashMap<u32, VecDeque<usize>>,
    compression_level: CompressionLevel,
}

const MAX_SMALL_CHAIN_SIZE: usize = 10;

impl LzssHashTable {
    pub fn new(compression_level: CompressionLevel) -> Self {
        Self {
            map: HashMap::new(),
            compression_level,
        }
    }

    pub fn search(
        &self,
        whole_input: &[u8],
        cursor: usize,
        window_start_index: usize,
    ) -> Option<(u16, u16)> {
        let chain = self.get_chain(&whole_input[cursor..])?;
        let (index, length) = chain
            .iter()
            .filter(|idx| **idx < cursor && **idx >= window_start_index)
            .map(|idx| {
                (
                    idx,
                    number_of_matching_bytes(
                        &whole_input[cursor..(cursor + 258).min(whole_input.len())],
                        &whole_input[*idx..(cursor + 258).min(whole_input.len())],
                    ),
                )
            })
            .max_by_key(|(idx, length)| match self.compression_level {
                CompressionLevel::Best => {
                    let distance = cursor - *idx;
                    let penalty = if distance > 2048 { 1 } else { 0 };

                    *length - penalty
                }
                _ => *length,
            })?;

        let backreference = ((cursor - index) as u16, length as u16);

        Some(backreference)
    }

    fn get_chain(&self, byte_sequence: &[u8]) -> Option<&VecDeque<usize>> {
        let key = Self::get_key(
            *byte_sequence.first()?,
            *byte_sequence.get(1)?,
            *byte_sequence.get(2)?,
        );

        self.map.get(&key)
    }

    #[inline(always)]
    fn get_key(b1: u8, b2: u8, b3: u8) -> u32 {
        ((b1 as u32) << 16) + ((b2 as u32) << 8) + (b3 as u32)
    }

    pub fn insert(
        &mut self,
        cursor: usize,
        byte1: u8,
        byte2: u8,
        byte3: u8,
        window_start: usize,
        window_end: usize,
    ) {
        let key = Self::get_key(byte1, byte2, byte3);
        let chain = self.map.get_mut(&key);

        match chain {
            None => {
                let chain = VecDeque::from([cursor]);
                self.map.insert(key, chain);
            }
            Some(chain) => {
                chain.push_back(cursor);

                match self.compression_level {
                    CompressionLevel::Best => {
                        chain.retain(|idx| *idx >= window_start && *idx <= window_end)
                    }
                    _ => {
                        if chain.len() > MAX_SMALL_CHAIN_SIZE {
                            chain.pop_front();
                        }
                    }
                }
            }
        }
    }

    pub fn get_all_backreferences(
        &self,
        whole_input: &[u8],
        cursor: usize,
    ) -> Option<Vec<(u16, u16)>> {
        let window_start_index = cursor.max(LZSS_WINDOW_SIZE) - LZSS_WINDOW_SIZE;
        let chain = self.get_chain(&whole_input[cursor..])?;
        let backreferences: Vec<_> = chain
            .iter()
            .filter(|idx| **idx < cursor && **idx >= window_start_index)
            .map(|idx| {
                (
                    (cursor - *idx) as u16,
                    number_of_matching_bytes(
                        &whole_input[cursor..(cursor + 258).min(whole_input.len())],
                        &whole_input[*idx..(cursor + 258).min(whole_input.len())],
                    ) as u16,
                )
            })
            .collect();

        Some(backreferences)
    }
}

fn number_of_matching_bytes(a: &[u8], b: &[u8]) -> usize {
    let mut res = 0;
    let m = a.len().min(b.len());
    while res < m && a[res] == b[res] {
        res += 1;
    }

    res
}
