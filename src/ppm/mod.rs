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
const LINE_FEED: u8 = 10;

pub fn decode_ppm(bytes: &[u8]) -> Result<Vec<Vec<RGB>>, PpmParseError> {
    let mut reader = ByteReader::new(bytes);
    let signature = ppm_read_bytes!(reader.read_ppm_symbol(), "expected magic number");

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

    let bytes_per_pixel = if max_color_value > u8::MAX.into() {
        2
    } else {
        1
    };
    let expected_pixel_bytes_size: usize = (width * height * 3) as usize * bytes_per_pixel;
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
        .chunks_exact(width as usize * 3 * bytes_per_pixel)
        .map(|row| {
            row.chunks_exact(3 * bytes_per_pixel)
                .map(|rgb_bytes| {
                    let normalization_factor = 255. / max_color_value as f32;
                    if bytes_per_pixel == 1 {
                        rgb24_from_bytes(rgb_bytes, normalization_factor)
                    } else {
                        rgb48_from_bytes(rgb_bytes, normalization_factor)
                    }
                })
                .collect()
        })
        .collect();
    debug_assert_eq!(pixels.len(), height as usize);

    Ok(pixels)
}

fn rgb24_from_bytes(rgb_bytes: &[u8], normalization_factor: f32) -> RGB {
    RGB::new(
        ((rgb_bytes[0] as f32) * normalization_factor) as u8,
        ((rgb_bytes[1] as f32) * normalization_factor) as u8,
        ((rgb_bytes[2] as f32) * normalization_factor) as u8,
    )
}

fn rgb48_from_bytes(rgb_bytes: &[u8], normalization_factor: f32) -> RGB {
    RGB::new(
        ((((rgb_bytes[0] as u16) << 8) + rgb_bytes[1] as u16) as f32 * normalization_factor) as u8,
        ((((rgb_bytes[2] as u16) << 8) + rgb_bytes[3] as u16) as f32 * normalization_factor) as u8,
        ((((rgb_bytes[4] as u16) << 8) + rgb_bytes[5] as u16) as f32 * normalization_factor) as u8,
    )
}

fn read_ascii_integer(reader: &mut ByteReader, field_name: &str) -> Result<u32, PpmParseError> {
    let bytes = ppm_read_bytes!(reader.read_ppm_symbol(), format!("expected {}", field_name));
    let number = String::from_utf8(bytes.to_vec())
        .map_err(|_e| PpmParseError(format!("{} is not valid utf8", field_name)))?
        .parse::<u32>()
        .map_err(|_e| PpmParseError(format!("{} is not a valid unsigned integer", field_name)))?;

    Ok(number)
}

pub fn encode_ppm(pixels: &[Vec<RGB>]) -> Vec<u8> {
    let mut result = Vec::with_capacity(20 + pixels.len() * pixels[0].len());

    result.extend_from_slice(PPM_SIGNATURE);
    result.push(LINE_FEED);

    let width = pixels[0].len();
    result.extend_from_slice(width.to_string().as_bytes());
    result.push(LINE_FEED);

    let height = pixels.len();
    result.extend_from_slice(height.to_string().as_bytes());
    result.push(LINE_FEED);

    let maxval = 255;
    result.extend_from_slice(maxval.to_string().as_bytes());
    result.push(LINE_FEED);

    for row in pixels {
        for pixel in row {
            result.push(pixel.r);
            result.push(pixel.g);
            result.push(pixel.b);
        }
    }

    result
}
