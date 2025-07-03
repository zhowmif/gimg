use std::{
    iter::{self, repeat_n},
    time::Instant,
};

use crate::simd_utils::{paeth_predictor_simd, png_average_simd, subtract_simd};

use super::{deflate::DeflateEncoder, CompressionLevel, PngParseError};

#[derive(Debug, Clone)]
enum AdaptiveFilterType {
    None,
    Sub,
    Up,
    Average,
    Paeth,
}

impl AdaptiveFilterType {
    fn apply_filter_simd(&self, x: &[u8], a: &[u8], b: &[u8], c: &[u8]) -> Vec<u8> {
        match self {
            AdaptiveFilterType::None => x.to_vec(),
            AdaptiveFilterType::Sub => subtract_simd(x, a),
            AdaptiveFilterType::Up => subtract_simd(x, b),
            AdaptiveFilterType::Average => png_average_simd(x, a, b),
            AdaptiveFilterType::Paeth => paeth_predictor_simd(x, a, b, c),
        }
    }

    fn revert_filter(&self, x: u8, a: u8, b: u8, c: u8) -> u8 {
        match self {
            AdaptiveFilterType::None => x,
            AdaptiveFilterType::Sub => x.overflowing_add(a).0,
            AdaptiveFilterType::Up => x.overflowing_add(b).0,
            AdaptiveFilterType::Average => {
                x.overflowing_add(((a as f32 + b as f32) / 2.).floor() as u8)
                    .0
            }
            AdaptiveFilterType::Paeth => x.overflowing_add(paeth_predictor(a, b, c)).0,
        }
    }

    fn to_byte(&self) -> u8 {
        match self {
            AdaptiveFilterType::None => 0,
            AdaptiveFilterType::Sub => 1,
            AdaptiveFilterType::Up => 2,
            AdaptiveFilterType::Average => 3,
            AdaptiveFilterType::Paeth => 4,
        }
    }

    fn from_byte(byte: u8) -> Result<Self, PngParseError> {
        Ok(match byte {
            0 => AdaptiveFilterType::None,
            1 => AdaptiveFilterType::Sub,
            2 => AdaptiveFilterType::Up,
            3 => AdaptiveFilterType::Average,
            4 => AdaptiveFilterType::Paeth,
            f => {
                return Err(PngParseError(format!(
                    "Unrecognized adaptive filter type {}",
                    f
                )));
            }
        })
    }
}

fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let (a, b, c) = (a as i16, b as i16, c as i16);
    let p = a + b - c;
    let pa = p.abs_diff(a);
    let pb = p.abs_diff(b);
    let pc = p.abs_diff(c);

    if pa <= pb && pa <= pc {
        a as u8
    } else if pb <= pc {
        b as u8
    } else {
        c as u8
    }
}

const ALL_FILTERS: [AdaptiveFilterType; 5] = [
    AdaptiveFilterType::None,
    AdaptiveFilterType::Sub,
    AdaptiveFilterType::Up,
    AdaptiveFilterType::Average,
    AdaptiveFilterType::Paeth,
];

type FilteredScenaline = (AdaptiveFilterType, Vec<u8>);

pub fn filter_scanlines(
    scanlines: &Vec<Vec<u8>>,
    bbp: usize,
    compression_level: CompressionLevel,
) -> Vec<Vec<u8>> {
    let filters_to_test = match compression_level {
        CompressionLevel::None => vec![AdaptiveFilterType::None],
        CompressionLevel::Best => ALL_FILTERS.to_vec(),
        CompressionLevel::Fast => vec![AdaptiveFilterType::Paeth],
    };
    let other_byte_offsets = if bbp <= 8 { 1 } else { (bbp >> 3) as i16 };
    let mut filtered_scanelines: Vec<Vec<u8>> = Vec::with_capacity(scanlines.len());
    let empty_row: Vec<u8> = repeat_n(0u8, scanlines[0].len()).collect();
    let mut b = &empty_row[..];
    let mut c: Vec<u8> = empty_row.clone();

    for row in 0..scanlines.len() {
        let mut filter_results: Vec<FilteredScenaline> = Vec::with_capacity(ALL_FILTERS.len());
        let x = &scanlines[row][..];
        let mut a: Vec<u8> = iter::repeat_n(0u8, other_byte_offsets as usize).collect();

        for filter in filters_to_test.iter() {
            a.extend_from_slice(&x[..x.len() - other_byte_offsets as usize]);

            let current_filter_result = filter.apply_filter_simd(x, &a.clone(), b, &c.clone());

            filter_results.push((filter.clone(), current_filter_result));
        }

        b = x;
        c = a;
        let (filter, mut scanline) = if filter_results.len() == 1 {
            filter_results.into_iter().next().unwrap()
        } else {
            filter_results
                .into_iter()
                .min_by_key(|filtered_row| {
                    let mut encoder = DeflateEncoder::new(CompressionLevel::Fast);
                    encoder.write_bytes(&filtered_row.1);
                    encoder.finish().len()
                })
                .unwrap()
        };

        scanline.insert(0, filter.to_byte());
        filtered_scanelines.push(scanline);
    }

    filtered_scanelines
}

fn get_byte(scanlines: &Vec<Vec<u8>>, row: i16, col: i16) -> u8 {
    if row < 0 || col < 0 {
        return 0;
    }

    scanlines
        .get(row as usize)
        .map(|scanline| scanline.get(col as usize).map(|val| val.clone()))
        .flatten()
        .unwrap_or(0)
}

pub fn remove_scanlines_filter(
    scanlines: &Vec<Vec<u8>>,
    bbp: usize,
) -> Result<Vec<Vec<u8>>, PngParseError> {
    let mut unfiltered_scanlines = Vec::with_capacity(scanlines.len());
    let other_byte_offsets = if bbp <= 8 { 1 } else { (bbp >> 3) as i16 };

    for row in 0..scanlines.len() {
        let filter_type = AdaptiveFilterType::from_byte(scanlines[row][0])?;
        unfiltered_scanlines.push(Vec::with_capacity(scanlines[row].len()));

        for col in 1..scanlines[row].len() {
            let x = scanlines[row][col];
            //subtract 1 from col because there is no filter type byte in unfiltered_scanlines
            let (row, col) = (row as i16, col as i16 - 1);

            let a = get_byte(&unfiltered_scanlines, row, col - other_byte_offsets);
            let b = get_byte(&unfiltered_scanlines, row - 1, col);
            let c = get_byte(&unfiltered_scanlines, row - 1, col - other_byte_offsets);

            let val = filter_type.revert_filter(x, a, b, c);

            unfiltered_scanlines[row as usize].push(val);
        }
    }

    Ok(unfiltered_scanlines)
}
