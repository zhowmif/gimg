use std::iter::repeat_n;

use crate::{binary::byte_reader::ByteReader, colors::RGBA, png_assert};

use super::PngParseError;

#[derive(Debug, Clone, Copy)]
pub enum InterlaceMethod {
    NoInterlace,
    Adam7,
}

impl InterlaceMethod {
    pub fn perform_pass_extraction(&self, pixels: Vec<Vec<RGBA>>) -> Vec<Vec<Vec<RGBA>>> {
        match self {
            InterlaceMethod::NoInterlace => vec![pixels],
            InterlaceMethod::Adam7 => {
                let mut reduced_images: Vec<Vec<Vec<RGBA>>> = Vec::new();

                for pass in ADAM7_PASSES.iter() {
                    let mut reduced_image: Vec<Vec<RGBA>> = Vec::new();

                    for subset_top in (0..pixels.len()).step_by(8) {
                        for (row_num, pass_row) in *pass {
                            let y = subset_top + row_num;
                            if y >= pixels.len() {
                                break;
                            }

                            let mut row: Vec<RGBA> = Vec::new();

                            for subset_left in (0..pixels[y].len()).step_by(8) {
                                for col in *pass_row {
                                    let x = subset_left + col;
                                    if x >= pixels[y].len() {
                                        break;
                                    }

                                    row.push(pixels[y][x].clone());
                                }
                            }

                            reduced_image.push(row);
                        }
                    }

                    reduced_images.push(reduced_image);
                }

                reduced_images
            }
        }
    }

    pub fn reconstruct_filtered_scanlines(
        &self,
        data: &[u8],
        height: usize,
        width: usize,
        bits_per_pixel: usize,
    ) -> Result<Vec<Vec<Vec<u8>>>, PngParseError> {
        let filter_byte_size = 1;

        match self {
            InterlaceMethod::NoInterlace => {
                let bytes_per_scanline = filter_byte_size + ((width * bits_per_pixel) >> 3);
                let expected_data_size = height * bytes_per_scanline;
                png_assert!(
                    data.len() == expected_data_size,
                    format!(
            "Expected {} bytes for resolution {}x{} after decompressing, but received {}",
            expected_data_size,
            height,
            width,
            data.len()
        )
                );

                Ok(vec![data
                    .chunks(bytes_per_scanline)
                    .map(|scanline| scanline.to_vec())
                    .collect()])
            }
            InterlaceMethod::Adam7 => {
                let mut data_reader = ByteReader::new(data);
                let scanline_dimensions_by_pass = adam7_scanlines_dimensions_by_pass(height, width);
                let mut all_scanlines = Vec::new();

                for pass in 0..ADAM7_PASSES.len() {
                    let (number_of_scanlines, scanline_number_of_pixels) =
                        scanline_dimensions_by_pass[pass];
                    let scanline_width_bytes = filter_byte_size
                        + ((scanline_number_of_pixels as f32 * bits_per_pixel as f32) / 8.).ceil()
                            as usize;
                    let expected_data_size = number_of_scanlines * scanline_width_bytes;

                    let current_pass_scanlines: Vec<Vec<u8>> = match data_reader
                        .read_bytes(expected_data_size)
                    {
                        Some(bytes) => bytes
                            .chunks(scanline_width_bytes)
                            .map(|slice| slice.to_vec())
                            .collect(),
                        None => {
                            return Err(PngParseError(format!(
                                        "Expected {expected_data_size} bytes for adam7 pass #{pass}, but only had {} bytes left in the buffer",
                                        data_reader.number_of_bytes_left()
                            )));
                        }
                    };

                    all_scanlines.push(current_pass_scanlines);
                }

                png_assert!(
                    data_reader.is_finished(),
                    format!(
                        "Interlace decode error: Decompressed data is too long, expected {} bytes but data is {} bytes", data.len() -  data_reader.number_of_bytes_left(), data.len()
                    )
                );

                Ok(all_scanlines)
            }
        }
    }

    pub fn deinterlace_image(
        &self,
        mut reduced_images: Vec<Vec<Vec<RGBA>>>,
        image_height: usize,
        image_width: usize,
    ) -> Vec<Vec<RGBA>> {
        match self {
            InterlaceMethod::NoInterlace => reduced_images.pop().unwrap(),
            InterlaceMethod::Adam7 => {
                let mut image: Vec<Vec<RGBA>> = Vec::with_capacity(image_height);
                let mut pass_indexes: Vec<Option<((usize, usize), usize)>> =
                    repeat_n(None, ADAM7_PASSES.len()).collect();

                for y in 0..image_height {
                    let mut row = Vec::with_capacity(image_width);
                    for x in 0..image_width {
                        let pass_number: usize = coordinates_to_pass_number(y, x);
                        let new_pass_coordinates = match pass_indexes[pass_number] {
                            Some(((pass_y, pass_x), pass_absolute_y)) => {
                                if y > pass_absolute_y {
                                    ((pass_y + 1, 0), y)
                                } else {
                                    ((pass_y, pass_x + 1), y)
                                }
                            }
                            None => ((0, 0), y),
                        };

                        let pixel = reduced_images[pass_number][new_pass_coordinates.0 .0]
                            [new_pass_coordinates.0 .1]
                            .clone();

                        row.push(pixel);

                        pass_indexes[pass_number] = Some(new_pass_coordinates);
                    }
                    image.push(row);
                }

                image
            }
        }
    }
}

