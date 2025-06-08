use crate::bits::Bit;

#[derive(Debug)]
pub struct BitStream {
    bits: Vec<Bit>,
}

impl BitStream {
    pub fn new() -> Self {
        Self { bits: vec![] }
    }

    pub fn push_zero(&mut self) {
        self.bits.push(Bit::Zero);
    }

    pub fn push_one(&mut self) {
        self.bits.push(Bit::One);
    }

    pub fn push_number(&mut self, n: u16, bitsize: u8) {
        let mut n = n;

        for _i in 0..bitsize {
            if n & 1 == 0 {
                self.push_zero()
            } else {
                self.push_one()
            }

            n >>= 1
        }
    }

    pub fn push_byte(&mut self, byte: u8) {
        self.push_number(byte as u16, 8);
    }

    pub fn push_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.push_byte(*byte);
        }
    }

    pub fn extend(&mut self, other: &Self) {
        self.bits.extend_from_slice(&other.bits);
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        if self.bits.len() % 8 != 0 {
            panic!("Can't convert non aligned bits to bytes");
        }
        let mut bytes = Vec::with_capacity(self.bits.len() / 8);

        //TODO: use read_byte instead
        for byte_bits in self.bits.chunks(8) {
            let mut current_byte = 0;

            for i in (0..byte_bits.len()).rev() {
                current_byte <<= 1;

                let bit = &byte_bits[i];

                current_byte |= match bit {
                    Bit::Zero => 0,
                    Bit::One => 1,
                };
            }

            bytes.push(current_byte);
        }

        bytes
    }

    pub fn read_bit(&self, index: &mut usize) -> Bit {
        let bit = self.bits[*index].clone();
        *index += 1;

        bit
    }

    pub fn read_byte(&self, index: &mut usize) -> u8 {
        let byte_bits = &self.bits[*index..*index + 8];
        *index += 8;
        let mut current_byte = 0;

        for i in (0..byte_bits.len()).rev() {
            current_byte <<= 1;

            let bit = &byte_bits[i];

            current_byte |= match bit {
                Bit::Zero => 0,
                Bit::One => 1,
            };
        }

        current_byte
    }

    pub fn len(&self) -> usize {
        self.bits.len()
    }
}
