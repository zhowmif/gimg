use binary_utils::read_bytes;
use chunks::{
    idat::IDAT,
    iend::IEND,
    ihdr::{self},
    plte::PLTE,
    Chunk,
};
pub use color_type::ColorType;
use consts::{IDAT_CHUNK_MAX_SIZE, IDAT_CHUNK_TYPE, IEND_CHUNK_TYPE, PNG_SIGNATURE};
use crc::CrcCalculator;
use deflate::{compress_scanlines, uncompress_scanlines};
use filter::{filter_scanlines, remove_scanlines_filter};
use ihdr::IHDR;
use palette::{create_pallete_from_colors_median_cut, get_unique_colors};
use serialization::scanline_to_pixels;

use crate::colors::RGBA;

mod adler32;
mod binary_utils;
mod chunks;
mod color_type;
mod consts;
mod crc;
pub mod deflate;
mod filter;
pub mod palette;
mod serialization;

#[derive(Debug)]
pub struct PngParseError(String);

#[macro_export]
macro_rules! png_assert {
    ($assert_value:expr, $msg:expr) => {
        if !$assert_value {
            return Err(PngParseError(format!("png parse error: {}", $msg)));
        }
    };
}

pub fn decode_png(bytes: &[u8]) -> Result<Vec<Vec<RGBA>>, PngParseError> {
    let mut offset: usize = 0;
    let siganture = read_bytes(&mut offset, bytes, PNG_SIGNATURE.len());

    if *siganture != *PNG_SIGNATURE {
        return Err(PngParseError(
            "File does not appear to be a png file (signature missing)".to_string(),
        ));
    }
    let ihdr_chunk = IHDR::from_chunk(Chunk::from_bytes(bytes, &mut offset)?)?;
    ihdr_chunk.check_compatibility()?;
    let mut compressed_data: Vec<u8> = Vec::new();

    loop {
        let chunk = Chunk::from_bytes(bytes, &mut offset)?;

        match chunk.chunk_type {
            IDAT_CHUNK_TYPE => compressed_data.extend_from_slice(chunk.chunk_data),
            IEND_CHUNK_TYPE => {
                break;
            }
            _ => {
                return Err(PngParseError(format!(
                    "Unrecognized chunk type: {:?}",
                    chunk.chunk_type
                )))
            }
        }
    }

    let bbp = ihdr_chunk.get_bits_per_pixel();
    let filtered_scanlines = uncompress_scanlines(
        &compressed_data,
        ihdr_chunk.height as usize,
        ihdr_chunk.width as usize,
        ihdr_chunk.get_bits_per_pixel(),
    )?;
    let scanlines = remove_scanlines_filter(&filtered_scanlines, bbp)?;
    let pixels = ihdr_chunk.color_type.scanline_to_pixels(
        &scanlines,
        ihdr_chunk.bit_depth,
        ihdr_chunk.width as usize,
    );

    Ok(pixels)
}

pub fn encode_png(
    pixels: Vec<Vec<RGBA>>,
    color_type: Option<ColorType>,
    bit_depth: Option<u8>,
) -> Vec<u8> {
    let color_type = color_type.unwrap_or(ColorType::TrueColorAlpha);
    let bit_depth = bit_depth.unwrap_or(8);

    let palette = match color_type {
        ColorType::IndexedColor => {
            let unique_colors = get_unique_colors(&pixels[..]);
            let palette = create_pallete_from_colors_median_cut(&unique_colors, bit_depth as usize);

            Some(palette)
        }
        _ => None,
    };

    let mut crc = CrcCalculator::new();
    let ihdr = IHDR::new(
        pixels[0].len() as u32,
        pixels.len() as u32,
        color_type,
        bit_depth,
    );
    ihdr.check_bit_depth_validity().unwrap();

    let scanlines = color_type.create_scanlines(&pixels, ihdr.bit_depth, &palette);
    let filtered_scanlines = filter_scanlines(&scanlines, ihdr.get_bits_per_pixel());
    let compressed_data = compress_scanlines(&filtered_scanlines);

    let mut encoded_png: Vec<u8> = Vec::with_capacity(compressed_data.len() + 1000);
    encoded_png.extend_from_slice(PNG_SIGNATURE);
    encoded_png.extend_from_slice(&ihdr.to_bytes(&mut crc));

    if let Some(palette) = palette {
        encoded_png.extend_from_slice(&PLTE::encode_palette(&palette, &mut crc));
    }

    compressed_data
        .chunks(IDAT_CHUNK_MAX_SIZE as usize)
        .for_each(|chunk_data| {
            let chunk = IDAT::encode_bytes(chunk_data, &mut crc);

            encoded_png.extend_from_slice(&chunk);
        });

    encoded_png.extend_from_slice(&IEND::to_bytes(&mut crc));

    encoded_png
}
