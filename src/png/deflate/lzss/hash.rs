use std::collections::{HashMap, VecDeque};

use crate::{png::CompressionLevel, simd_utils::number_of_matching_bytes};

use super::backreference::LZSS_MAX_LENGTH;

#[derive(Clone)]
pub struct LzssHashTable {
    map: HashMap<u32, VecDeque<(usize, usize)>>,
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
        let key = Self::get_key(whole_input, cursor)?;
        let chain = self.map.get(&key)?;
        let (index, length) = chain
            .iter()
            .filter(|(idx, _)| *idx < cursor && *idx >= window_start_index)
            .map(|(idx, _)| {
                (
                    idx,
                    number_of_matching_bytes(
                        &whole_input[cursor..(cursor + LZSS_MAX_LENGTH).min(whole_input.len())],
                        &whole_input[*idx..(cursor + LZSS_MAX_LENGTH).min(whole_input.len())],
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

    #[inline(always)]
    fn get_key(bytes: &[u8], cursor: usize) -> Option<u32> {
        let b1 = *bytes.get(cursor)?;
        let b2 = *bytes.get(cursor + 1)?;
        let b3 = *bytes.get(cursor + 2)?;

        Some(((b1 as u32) << 16) + ((b2 as u32) << 8) + (b3 as u32))
    }

    pub fn insert(&mut self, cursor: usize, bytes: &[u8], first_byte_repeat_count: usize) {
        let key = Self::get_key(bytes, cursor).expect("Must have at least 3 bytes to insert");
        let chain = self.map.get_mut(&key);

        match chain {
            None => {
                let chain = VecDeque::from([(cursor, first_byte_repeat_count)]);
                self.map.insert(key, chain);
            }
            Some(chain) => {
                chain.push_back((cursor, first_byte_repeat_count));

                match self.compression_level {
                    CompressionLevel::Best => {}
                    _ => {
                        if chain.len() > MAX_SMALL_CHAIN_SIZE {
                            chain.pop_front();
                        }
                    }
                }
            }
        }
    }

    pub fn get_all_backreferences<'a>(
        &'a mut self,
        whole_input: &[u8],
        cursor: usize,
    ) -> Option<&'a VecDeque<(usize, usize)>> {
        let key = Self::get_key(whole_input, cursor)?;

        let chain = self.map.get_mut(&key)?;

        while let Some(elem) = chain.front() {
            if elem.0 >= cursor {
                chain.pop_front();
            } else {
                break;
            }
        }

        Some(chain)
    }
}

#[inline(always)]
pub fn first_byte_repeat_count(bytes: &[u8]) -> usize {
    let first = bytes[0];
    bytes
        .iter()
        .take(LZSS_MAX_LENGTH)
        .take_while(|&&b| b == first)
        .count()
}
