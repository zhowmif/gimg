#[derive(Debug)]
pub struct MpegtsParseError(pub String);

#[macro_export]
macro_rules! mpegts_assert {
    ($assert_value:expr, $msg:expr $(, $arg:expr)*) => {
        if !$assert_value {
            let message = format!($msg $(, $arg)*);
            return Err(MpegtsParseError(format!("mpegts parse error: {}", message)));
        }
    };
}

#[macro_export]
macro_rules! extract_bits {
    ($name:ident, $type:ty, $offset:expr, $length:expr) => {
        fn $name(buf: $type) -> $type {
            (buf << $offset) >> (<$type>::BITS - $length)
        }
    };
}


#[macro_export]
macro_rules! read_flag_bit {
    ($name:ident, $offset:expr) => {
        fn $name(buf: u8) -> bool {
            buf & (1 << (7 - $offset)) > 0
        }
    };
}
 
extract_bits!(read_33b_first, u64, 28, 3);
extract_bits!(read_33b_second, u64, 32, 15);
extract_bits!(read_33b_third, u64, 48, 15);

pub fn read_marker_bit_seperated_33_bit_uint(bytes: &[u8; 5]) -> u64 {
    let bytes_num = u64::from_be_bytes([0, 0, 0, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4]]);
    let first = read_33b_first(bytes_num);
    let second = read_33b_second(bytes_num);
    let third = read_33b_third(bytes_num);

    (first << 30) | (second << 15) | third
}

extract_bits!(read_base_first_part, u64, 18, 3);
extract_bits!(read_base_second_part, u64, 22, 15);
extract_bits!(read_base_third_part, u64, 38, 15);
extract_bits!(read_extension, u64, 54, 9);

pub fn read_33_bit_uint_with_extension(bytes: &[u8; 6]) -> u64 {
    let bytes_as_int = u64::from_be_bytes([
        0, 0, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
    ]);
    let first = read_base_first_part(bytes_as_int);
    let second = read_base_second_part(bytes_as_int);
    let third = read_base_third_part(bytes_as_int);
    let base = (first << 30) | (second << 15) | third;
    let extension = read_extension(bytes_as_int);

    base * 300 + extension
}
