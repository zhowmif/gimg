use std::{
    collections::{HashMap, HashSet},
    u8,
};

use crate::{
    colors::{RGB, RGBA},
    png::{chunks::Chunk, consts::PLTE_CHUNK_TYPE, crc::CrcCalculator, PngParseError},
    png_assert,
};

pub struct PLTE;

impl PLTE {
    pub fn encode_palette(
        palette: &HashMap<RGBA, (usize, RGBA)>,
        crc_calculator: &mut CrcCalculator,
    ) -> Vec<u8> {
        let mut palette_colors: HashSet<(usize, RGB)> = HashSet::new();

        for (_c, (idx, color)) in palette {
            palette_colors.insert((*idx, color.into()));
        }

        let mut palette_colors: Vec<(usize, RGB)> = palette_colors.into_iter().collect();
        palette_colors.sort_by_key(|(idx, _c)| *idx);
        let color_bytes: Vec<u8> = palette_colors
            .iter()
            .map(|(_idx, color)| vec![color.r, color.g, color.b])
            .flatten()
            .collect();
        let chunk = Chunk::new(PLTE_CHUNK_TYPE, &color_bytes, crc_calculator);

        chunk.to_bytes()
    }

    pub fn decode_palette(bytes: &[u8]) -> Result<Vec<RGBA>, PngParseError> {
        png_assert!(
            bytes.len() % 3 == 0 && bytes.len() <= 256 * 3,
            format!("invalid PLTE chunk size - {}, PLTE size must a multiple of 3 and contain no more than 256 colors", bytes.len())
        );
        let mut palette = Vec::with_capacity(bytes.len() / 3);

        for pixel_bytes in bytes.chunks_exact(3) {
            let rgb = RGBA::new(pixel_bytes[0], pixel_bytes[1], pixel_bytes[2], u8::MAX);

            palette.push(rgb);
        }

        Ok(palette)
    }
}
