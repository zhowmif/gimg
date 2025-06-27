use super::PngParseError;

#[derive(Debug)]
enum AdaptiveFilterType {
    None,
    Sub,
    Up,
    Average,
    Paeth,
}

impl AdaptiveFilterType {
    fn apply_filter(&self, x: u8, a: u8, b: u8, c: u8) -> u8 {
        match self {
            AdaptiveFilterType::None => x,
            AdaptiveFilterType::Sub => x.overflowing_sub(a).0,
            AdaptiveFilterType::Up => x.overflowing_sub(b).0,
            AdaptiveFilterType::Average => {
                x.overflowing_sub(((a as f32 + b as f32) / 2.).floor() as u8)
                    .0
            }
            AdaptiveFilterType::Paeth => x.overflowing_sub(paeth_predictor(a, b, c)).0,
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

const FILTERS: [AdaptiveFilterType; 5] = [
    AdaptiveFilterType::None,
    AdaptiveFilterType::Sub,
    AdaptiveFilterType::Up,
    AdaptiveFilterType::Average,
    AdaptiveFilterType::Paeth,
];

type FilteredScenaline = (AdaptiveFilterType, Vec<u8>);

pub fn filter_scanlines(scanlines: &Vec<Vec<u8>>, bbp: usize) -> Vec<Vec<u8>> {
    let other_byte_offsets = if bbp <= 8 { 1 } else { (bbp >> 3) as i16 };
    let mut filtered_scanelines: Vec<Vec<u8>> = Vec::with_capacity(scanlines.len());

    for row in 0..scanlines.len() {
        let mut filter_results: Vec<FilteredScenaline> = Vec::with_capacity(FILTERS.len());

        for filter in FILTERS {
            let mut current_filter_result: Vec<u8> = Vec::with_capacity(scanlines[row].len());

            for col in 0..scanlines[row].len() {
                let x = scanlines[row][col];
                let (row, col) = (row as i16, col as i16);
                let a = get_byte(&scanlines, row, col - other_byte_offsets);
                let b = get_byte(&scanlines, row - 1, col);
                let c = get_byte(&scanlines, row - 1, col - other_byte_offsets);

                current_filter_result.push(filter.apply_filter(x, a, b, c));
            }

            filter_results.push((filter, current_filter_result));
        }

        let (filter, mut scanline) = filter_results
            .into_iter()
            .min_by_key(|filtered_row| filtered_row.1.iter().map(|x| *x as u32).sum::<u32>())
            .unwrap();
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
