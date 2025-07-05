use binary_utils::read_bytes;
use chunks::{
    idat::IDAT,
    iend::IEND,
    ihdr::{self},
    plte::PLTE,
    Chunk,
};
pub use color_type::ColorType;
pub use config::{CompressionLevel, PartialPngConfig, PngConfig};
use consts::{
    IDAT_CHUNK_MAX_SIZE, IDAT_CHUNK_TYPE, IEND_CHUNK_TYPE, PLTE_CHUNK_TYPE, PNG_SIGNATURE,
};
use crc::CrcCalculator;
use deflate::{compress_scanlines, zlib::decode_zlib};
use filter::{filter_scanlines, remove_scanlines_filter};
use ihdr::IHDR;
pub use interlace::InterlaceMethod;
use palette::{create_pallete_from_colors_median_cut, get_unique_colors};

use crate::colors::RGBA;

mod adler32;
mod binary_utils;
mod chunks;
mod color_type;
mod config;
mod consts;
mod crc;
pub mod deflate;
mod filter;
mod interlace;
mod palette;

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
    let mut palette: Option<Vec<RGBA>> = None;
    let mut compressed_data: Vec<u8> = Vec::new();

    loop {
        let chunk = Chunk::from_bytes(bytes, &mut offset)?;

        match chunk.chunk_type {
            IDAT_CHUNK_TYPE => compressed_data.extend_from_slice(chunk.chunk_data),
            IEND_CHUNK_TYPE => {
                break;
            }
            PLTE_CHUNK_TYPE => match palette {
                Some(_) => {
                    return Err(PngParseError(
                        "PLTE chunks appears more than once".to_string(),
                    ))
                }
                None => palette = Some(PLTE::decode_palette(chunk.chunk_data)?),
            },
            chunk_type => {
                if chunk_type[0] & 32 == 0 {
                    return Err(PngParseError(format!(
                        "Unrecognized critical chunk: {:?}",
                        chunk.chunk_type
                    )));
                }
            }
        }
    }

    png_assert!(
        !(matches!(ihdr_chunk.color_type, ColorType::IndexedColor) && palette.is_none()),
        "No PLTE chunk for indexed color".to_string()
    );

    let bbp = ihdr_chunk.get_bits_per_pixel();
    let uncompressed_data = match decode_zlib(&compressed_data) {
        Ok(data) => data,
        Err(deflate_error) => {
            return Err(PngParseError(deflate_error.0));
        }
    };
    let reduced_images_scanlines = ihdr_chunk.interlace_method.reconstruct_filtered_scanlines(
        &uncompressed_data,
        ihdr_chunk.height as usize,
        ihdr_chunk.width as usize,
        ihdr_chunk.get_bits_per_pixel(),
    )?;

    let reduced_images = reduced_images_scanlines
        .into_iter()
        .map(|reduced_image_scanlines| {
            let scanlines = remove_scanlines_filter(&reduced_image_scanlines, bbp)?;
            let reduced_image_pixels = ihdr_chunk.color_type.scanline_to_pixels(
                &scanlines,
                ihdr_chunk.bit_depth,
                ihdr_chunk.width as usize,
                &palette,
            )?;

            Ok(reduced_image_pixels)
        })
        .collect::<Result<Vec<Vec<Vec<RGBA>>>, PngParseError>>()?;
    let image = ihdr_chunk.interlace_method.deinterlace_image(
        reduced_images,
        ihdr_chunk.height as usize,
        ihdr_chunk.width as usize,
    );

    Ok(image)
}

pub fn encode_png(pixels: Vec<Vec<RGBA>>, partial_config: PartialPngConfig) -> Vec<u8> {
    let unique_colors = get_unique_colors(&pixels[..]);
    let config = PngConfig::create_from_partial(partial_config, &unique_colors[..]);
    let palette = match config.color_type {
        ColorType::IndexedColor => {
            let palette =
                create_pallete_from_colors_median_cut(&unique_colors, config.bit_depth as usize);

            Some(palette)
        }
        _ => None,
    };

    let mut crc = CrcCalculator::new();
    let ihdr = IHDR::new(
        pixels[0].len() as u32,
        pixels.len() as u32,
        config.color_type,
        config.bit_depth,
        config.interlace_method,
    );
    let mut encoded_png: Vec<u8> = Vec::new();
    encoded_png.extend_from_slice(PNG_SIGNATURE);
    encoded_png.extend_from_slice(&ihdr.to_bytes(&mut crc));

    let reduced_images = config.interlace_method.perform_pass_extraction(pixels);

    let mut all_filtered_scanlines: Vec<Vec<u8>> = Vec::new();

    for reduced_image in reduced_images.iter() {
        let scanlines = config
            .color_type
            .create_scanlines(reduced_image, ihdr.bit_depth, &palette);
        let filtered_scanlines = filter_scanlines(
            &scanlines,
            ihdr.get_bits_per_pixel(),
            config.compression_level,
        );
        all_filtered_scanlines.extend_from_slice(&filtered_scanlines);
    }

    let compressed_data = compress_scanlines(&all_filtered_scanlines, config.compression_level);

    if let Some(ref palette) = palette {
        encoded_png.extend_from_slice(&PLTE::encode_palette(palette, &mut crc));
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
