use std::{fmt::format, num};

use crate::{binary::byte_reader::ByteReader, colors::RGB};

#[derive(Debug)]
pub struct PpmParseError(String);

macro_rules! ppm_read_bytes {
    ($read_value:expr, $msg:expr) => {
        match $read_value {
            Some(value) => value,
            None => {
                return Err(PpmParseError(format!(
                    "PPM stream ended unexpectedly: {}",
                    $msg
                )));
            }
        }
    };
}

const PPM_SIGNATURE: &[u8] = &[80, 54];

pub fn decode_ppm(bytes: &[u8]) -> Result<Vec<Vec<RGB>>, PpmParseError> {
    let mut reader = ByteReader::new(bytes);
    let signature = ppm_read_bytes!(reader.read_until_whitespace(), "expected magic number");

    if signature != PPM_SIGNATURE {
        return Err(PpmParseError(
            "File does not look like a PPM file (magic number missing)".to_string(),
        ));
    }

    let width = read_ascii_integer(&mut reader, "width")?;
    let height = read_ascii_integer(&mut reader, "height")?;
    let max_color_value = read_ascii_integer(&mut reader, "maxval")?;

    if max_color_value > u16::MAX.into() {
        return Err(PpmParseError(format!(
            "Invalid Maxval, expected value between 0 and {}, found {}",
            u16::MAX,
            max_color_value
        )));
    }

    if max_color_value > 255 {
        panic!("Max val greater than 255 not supported yet");
    }

    let expected_pixel_bytes_size: usize = (width * height * 3) as usize;
    reader.skip_whitespace();
    let mut pixel_bytes = ppm_read_bytes!(
        reader.read_bytes(expected_pixel_bytes_size),
        format!(
            "Expected {} pixel value bytes for {}x{} file, only found {}",
            expected_pixel_bytes_size,
            width,
            height,
            reader.number_of_bytes_left()
        )
    );

    if pixel_bytes.len() > expected_pixel_bytes_size {
        pixel_bytes = &pixel_bytes[..expected_pixel_bytes_size];
    }

    let pixels: Vec<_> = pixel_bytes
        .chunks_exact(width as usize * 3)
        .map(|row| {
            row.chunks_exact(3)
                .map(|rgb_bytes| RGB::new(rgb_bytes[0], rgb_bytes[1], rgb_bytes[2]))
                .collect()
        })
        .collect();
    debug_assert_eq!(pixels.len(), height as usize);

    Ok(pixels)
}

fn read_ascii_integer(reader: &mut ByteReader, field_name: &str) -> Result<u32, PpmParseError> {
    let bytes = ppm_read_bytes!(
        reader.read_until_whitespace(),
        format!("expected {}", field_name)
    );
    let number = String::from_utf8(bytes.to_vec())
        .map_err(|_e| PpmParseError(format!("{} is not valid utf8", field_name)))?
        .parse::<u32>()
        .map_err(|_e| PpmParseError(format!("{} is not a valid unsigned integer", field_name)))?;

    Ok(number)
}
