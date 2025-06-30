use std::collections::{HashMap, VecDeque};

pub struct LzssHashTable {
    map: HashMap<(u8, u8, u8), VecDeque<usize>>,
}

const MAX_CHAIN_SIZE: usize = 10;

impl LzssHashTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
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
            .filter(|idx| **idx >= window_start_index)
            .map(|idx| {
                (
                    idx,
                    number_of_matching_bytes(
                        &whole_input[cursor..(cursor + 258).min(whole_input.len())],
                        &whole_input[*idx..(cursor + 258).min(whole_input.len())],
                    ),
                )
            })
            .max_by_key(|(_idx, length)| *length)?;
        let backreference = ((cursor - index) as u16, length as u16);

        Some(backreference)
    }

    fn get_chain(&self, byte_sequence: &[u8]) -> Option<&VecDeque<usize>> {
        let byte_prefix = (
            *byte_sequence.get(0)?,
            *byte_sequence.get(1)?,
            *byte_sequence.get(2)?,
        );

        self.map.get(&byte_prefix)
    }
    pub fn insert(&mut self, cursor: usize, byte1: u8, byte2: u8, byte3: u8) {
        let key = (byte1, byte2, byte3);
        let chain = self.map.get_mut(&key);

        match chain {
            None => {
                let chain = VecDeque::from([cursor]);
                self.map.insert(key, chain);
            }
            Some(chain) => {
                chain.push_back(cursor);

                if chain.len() > MAX_CHAIN_SIZE {
                    chain.pop_front();
                }
            }
        }
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
