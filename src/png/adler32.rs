const BASE: u32 = 65521;

pub struct Adler32Calculator {
    value: u32,
}

impl Adler32Calculator {
    pub fn new() -> Self {
        Self { value: 1 }
    }

    pub fn update_adler32(&mut self, bytes: &[u8]) {
        let mut s1: u32 = self.value & 0xffff;
        let mut s2: u32 = (self.value >> 16) & 0xffff;

        for byte in bytes {
            s1 = (s1 + *byte as u32) % BASE;
            s2 = (s2 + s1) % BASE;
        }

        self.value = (s2 << 16) + s1;
    }

    pub fn get_adler32(&self) -> u32 {
        self.value
    }

    pub fn reset(&mut self) {
        self.value = 1;
    }
}
