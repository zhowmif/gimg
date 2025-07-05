use std::fmt::Display;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct WriteBitStream {
    pub stream: Vec<u8>,
    pub buffer: u64,
    pub current_bit_number: u8,
}

impl WriteBitStream {
    pub fn new() -> Self {
        Self {
            stream: Vec::new(),
            buffer: 0,
            current_bit_number: 0,
        }
    }

    pub fn from_u32_ltr(num: u32, length: u8) -> Self {
        Self::from_u32_ltr_with_offset(num, length as usize, length)
    }

    pub fn from_u32_ltr_with_offset(num: u32, start_index: usize, length: u8) -> Self {
        let buffer =
            u64::reverse_bits((num >> (start_index - length as usize)) as u64) >> (64 - length);
        let mut res = Self {
            stream: Vec::new(),
            buffer,
            current_bit_number: length,
        };
        res.flush_buffer();
        res
    }

    fn flush_buffer(&mut self) {
        while self.current_bit_number >= 8 {
            let last_byte = (self.buffer & 0xff) as u8;
            self.stream.push(last_byte);
            self.buffer >>= 8;
            self.current_bit_number -= 8;
        }
    }

    pub fn push_zero(&mut self) {
        self.current_bit_number += 1;
        self.flush_buffer();
    }

    pub fn push_one(&mut self) {
        self.buffer |= 1 << self.current_bit_number;
        self.current_bit_number += 1;
        self.flush_buffer();
    }

    pub fn push_byte_ltr(&mut self, byte: u8) {
        self.push_u8_rtl(byte, 8);
    }

    pub fn push_bytes_ltr(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.push_byte_ltr(*byte);
        }
    }

    pub fn push_u16_ltr_le(&mut self, n: u16) {
        self.push_byte_ltr(n as u8);
        self.push_byte_ltr((n >> 8) as u8);
    }

    //assumes other is flushed
    pub fn extend(&mut self, other: &Self) {
        self.flush_buffer();
        for byte in other.stream.iter() {
            self.push_u8_rtl_from_middle(*byte, 8);
        }

        if other.current_bit_number != 0 {
            self.push_u8_rtl(other.buffer as u8, other.current_bit_number);
        }
    }

    pub fn push_u8_rtl(&mut self, num: u8, length: u8) {
        self.push_u16_rtl(num as u16, length);
    }

    pub fn push_u8_rtl_from_middle(&mut self, num: u8, length: u8) {
        let n = num >> (8 - length);
        self.push_u8_rtl(n, length);
    }

    pub fn push_u16_rtl(&mut self, num: u16, length: u8) {
        let sanitized_num = (num & ((1 << length) - 1)) as u64;
        self.buffer |= sanitized_num << self.current_bit_number;
        self.current_bit_number += length;
        self.flush_buffer();
    }

    pub fn len(&self) -> usize {
        self.stream.len() * 8 + (self.current_bit_number as usize)
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn flush_to_bytes(&mut self) -> Vec<u8> {
        self.flush_buffer();
        let mut bytes = std::mem::take(&mut self.stream);

        if self.current_bit_number != 0 {
            bytes.push(self.buffer as u8);
        }

        bytes
    }
}

impl Display for WriteBitStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut n = self.clone();
        n.flush_buffer();
        for byte in n.stream.iter() {
            write!(f, "{:08b}", u8::reverse_bits(*byte))?
        }

        if n.current_bit_number != 0 {
            let last_byte = u8::reverse_bits(n.buffer as u8) >> (8 - n.current_bit_number);
            write!(
                f,
                "{:0width$b}",
                last_byte,
                width = n.current_bit_number as usize
            )?;
        }

        Ok(())
    }
}

pub struct ReadBitStream<'a> {
    bytes: &'a [u8],
    bit_index: usize,
}

impl<'a> ReadBitStream<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            bit_index: 0,
        }
    }

    pub fn read_bit(&mut self) -> Option<u8> {
        let byte = self.bytes.get(self.bit_index >> 3)?;
        let bit = (byte >> (self.bit_index & 0b111)) & 1;
        self.bit_index += 1;

        Some(bit)
    }

    pub fn read_bit_boolean(&mut self) -> Option<bool> {
        Some(self.read_bit()? == 1)
    }

    pub fn read_number_lsb(&mut self, length: usize) -> Option<u16> {
        let mut number: u16 = 0;

        for shift in 0..length {
            number |= (self.read_bit()? << shift) as u16;
        }

        Some(number)
    }

    pub fn align_to_next_byte(&mut self) {
        if self.bit_index & 0b111 != 0 {
            self.bit_index = ((self.bit_index >> 3) + 1) << 3;
        }
    }

    pub fn read_u16_lsb_le(&mut self) -> Option<u16> {
        let first = self.read_number_lsb(8)?;
        let second = self.read_number_lsb(8)?;
        let number = (second << 8) + first;

        Some(number)
    }

    pub fn read_bytes_aligned(&mut self, length: usize) -> Option<&[u8]> {
        let byte_index = self.bit_index >> 3;

        if byte_index + length > self.bytes.len() {
            return None;
        }

        self.bit_index = (byte_index + length) << 3;

        Some(&self.bytes[byte_index..byte_index + length])
    }
}
