#[derive(Debug, PartialEq, Eq)]
pub enum PesStreamId {
    Audio(u8),
    Video(u8),
    Padding,
    Private1,
    Unknown(u8),
}

impl From<u8> for PesStreamId {
    fn from(byte: u8) -> PesStreamId {
        match byte {
            0xC0..=0xDF => PesStreamId::Audio(byte & 0x1F),
            0xE0..=0xEF => PesStreamId::Video(byte & 0x0F),
            0xBE => PesStreamId::Padding,
            0xBD => PesStreamId::Private1,
            _ => PesStreamId::Unknown(byte),
        }
    }
}
