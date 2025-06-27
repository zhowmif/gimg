use std::collections::{HashMap, HashSet};

use crate::{
    colors::RGB,
    png::{chunks::Chunk, consts::PLTE_CHUNK_TYPE, crc::CrcCalculator},
};

pub struct PLTE;

impl PLTE {
    pub fn encode_palette(
        palette: &HashMap<RGB, (usize, RGB)>,
        crc_calculator: &mut CrcCalculator,
    ) -> Vec<u8> {
        let mut palette_colors: HashSet<(usize, RGB)> = HashSet::new();

        for (_c, color_value) in palette {
            palette_colors.insert(color_value.clone());
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
}
