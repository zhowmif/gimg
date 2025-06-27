pub const PNG_SIGNATURE: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
pub const CHUNK_METADATA_LENGTH: usize = 12;

pub const IHDR_CHUNK_TYPE: &[u8] = &[0x49, 0x48, 0x44, 0x52];
pub const IHDR_DATA_LENGTH: usize = 13;

pub const IDAT_CHUNK_TYPE: &[u8] = &[0x49, 0x44, 0x41, 0x54];
pub const IDAT_CHUNK_MAX_SIZE: u32 = (2 as u32).pow(31) - 1;

pub const IEND_CHUNK_TYPE: &[u8] = &[0x49, 0x45, 0x4e, 0x44];

pub const PLTE_CHUNK_TYPE: &[u8] = &[0x50, 0x4C, 0x54, 0x45];
