use std::fmt::Debug;

pub struct Bits {
    bits: Vec<Bit>,
}

impl Debug for Bits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Bits {
    pub fn new(bits: Vec<Bit>) -> Bits {
        Bits { bits }
    }

    pub fn empty() -> Bits {
        Bits { bits: vec![] }
    }

    pub fn push_zero(&self) -> Bits {
        let mut new_bits = self.bits.clone();
        new_bits.push(Bit::Zero);

        Bits::new(new_bits)
    }

    pub fn push_one(&self) -> Bits {
        let mut new_bits = self.bits.clone();
        new_bits.push(Bit::One);

        Bits::new(new_bits)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result: Vec<u8> = vec![];
        let mut bit_offset = 0;
        let mut curr_byte: u8 = 0;

        for bit in self.bits.iter() {
            curr_byte = curr_byte << 1;
            if let Bit::One = *bit {
                curr_byte = curr_byte | 1
            }
            bit_offset += 1;

            if bit_offset == 8 {
                result.push(curr_byte);
                bit_offset = 0;
                curr_byte = 0;
            }
        }

        if bit_offset != 0 {
            result.push(curr_byte);
        }

        result
    }

    pub fn print_bin(&self) {
        for byte in self.to_bytes() {
            print!("{:08b}", byte);
        }
        println!();
    }
}

impl ToString for Bits {
    fn to_string(&self) -> String {
        self.bits.iter().map(|b| char::from(b)).collect()
    }
}

impl From<&Bit> for char {
    fn from(bit: &Bit) -> Self {
        match bit {
            Bit::Zero => '0',
            Bit::One => '1',
        }
    }
}

#[derive(Clone, Debug)]
pub enum Bit {
    Zero,
    One,
}

impl From<char> for Bit {
    fn from(chr: char) -> Self {
        match chr {
            '0' => Self::Zero,
            '1' => Self::One,
            _ => panic!("Tried converting invalid char to bits"),
        }
    }
}
