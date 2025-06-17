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

    pub fn push_u16_lsb(&mut self, n: u16) {
        self.push_byte_lsb(n as u8);
        self.push_byte_lsb((n >> 8) as u8);
    }

    pub fn extend(&mut self, other: &Self) {
        if self.current_bit_number == 0 {
            self.stream.extend_from_slice(&other.stream);
            return;
        }

        for byte in other.stream.iter() {
            self.push_u8_lsb_ltr(*byte, 8);
        }

        if other.current_bit_number != 0 {
            self.push_u8_lsb_ltr(other.working_byte, other.current_bit_number);
        }
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

    pub fn push_u16_msb(&mut self, num: u16, len: u8) {
        if len > 8 {
            self.push_u8_msb((num >> 8) as u8, 8);
            self.push_u8_msb(num as u8, len - 8);
        } else {
            self.push_u8_msb(num as u8, len);
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
