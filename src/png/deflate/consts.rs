pub const MAX_UNCOMPRESSED_BLOCK_SIZE: u16 = u16::MAX;
pub const LZSS_WINDOW_SIZE: usize = u16::MAX as usize / 2;
pub const END_OF_BLOCK_MARKER_VALUE: u16 = 256;
pub const MAX_SYMBOL_CODE_LENGTH: usize = 15;
pub const MAX_CL_CODE_LENGTH: usize = 7;
pub const CL_ALPHABET: [u32; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];
pub const COMPRESSION_TEST_CHUNK_SIZE: usize = 150_000;
