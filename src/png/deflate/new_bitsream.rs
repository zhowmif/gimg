use std::fmt::Display;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct NewBitStream {
    stream: Vec<u8>,
    working_byte: u8,
    pub current_bit_number: u8,
}

impl NewBitStream {
    pub fn new() -> Self {
        Self {
            stream: Vec::new(),
            working_byte: 0,
            current_bit_number: 0,
        }
    }

    pub fn from_u32_lsb(num: u32, length: u8) -> Self {
        let mut bitstream = NewBitStream::new();
        let mut mask = 1;

        for _i in 0..length {
            match num & mask {
                0 => bitstream.push_zero(),
                _ => bitstream.push_one(),
            };

            mask <<= 1;
        }

        bitstream
    }

    pub fn from_u32_msb(num: u32, length: u8) -> Self {
        Self::from_u32_msb_ltr(num, length as usize, length)
    }

    pub fn from_u32_msb_ltr(num: u32, start_index: usize, length: u8) -> Self {
        let mut bitstream = NewBitStream::new();
        let mut mask = 1 << (start_index - 1);

        for _i in 0..length {
            match num & mask {
                0 => bitstream.push_zero(),
                _ => bitstream.push_one(),
            };

            mask >>= 1;
        }

        bitstream
    }

    fn flush_working_byte(&mut self) {
        if self.current_bit_number == 8 {
            self.stream.push(self.working_byte);
            self.current_bit_number = 0;
            self.working_byte = 0;
        }
    }

    pub fn push_zero(&mut self) {
        self.working_byte >>= 1;
        self.current_bit_number += 1;
        self.flush_working_byte();
    }

    pub fn push_one(&mut self) {
        self.working_byte = (self.working_byte >> 1) | 0b10000000;
        self.current_bit_number += 1;
        self.flush_working_byte();
    }

    pub fn push_byte_lsb(&mut self, byte: u8) {
        self.push_u8_lsb(byte, 8);
    }

    pub fn push_bytes_lsb(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.push_byte_lsb(*byte);
        }
    }

    pub fn push_u16_lsb_le(&mut self, n: u16) {
        self.push_byte_lsb(n as u8);
        self.push_byte_lsb((n >> 8) as u8);
    }

    pub fn extend(&mut self, other: &Self) {
        for byte in other.stream.iter() {
            self.push_u8_lsb_ltr(*byte, 8);
        }

        if other.current_bit_number != 0 {
            self.push_u8_lsb_ltr(other.working_byte, other.current_bit_number);
        }
    }

    pub fn extend_reverse(&mut self, other: &Self) {
        let mut other = other.clone();
        let other_len = other.len();
        let other_bytes = other.flush_to_bytes();

        if other_bytes.len() > 1 {
            panic!("I did not expect this");
        }

        self.push_u8_lsb(other_bytes[0], other_len as u8);
    }

    pub fn push_u8_msb(&mut self, num: u8, length: u8) {
        let mut mask = 1 << (length - 1);

        while mask > 0 {
            match num & mask {
                0 => self.push_zero(),
                _ => self.push_one(),
            };

            mask >>= 1;
        }
    }

    pub fn push_u8_lsb(&mut self, num: u8, length: u8) {
        let mut mask = 1u16;

        while mask <= 1 << (length - 1) {
            match num as u16 & mask {
                0 => self.push_zero(),
                _ => self.push_one(),
            };

            mask <<= 1;
        }
    }

    pub fn push_u8_lsb_ltr(&mut self, num: u8, length: u8) {
        let mut mask = 1 << 7;

        for _i in 0..length {
            match num & mask {
                0 => self.push_zero(),
                _ => self.push_one(),
            };

            mask >>= 1;
        }
    }

    pub fn push_u16_msb_le(&mut self, num: u16, len: u8) {
        if len == 0 {
            return;
        }

        let mut mask = 1u16;

        for _i in 0..len {
            match num & mask {
                0 => self.push_zero(),
                _ => self.push_one(),
            };

            mask <<= 1;
        }
        // if len > 8 {
        //     self.push_u8_lsb((num >> 8) as u8, 8);
        //     self.push_u8_lsb(num as u8, len - 8);
        // } else if len > 0 {
        //     self.push_u8_lsb(num as u8, len);
        // }
    }

    pub fn test_me(&mut self, num: u16, len: u8) {
        if len == 0 {
            return;
        }

        let mut mask = 1 << (len - 1);

        for _i in 0..len {
            match num & mask {
                0 => self.push_zero(),
                _ => self.push_one(),
            };

            mask >>= 1;
        }
    }

    pub fn read_byte_lsb(&self, bit_index: &mut usize) -> u8 {
        let byte_index = *bit_index >> 3;
        let containing_word: u16 = ((self.stream[byte_index] as u16) << 8)
            + (self
                .stream
                .get(byte_index + 1)
                .map(|b| b.clone())
                .unwrap_or(0) as u16);
        *bit_index += 8;
        let misalignment = (*bit_index as u16) & (8 - 1);
        let answer: u8 = ((containing_word << misalignment) >> 8) as u8;

        u8::reverse_bits(answer)
    }

    pub fn len(&self) -> usize {
        self.stream.len() * 8 + (self.current_bit_number as usize)
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn flush_to_bytes(&mut self) -> Vec<u8> {
        let mut bytes = std::mem::replace(&mut self.stream, Vec::new());

        if self.current_bit_number != 0 {
            bytes.push(self.working_byte >> (8 - self.current_bit_number));

            self.current_bit_number = 0;
            self.working_byte = 0;
        }

        bytes
    }
}

impl Display for NewBitStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.stream.iter() {
            write!(f, "{:08b}", u8::reverse_bits(*byte))?
        }

        if self.current_bit_number != 0 {
            write!(
                f,
                "{:0width$b}",
                u8::reverse_bits(self.working_byte),
                width = self.current_bit_number as usize
            )?;
        }

        Ok(())
    }
}

fn saturating_shr(lhs: u8, rhs: u8) -> u8 {
    lhs.checked_shr(rhs as u32).unwrap_or(0)
}

fn saturating_shl(lhs: u8, rhs: u8) -> u8 {
    lhs.checked_shl(rhs as u32).unwrap_or(0)
}

pub struct BitStreamReader<'a> {
    bytes: &'a [u8],
    bit_index: usize,
}

impl<'a> BitStreamReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            bit_index: 0,
        }
    }

    pub fn read_bit(&mut self) -> u8 {
        let byte = self.bytes[self.bit_index >> 3];
        let bit = (byte >> (self.bit_index & 0b111)) & 1;
        self.bit_index += 1;

        bit
    }

    pub fn read_bit_boolean(&mut self) -> bool {
        self.read_bit() == 1
    }

    pub fn read_number_lsb(&mut self, length: usize) -> u16 {
        let mut number: u16 = 0;

        for shift in 0..length {
            number |= ((self.read_bit() as u16) << shift) as u16;
        }

        number
    }

    pub fn align_to_next_byte(&mut self) {
        if self.bit_index & 0b111 != 0 {
            self.bit_index = ((self.bit_index >> 3) + 1) << 3;
        }
    }

    pub fn read_u16_lsb_le(&mut self) -> u16 {
        let first = self.read_number_lsb(8);
        let second = self.read_number_lsb(8);
        let number = (second << 8) + first;

        number
    }

    pub fn read_bytes_aligned(&mut self, length: usize) -> &[u8] {
        let byte_index = self.bit_index >> 3;
        self.bit_index = (byte_index + length) << 3;

        &self.bytes[byte_index..byte_index + length]
    }
}
