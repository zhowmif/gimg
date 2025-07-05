pub struct ByteReader<'a> {
    bytes: &'a [u8],
    pub offset: usize,
}

impl<'a> ByteReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    pub fn reset(&mut self) {
        self.offset = 0;
    }

    pub fn is_finished(&self) -> bool {
        self.number_of_bytes_left() == 0
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        let byte = self.bytes.get(self.offset);
        self.offset += 1;

        byte.copied()
    }

    pub fn read_bytes(&mut self, size: usize) -> Option<&'a [u8]> {
        if self.offset + size > self.bytes.len() {
            return None;
        }

        let result = &self.bytes[self.offset..self.offset + size];
        self.offset += size;

        Some(result)
    }

    pub fn read_u32_be(&mut self) -> Option<u32> {
        let bytes = self.read_bytes(4)?;

        Some(u32::from_be_bytes(bytes.try_into().unwrap()))
    }

    pub fn skip_whitespace(&mut self) {
        if !Self::is_whitespace(self.bytes[self.offset]) {
            return;
        }

        while self.offset < self.bytes.len() && Self::is_whitespace(self.bytes[self.offset]) {
            self.offset += 1;
        }
    }

    pub fn number_of_bytes_left(&self) -> usize {
        self.bytes.len().saturating_sub(self.offset)
    }

    fn skip_line(&mut self) -> Option<()> {
        while !Self::is_newline(self.read_byte()?) {}
        self.offset -= 1;

        Some(())
    }

    fn read_until_whitespace(&mut self) -> Option<&'a [u8]> {
        //should also read until end of file
        while Self::is_whitespace(self.read_byte()?) {}
        self.offset -= 1;
        let start_index = self.offset;
        while let Some(byte) = self.read_byte() {
            if Self::is_whitespace(byte) {
                self.offset -= 1;
                break;
            }
        }

        Some(&self.bytes[start_index..self.offset])
    }

    pub fn read_ppm_symbol(&mut self) -> Option<&'a [u8]> {
        loop {
            let symbol = self.read_until_whitespace()?;

            if symbol[0] != PPM_COMMENT_START_BYTE {
                return Some(symbol);
            }

            self.skip_line();
        }
    }

    fn is_whitespace(byte: u8) -> bool {
        WHITESPACE_SYMBOLS.contains(&byte)
    }

    fn is_newline(byte: u8) -> bool {
        NEWLINE_SYMBOLS.contains(&byte)
    }
}

const WHITESPACE_SYMBOLS: [u8; 6] = [10, 32, 13, 9, 11, 12];
const NEWLINE_SYMBOLS: [u8; 2] = [13, 10];
const PPM_COMMENT_START_BYTE: u8 = 35;
