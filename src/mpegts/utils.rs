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
