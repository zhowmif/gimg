use std::{collections::HashMap, fmt::Debug, hash::Hash, iter::repeat};

use crate::bits::Bits;

pub struct BinaryTree<T> {
    value: T,
    left: Option<Box<BinaryTree<T>>>,
    right: Option<Box<BinaryTree<T>>>,
}

impl<T> BinaryTree<T> {
    pub fn new(value: T) -> BinaryTree<T> {
        BinaryTree {
            value,
            left: None,
            right: None,
        }
    }

    pub fn new_branch(value: T, left: BinaryTree<T>, right: BinaryTree<T>) -> BinaryTree<T> {
        BinaryTree {
            value,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }
    }

    fn depth(&self) -> u32 {
        1 + self
            .left()
            .map_or(0, |l| l.depth())
            .max(self.right().map_or(0, |r| r.depth()))
    }

    fn left(&self) -> Option<&Box<BinaryTree<T>>> {
        self.left.as_ref()
    }

    fn right(&self) -> Option<&Box<BinaryTree<T>>> {
        self.right.as_ref()
    }

    fn left_mut(&mut self) -> Option<&mut Box<BinaryTree<T>>> {
        self.left.as_mut()
    }

    fn right_mut(&mut self) -> Option<&mut Box<BinaryTree<T>>> {
        self.right.as_mut()
    }

    fn insert_left(&mut self, value: T) {
        self.left = Some(Box::new(BinaryTree::new(value)));
    }

    fn insert_right(&mut self, value: T) {
        self.right = Some(Box::new(BinaryTree::new(value)));
    }
}

impl<T: Eq + Hash + Clone> BinaryTree<Option<T>> {
    pub fn get_huffman_codes(&self) -> HashMap<T, Bits> {
        let mut codes: HashMap<T, Bits> = HashMap::new();

        self.extend_huffman_codes(&mut codes, Bits::new(vec![]));

        codes
    }

    fn extend_huffman_codes(&self, codes: &mut HashMap<T, Bits>, current_code: Bits) {
        match &self.value {
            Some(val) => {
                codes.insert(val.clone(), current_code);
            }
            None => {
                self.left()
                    .unwrap()
                    .extend_huffman_codes(codes, current_code.push_zero());
                self.right()
                    .unwrap()
                    .extend_huffman_codes(codes, current_code.push_one());
            }
        }
    }
}

impl<T: Debug> BinaryTree<T> {
    fn fmt(&self) -> Vec<Vec<Option<String>>> {
        let left_vector = self.left.as_ref().map_or(vec![], |bt| bt.fmt());
        let right_vector = self.right.as_ref().map_or(vec![], |bt| bt.fmt());

        let mut result = vec![vec![Some(format!("{:?}", self.value))]];

        for layer in 0..right_vector.len().max(left_vector.len()) {
            let layer_width = (2 as u64).pow(layer as u32 + 1);

            let nones: Vec<Option<String>> = repeat(None).take(layer_width as usize / 2).collect();

            let l = left_vector.get(layer).unwrap_or(&nones);
            let r = right_vector.get(layer).unwrap_or(&nones);

            let mut layer: Vec<Option<String>> = vec![];

            layer.append(&mut l.clone());
            layer.append(&mut r.clone());

            result.push(layer);
        }

        return result;
    }

    fn longest_elem_len(&self) -> usize {
        let lm = self.left().map_or(0, |l| l.longest_elem_len());
        let rm = self.right().map_or(0, |r| r.longest_elem_len());

        lm.max(rm).max(format!("{:?}", self.value).len())
    }

    pub fn print(&self) {
        let formatted = self.fmt();
        let depth = formatted.len() as u32;
        let longest_elem_len = self.longest_elem_len();
        let longest_line_elems_len = (2 as usize).pow(depth - 1);

        let mut indexes: Vec<usize> = repeat(longest_elem_len + 1)
            .zip(1..)
            .map(|(a, b)| a * b)
            .take(longest_line_elems_len)
            .collect();

        for line in formatted.into_iter().rev() {
            let mut curr_index = 0;

            for (i, target_index) in indexes.iter().enumerate() {
                while curr_index < *target_index {
                    print!(" ");
                    curr_index += 1;
                }

                let str_elem = line[i]
                    .as_ref()
                    .map_or("@".repeat(longest_elem_len).to_string(), |s| s.clone());
                let padding_length = (longest_elem_len - str_elem.len()) as f32 / 2.0;

                let left_padding = " ".repeat(padding_length.floor() as usize);
                let right_padding = " ".repeat(padding_length.ceil() as usize);
                print!("{}{}{} ", left_padding, str_elem, right_padding);
                curr_index += longest_elem_len;
            }

            println!("");
            indexes = pair_averages(indexes);
        }
    }
}

fn pair_averages(v: Vec<usize>) -> Vec<usize> {
    if v.len() == 1 {
        return v;
    }
    let mut res = vec![];

    for t in v.chunks(2) {
        res.push((t[0] + t[1]) / 2);
    }

    res
}