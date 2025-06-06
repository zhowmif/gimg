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
            Self::None => x,
            Self::Sub => x.overflowing_sub(a).0,
            Self::Up => x.overflowing_sub(b).0,
            Self::Average => {
                x.overflowing_sub(((a as f32 + b as f32) / 2.).floor() as u8)
                    .0
            }
            Self::Paeth => x.overflowing_sub(paeth_predictor(a, b, c)).0,
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

pub fn filter_scanlines(scanlines: &Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let mut filtered_scanelines: Vec<Vec<u8>> = Vec::with_capacity(scanlines.len());

    for row in 0..scanlines.len() {
        let mut filter_results: Vec<FilteredScenaline> = Vec::with_capacity(FILTERS.len());

        for filter in FILTERS {
            let mut current_filter_result: Vec<u8> = Vec::with_capacity(scanlines[row].len());

            for col in 0..scanlines[row].len() {
                let x = scanlines[row][col];
                let (row, col) = (row as i16, col as i16);
                let a = get_byte(&scanlines, row, col - 1);
                let b = get_byte(&scanlines, row - 1, col);
                let c = get_byte(&scanlines, row - 1, col - 1);

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
    scanlines
        .get(row.max(0) as usize)
        .map(|scanline| scanline.get(col.max(0) as usize).map(|val| val.clone()))
        .flatten()
        .unwrap_or(0)
}
