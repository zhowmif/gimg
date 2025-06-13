use std::fmt::Display;

#[derive(Debug)]

pub struct NewBitStream {
    stream: Vec<u8>,
    working_byte: u8,
    current_bit_number: u8,
}

impl NewBitStream {
    pub fn new() -> Self {
        Self {
            stream: Vec::new(),
            working_byte: 0,
            current_bit_number: 0,
        }
    }

    pub fn from_byte_msb(byte: u8, len: u8) -> Self {
        Self {
            stream: Vec::new(),
            working_byte: byte,
            current_bit_number: len,
        }
    }

    pub fn from_u32_msb(num: u32, offset: u8) -> Self {
        let mut bitstream = NewBitStream::new();
        let mut mask = (1 as u32) << (offset - 1);

        while mask > 0 {
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
        self.working_byte <<= 1;
        self.current_bit_number += 1;
        self.flush_working_byte();
    }

    pub fn push_one(&mut self) {
        self.working_byte = self.working_byte << 1 | 1;
        self.current_bit_number += 1;
        self.flush_working_byte();
    }

    pub fn push_u8_lsb(&mut self, n: u8, bitsize: u8) {
        let g = 8 - self.current_bit_number;
        if g > bitsize {
            let bits_to_add_from_n = u8::reverse_bits(n) >> (8 - bitsize);
            self.working_byte = (self.working_byte << bitsize) + bits_to_add_from_n;
            self.current_bit_number += bitsize;
        } else {
            let bits_to_add_from_n = u8::reverse_bits(n) >> (8 - g);
            self.stream
                .push(saturating_shr(self.working_byte, g) + bits_to_add_from_n);
            let bits_left = bitsize - g;
            self.current_bit_number = bits_left;
            self.working_byte =
                saturating_shr(u8::reverse_bits(saturating_shr(n, g)), 8 - bits_left);
        }
    }

    pub fn push_byte_lsb(&mut self, byte: u8) {
        self.push_u8_lsb(byte, 8);
    }

    pub fn push_bytes_lsb(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.push_byte_lsb(*byte);
        }
    }

    pub fn push_u16_lsb(&mut self, n: u16) {
        self.push_byte_lsb(n as u8);
        self.push_byte_lsb((n >> 8) as u8);
    }

    pub fn extend_aligned(&mut self, other: &Self) {
        if self.current_bit_number == 0 {
            self.stream.extend_from_slice(&other.stream);
        } else {
            panic!("Can't extend when current bitstream is not aligned");
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

    //TODO: this should probably transfer ownership instead of cloning
    pub fn to_bytes(&self) -> Vec<u8> {
        if self.current_bit_number != 0 {
            panic!("Can't convert to bytes if it's not good");
        }

        self.stream.clone()
    }
}

impl Display for NewBitStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.stream.iter() {
            write!(f, "{:08b}", byte)?
        }

        if self.current_bit_number != 0 {
            write!(
                f,
                "{:0width$b}",
                self.working_byte >> (8 - self.current_bit_number),
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
