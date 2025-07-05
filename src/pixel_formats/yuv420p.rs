use crate::{algebra::align_up, colors::YCbCr, image::Resolution};

use super::PixelFormat;

pub struct YUV420p;

impl YUV420p {
    fn get_adjacant_pixels(pixels: &[Vec<YCbCr>], row: usize, col: usize) -> Vec<YCbCr> {
        let mut adjacant_pixels: Vec<YCbCr> = Vec::with_capacity(4);

        for i in 0..=1 {
            for j in 0..=1 {
                let pixel = pixels
                    .get(row + i)
                    .and_then(|r| r.get(col + j))
                    .unwrap_or(&pixels[row][col]);
                adjacant_pixels.push(*pixel)
            }
        }

        adjacant_pixels
    }
}

impl PixelFormat for YUV420p {
    fn get_name(&self) -> &str {
        "yuv420p"
    }

    fn parse_bytestream(&self, bytes: &[u8], resolution: Resolution) -> Vec<Vec<YCbCr>> {
        let total_number_of_pixels = resolution.width * resolution.height;
        let number_of_luma_bytes = total_number_of_pixels;
        let number_of_chroma_channel_bytes =
            (align_up(resolution.width, 4) * align_up(resolution.height, 4)) / 4;

        let luma_bytes = &bytes[0..number_of_luma_bytes];
        let cb_bytes =
            &bytes[number_of_luma_bytes..number_of_luma_bytes + number_of_chroma_channel_bytes];
        let cr_bytes = &bytes[number_of_luma_bytes + number_of_chroma_channel_bytes
            ..number_of_luma_bytes + number_of_chroma_channel_bytes * 2];

        (0..resolution.height)
            .map(|row| {
                (0..resolution.width)
                    .map(|col| {
                        let pixel_number = row * resolution.width + col;
                        let luma = luma_bytes[pixel_number];
                        let drow = (row / 2) * resolution.width / 2;
                        let dcol = col / 2;
                        let cb = cb_bytes[drow + dcol];
                        let cr = cr_bytes[drow + dcol];

                        YCbCr::new(luma, cb, cr)
                    })
                    .collect()
            })
            .collect()
    }

    fn to_bytestream(&self, pixels: Vec<Vec<YCbCr>>) -> Vec<u8> {
        let number_of_pixels = pixels.len() * pixels[0].len();
        let aligned_number_of_pixels = align_up(pixels.len(), 4) * align_up(pixels[0].len(), 4);
        let mut bytestream: Vec<u8> = vec![0; number_of_pixels + aligned_number_of_pixels / 2];
        let cb_channel_offset = number_of_pixels;
        let cr_channel_offset = number_of_pixels + aligned_number_of_pixels / 4;
        let mut chroma_offset = 0;
        let mut luma_offset = 0;

        for (row, r) in pixels.iter().enumerate() {
            for (col, pixel) in r.iter().enumerate() {
                bytestream[luma_offset] = pixel.y;
                luma_offset += 1;
                if row & 1 == 0 && col & 1 == 0 {
                    let adjacant_pixels = Self::get_adjacant_pixels(&pixels, row, col);
                    bytestream[cb_channel_offset + chroma_offset] =
                        (adjacant_pixels.iter().map(|pix| pix.cb as f32).sum::<f32>()
                            / adjacant_pixels.len() as f32)
                            .round() as u8;
                    bytestream[cr_channel_offset + chroma_offset] =
                        (adjacant_pixels.iter().map(|pix| pix.cr as f32).sum::<f32>()
                            / adjacant_pixels.len() as f32)
                            .round() as u8;
                    chroma_offset += 1;
                }
            }
        }

        bytestream
    }
}
