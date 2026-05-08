use consts::GIF_SIGNATURE;

use crate::{binary::byte_reader::ByteReader, colors::Rgba};

mod consts;

#[derive(Debug)]
pub struct GifParseError(String);

#[macro_export]
macro_rules! gif_assert {
    ($assert_value:expr, $msg:expr) => {
        if !$assert_value {
            return Err(GifParseError(format!("GIF parse error: {}", $msg)));
        }
    };
}

pub fn decode_gif(bytes: &[u8]) -> Result<Vec<Vec<Rgba>>, GifParseError> {
    let mut byte_reader = ByteReader::new(bytes);
    let signature = byte_reader.read_bytes(GIF_SIGNATURE.len()).unwrap_or(&[]);

    gif_assert!(*signature == *GIF_SIGNATURE, "GIF header not found");

    todo!()
}
