use crate::colors::{YCbCr, RGB};
use std::{fs, time::Instant};

#[derive(Debug, Clone, Copy)]
pub struct Resolution {
    pub width: usize,
    pub height: usize,
}

impl Resolution {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
}

#[derive(Debug)]
pub struct Image {
    pub resolution: Resolution,
    pub pixels: Vec<Vec<RGB>>,
}

impl Image {
    pub fn new(resolution: Resolution, pixels: Vec<Vec<RGB>>) -> Self {
        Self { resolution, pixels }
    }
    pub fn from_bytes(resolution: Resolution, file: Vec<u8>) -> Self {
        assert!(
            file.len() == resolution.height * resolution.width * 3,
            "Tried parsing {}x{} image, but input bytes were length {}",
            resolution.width,
            resolution.height,
            file.len()
        );

        let pixels = file
            .chunks(resolution.width * 3)
            .map(|row| {
                row.chunks(3)
                    .map(|vec| RGB::new(vec[0], vec[1], vec[2]))
                    .collect()
            })
            .collect();

        Self { resolution, pixels }
    }

    pub fn crop(&mut self, new_resolution: Resolution) {
        assert!(self.resolution.height >= new_resolution.height);
        assert!(self.resolution.width >= new_resolution.width);
        self.resolution = new_resolution;
        self.pixels.truncate(new_resolution.height);
        self.pixels
            .iter_mut()
            .for_each(|row| row.truncate(new_resolution.width));
    }

    pub fn write_raw_to_file(&self, file_name: &str) {
        let bytes: Vec<u8> = self
            .pixels
            .iter()
            .flat_map(|line| line.iter().flat_map(|px| Vec::<u8>::from(px)))
            .collect();

        fs::write(file_name, bytes).unwrap();
    }
}

pub struct YCbCrImage {
    pixels: Vec<Vec<YCbCr>>,
}

pub const MACROBLOCKS_SIZE: usize = 16;
type Macroblock<'a> = Vec<&'a [YCbCr]>;
impl YCbCrImage {
    pub fn get_macroblocks<'a>(&'a self, block_size: usize) -> Vec<Vec<Macroblock<'a>>> {
        let num_rows = self.pixels.len();
        let num_cols = self.pixels[0].len();

        assert!(num_rows % block_size == 0 && num_cols % block_size == 0);

        let mut macroblocks: Vec<Vec<Macroblock>> = Vec::with_capacity(num_rows / block_size);
        for row in (0..num_rows).step_by(block_size) {
            macroblocks.push(Vec::with_capacity(num_cols / block_size));
            for col in (0..num_cols).step_by(block_size) {
                macroblocks[row / block_size].push(
                    self.pixels[row..row + block_size]
                        .iter()
                        .map(|row| &row[col..col + block_size])
                        .collect(),
                );
            }
        }

        macroblocks
    }

    pub fn get_cb_macroblocks(macroblocks: &Vec<Vec<Macroblock>>) -> Vec<Vec<Vec<Vec<u8>>>> {
        macroblocks
            .iter()
            .map(|macro_row| macro_row.iter().map(Self::get_cb_macroblock).collect())
            .collect()
    }
    fn get_cb_macroblock(macroblock: &Macroblock) -> Vec<Vec<u8>> {
        macroblock
            .iter()
            .map(|row| row.iter().map(|pixel| pixel.cb).collect())
            .collect()
    }
}
impl From<Image> for YCbCrImage {
    fn from(value: Image) -> Self {
        let mut pixels: Vec<Vec<YCbCr>> = Vec::with_capacity(value.resolution.height);
        for row in 0..value.resolution.height {
            pixels.push(Vec::with_capacity(value.resolution.width));
            for col in 0..value.resolution.width {
                let current_pixel = &value.pixels[row][col];
                pixels[row].push(YCbCr::from(current_pixel));
            }
        }
        Self { pixels }
    }
}
