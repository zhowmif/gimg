pub struct CrcCalculator {
    table: [u32; 256],
    c: u32,
}

impl CrcCalculator {
    pub fn new() -> Self {
        let mut table = [0; 256];

        for (n, item) in table.iter_mut().enumerate() {
            let mut c = n as u32;

            for _k in 0..8 {
                c = if c & 1 == 1 {
                    0xedb88320 ^ (c >> 1)
                } else {
                    c >> 1
                }
            }

            *item = c;
        }

        Self {
            table,
            c: 0xffffffff,
        }
    }

    pub fn update_crc(&mut self, bytes: &[u8]) {
        for byte in bytes {
            let index = (self.c ^ (*byte as u32)) & 0xff;
            self.c = self.table[index as usize] ^ (self.c >> 8)
        }
    }

    pub fn get_crc(&self) -> u32 {
        self.c ^ 0xffffffff
    }

    pub fn reset(&mut self) {
        self.c = 0xffffffff;
    }
}
