use crate::colors::{YCbCr, RGB};
use std::fs;

#[derive(Debug)]
pub struct Resolution {
    width: u32,
    height: u32,
}

impl Resolution {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug)]
pub struct Image {
    resolution: Resolution,
    pixels: Vec<Vec<RGB>>,
}

impl Image {
    pub fn from_raw_file(resolution: Resolution, file: Vec<u8>) -> Self {
        let mut all_pixels: Vec<RGB> = file
            .chunks(3)
            .map(|chunk| RGB::new(chunk[0], chunk[1], chunk[2]))
            .collect();
        let mut pixels: Vec<Vec<RGB>> = vec![];

        for _line in 0..resolution.height {
            if all_pixels.len() < resolution.width as usize {
                panic!("input file is too short for image resolution");
            }

            let chunk = all_pixels.split_off(resolution.width as usize);
            pixels.push(all_pixels);

            all_pixels = chunk;
        }

        if !all_pixels.is_empty() {
            panic!("input file is too long for image resolution")
        }

        Self { resolution, pixels }
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

        println!("writing!");
        fs::write(file_name, bytes).unwrap();
    }
}
