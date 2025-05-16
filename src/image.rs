use crate::colors::{YCbCr, RGB};
use std::{fs, time::Instant};

#[derive(Debug, Clone, Copy)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug)]
pub struct Image {
    pub resolution: Resolution,
    pub pixels: Vec<Vec<RGB>>,
}

impl Image {
    pub fn from_raw_file(resolution: Resolution, file: Vec<u8>) -> Self {
        assert!(
            file.len() as u32 == resolution.height * resolution.width * 3,
            "Tried parsing {}x{} image, but input bytes were length {}",
            resolution.width,
            resolution.height,
            file.len()
        );

        let pixels = file
            .chunks(resolution.width as usize * 3)
            .map(|row| {
                row.chunks(3)
                    .map(|vec| RGB::new(vec[0], vec[1], vec[2]))
                    .collect()
            })
            .collect();

        Self {
            resolution,
            pixels,
        }
    }

    pub fn crop(&mut self, new_resolution: Resolution) {
        assert!(self.resolution.height >= new_resolution.height);
        assert!(self.resolution.width >= new_resolution.width);
        self.resolution = new_resolution;
        self.pixels.truncate(new_resolution.height as usize);
        self.pixels
            .iter_mut()
            .for_each(|row| row.truncate(new_resolution.width as usize));
    }

    pub fn draw_red_circle(&mut self) {
        for row in 0..self.resolution.height {
            for col in 0..self.resolution.width {
                let dy = row.abs_diff(375);
                let dx = col.abs_diff(562);
                let sqr_distance = (dy * dy) + (dx * dx);

                if sqr_distance < 50000 {
                    self.pixels[row as usize][col as usize] = RGB::new(200, 0, 0);
                }
            }
        }
    }

    pub fn convert_to_grayscale(&mut self) {
        for row in 0..self.resolution.height {
            for col in 0..self.resolution.width {
                let current_pixel = &self.pixels[row as usize][col as usize];
                let luma = YCbCr::from(current_pixel).y;

                self.pixels[row as usize][col as usize] = RGB::new(luma, luma, luma);
            }
        }
    }

    pub fn only_keep_blue_chroma(&mut self) {
        for row in 0..self.resolution.height {
            for col in 0..self.resolution.width {
                let current_pixel = &self.pixels[row as usize][col as usize];
                let ycbcr = YCbCr::from(current_pixel);
                let back_to_rgb = RGB::from(YCbCr::new(127, ycbcr.cb, 127));

                self.pixels[row as usize][col as usize] = back_to_rgb;
            }
        }
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
        let mut pixels: Vec<Vec<YCbCr>> = Vec::with_capacity(value.resolution.height as usize);
        for row in 0..value.resolution.height as usize {
            pixels.push(Vec::with_capacity(value.resolution.width as usize));
            for col in 0..value.resolution.width as usize {
                let current_pixel = &value.pixels[row as usize][col as usize];
                pixels[row].push(YCbCr::from(current_pixel));
            }
        }
        Self { pixels }
    }
}
