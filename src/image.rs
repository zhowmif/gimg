use crate::colors::YCbCr;

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

#[derive(Debug, Clone)]
pub struct Image {
    pub resolution: Resolution,
    pub pixels: Vec<Vec<YCbCr>>,
}

impl Image {
    pub fn new(resolution: Resolution, pixels: Vec<Vec<YCbCr>>) -> Self {
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
                    .map(|vec| YCbCr::new(vec[0], vec[1], vec[2]))
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

    pub fn get_macroblocks(&self, block_size: usize) -> Vec<Vec<Vec<Vec<YCbCr>>>> {
        let num_rows = self.pixels.len();
        let num_cols = self.pixels[0].len();

        assert!(num_rows % block_size == 0 && num_cols % block_size == 0);

        let mut macroblocks: Vec<Vec<Vec<Vec<YCbCr>>>> = Vec::with_capacity(num_rows / block_size);
        for row in (0..num_rows).step_by(block_size) {
            macroblocks.push(Vec::with_capacity(num_cols / block_size));
            for col in (0..num_cols).step_by(block_size) {
                macroblocks[row / block_size].push(
                    self.pixels[row..row + block_size]
                        .iter()
                        .map(|row| {
                            return row[col..col + block_size]
                                .iter()
                                .map(|p| p.clone())
                                .collect::<Vec<YCbCr>>();
                        })
                        .collect(),
                );
            }
        }

        macroblocks
    }
}

pub const MACROBLOCKS_SIZE: usize = 16;
type Macroblock<'a> = Vec<&'a [YCbCr]>;
