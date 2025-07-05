use std::collections::{HashMap, VecDeque};

pub struct PriorityQueue<T> {
    queue: VecDeque<(T, u32)>,
}

impl<T> PriorityQueue<T> {
    fn from_vec(mut vec: Vec<(T, u32)>) -> PriorityQueue<T> {
        vec.sort_by_key(|(_, p)| *p);

        PriorityQueue { queue: vec.into() }
    }

    pub fn new() -> PriorityQueue<T> {
        PriorityQueue {
            queue: VecDeque::new(),
        }
    }

    pub fn enqueue(&mut self, new_value: T, new_value_priority: u32) {
        let mut index = self.queue.len();

        for (i, (_, priority)) in self.queue.iter().enumerate() {
            if new_value_priority < *priority {
                index = i;
                break;
            }
        }

        self.queue.insert(index, (new_value, new_value_priority));
    }

    pub fn dequeue(&mut self) -> Option<(T, u32)> {
        self.queue.pop_back()
    }

    pub fn dequeue_front(&mut self) -> Option<(T, u32)> {
        self.queue.pop_front()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

impl<T: ToString> PriorityQueue<T> {
    fn print(&self) {
        for (elem, frequency) in self.queue.iter() {
            print!("{} {} -> ", elem.to_string(), frequency)
        }
        println!();
    }
}

pub fn get_letter_frequencies(s: &str) -> PriorityQueue<char> {
    let mut frequencies: HashMap<char, u32> = HashMap::new();

    for c in s.chars() {
        match frequencies.get_mut(&c) {
            Some(occurences) => {
                *occurences += 1;
            }
            None => {
                frequencies.insert(c, 1);
            }
        }
    }

    let mut result: PriorityQueue<char> = PriorityQueue::from_vec(vec![]);

    for (c, frequency) in frequencies.into_iter() {
        result.enqueue(c, frequency);
    }

    result
}