impl Default for InterlaceMethod {
    fn default() -> Self {
        InterlaceMethod::NoInterlace
    }
}

impl TryFrom<u8> for InterlaceMethod {
    type Error = PngParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NoInterlace),
            1 => Ok(Self::Adam7),
            _ => Err(PngParseError(format!(
                "Unrecognized interlace method {value}"
            ))),
        }
    }
}

impl Into<u8> for &InterlaceMethod {
    fn into(self) -> u8 {
        match self {
            InterlaceMethod::NoInterlace => 0,
            InterlaceMethod::Adam7 => 1,
        }
    }
}

const ADAM7_PASSES: [&[(usize, &[usize])]; 7] = [
    &[(0, &[0])],
    &[(0, &[4])],
    &[(4, &[0, 4])],
    &[(0, &[2, 6]), (4, &[2, 6])],
    &[(2, &[0, 2, 4, 6]), (6, &[0, 2, 4, 6])],
    &[
        (0, &[1, 3, 5, 7]),
        (2, &[1, 3, 5, 7]),
        (4, &[1, 3, 5, 7]),
        (6, &[1, 3, 5, 7]),
    ],
    &[
        (1, &[0, 1, 2, 3, 4, 5, 6, 7]),
        (3, &[0, 1, 2, 3, 4, 5, 6, 7]),
        (5, &[0, 1, 2, 3, 4, 5, 6, 7]),
        (7, &[0, 1, 2, 3, 4, 5, 6, 7]),
    ],
];

const ADAM7_BLOCK_PASSES: [[usize; 8]; 8] = [
    [0, 5, 3, 5, 1, 5, 3, 5],
    [6, 6, 6, 6, 6, 6, 6, 6],
    [4, 5, 4, 5, 4, 5, 4, 5],
    [6, 6, 6, 6, 6, 6, 6, 6],
    [2, 5, 3, 5, 2, 5, 3, 5],
    [6, 6, 6, 6, 6, 6, 6, 6],
    [4, 5, 4, 5, 4, 5, 4, 5],
    [6, 6, 6, 6, 6, 6, 6, 6],
];

fn coordinates_to_pass_number(y: usize, x: usize) -> usize {
    ADAM7_BLOCK_PASSES[y & 0b111][x & 0b111]
}

fn adam7_scanlines_dimensions_by_pass(height: usize, width: usize) -> Vec<(usize, usize)> {
    let number_of_extra_rows = height & 0b111;
    let number_of_extra_cols = width & 0b111;

    (0..ADAM7_PASSES.len())
        .map(|pass_number| {
            let full_block_rows = FULL_BLOCK_NUMBER_OF_ROWS_BY_PASS[pass_number] * (height >> 3);
            let pass_number_of_extra_rows =
                PARTIAL_BLOCK_NUMBER_OF_ROWS_BY_HEIGHT_BY_PASS[pass_number][number_of_extra_rows];
            let full_block_cols = FULL_BLOCK_NUMBER_OF_COLS_BY_PASS[pass_number] * (width >> 3);
            let pass_number_of_extra_cols =
                PARTIAL_BLOCK_NUMBER_OF_COLS_BY_WIDTH_BY_PASS[pass_number][number_of_extra_cols];

            (
                full_block_rows + pass_number_of_extra_rows,
                full_block_cols + pass_number_of_extra_cols,
            )
        })
        .collect()
}

const FULL_BLOCK_NUMBER_OF_ROWS_BY_PASS: [usize; 7] = [1, 1, 1, 2, 2, 4, 4];
const FULL_BLOCK_NUMBER_OF_COLS_BY_PASS: [usize; 7] = [1, 1, 2, 2, 4, 4, 8];
const PARTIAL_BLOCK_NUMBER_OF_ROWS_BY_HEIGHT_BY_PASS: [[usize; 9]; 7] = [
    [0, 1, 1, 1, 1, 1, 1, 1, 1],
    [0, 1, 1, 1, 1, 1, 1, 1, 1],
    [0, 0, 0, 0, 0, 1, 1, 1, 1],
    [0, 1, 1, 1, 1, 2, 2, 2, 2],
    [0, 0, 0, 1, 1, 1, 1, 2, 2],
    [0, 1, 1, 2, 2, 3, 3, 4, 4],
    [0, 0, 1, 1, 2, 2, 3, 3, 4],
];
const PARTIAL_BLOCK_NUMBER_OF_COLS_BY_WIDTH_BY_PASS: [[usize; 9]; 7] = [
    [0, 1, 1, 1, 1, 1, 1, 1, 1],
    [0, 0, 0, 0, 0, 1, 1, 1, 1],
    [0, 1, 1, 1, 1, 2, 2, 2, 2],
    [0, 0, 0, 1, 1, 1, 1, 2, 2],
    [0, 1, 1, 2, 2, 3, 3, 4, 4],
    [0, 0, 1, 1, 2, 2, 3, 3, 4],
    [0, 1, 2, 3, 4, 5, 6, 7, 8],
];
