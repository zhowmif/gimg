use binary_utils::read_bytes;
use chunks::{idat::IDAT, iend::IEND, ihdr, Chunk};
use consts::{IDAT_CHUNK_MAX_SIZE, PNG_SIGNATURE};
use crc::CrcCalculator;
use deflate::compress_scanlines;
use filter::filter_scanlines;
use ihdr::IHDR;
use serialization::create_scanlines;

use crate::colors::{RGB, RGBA};

mod binary_utils;
mod chunks;
mod consts;
pub mod crc;
pub mod deflate;
mod filter;
mod serialization;

#[derive(Debug)]
pub struct PngParseError(String);

pub fn decode_png(bytes: &[u8]) -> Result<Vec<RGB>, PngParseError> {
    let mut offset: usize = 0;
    let siganture = read_bytes(&mut offset, bytes, PNG_SIGNATURE.len());

    if *siganture != *PNG_SIGNATURE {
        return Err(PngParseError(
            "File does not appear to be a png file (signature missing)".to_string(),
        ));
    }
    let first_chunk = IHDR::from_chunk(Chunk::from_bytes(bytes, &mut offset)?)?;
    println!("First chunk {:?}", first_chunk);
    first_chunk.check_compatibility()?;

    todo!()
}

pub fn encode_png(pixels: Vec<Vec<RGBA>>) -> Vec<u8> {
    let mut crc = CrcCalculator::new();
    let ihdr = IHDR::new(pixels[0].len() as u32, pixels.len() as u32);
    let scanlines = create_scanlines(&pixels);
    let filtered_scanlines = filter_scanlines(&scanlines);
    let compressed_data = compress_scanlines(&filtered_scanlines);

    let mut encoded_png: Vec<u8> = Vec::with_capacity(compressed_data.len() + 1000);
    encoded_png.extend_from_slice(PNG_SIGNATURE);
    encoded_png.extend_from_slice(&ihdr.to_bytes(&mut crc));

    compressed_data
        .chunks(IDAT_CHUNK_MAX_SIZE as usize)
        .for_each(|chunk_data| {
            let chunk = IDAT::encode_bytes(chunk_data, &mut crc);

            encoded_png.extend_from_slice(&chunk);
        });

    encoded_png.extend_from_slice(&IEND::to_bytes(&mut crc));

    encoded_png
}
