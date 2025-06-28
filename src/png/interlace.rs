use std::iter::repeat_n;

use crate::colors::RGBA;

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
                        println!("{pass_number}, {:?} {}", new_pass_coordinates, pixel.r);

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

const ADAM7_SUBSET_PASSES: [[usize; 8]; 8] = [
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
    ADAM7_SUBSET_PASSES[y & 0b111][x & 0b111]
}
