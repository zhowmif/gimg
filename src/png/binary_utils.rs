pub fn read_u32(offset: &mut usize, bytes: &[u8]) -> u32 {
    u32::from_be_bytes(read_bytes(offset, bytes, 4).try_into().unwrap())
}

pub fn read_byte(offset: &mut usize, bytes: &[u8]) -> u8 {
    let byte = bytes[*offset as usize];
    *offset += 1;

    byte
}

pub fn read_bytes<'a>(offset: &mut usize, bytes: &'a [u8], size: usize) -> &'a [u8] {
    let result = &bytes[*offset..*offset + size];
    *offset += size;

    result
}
