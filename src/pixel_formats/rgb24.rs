use crate::colors::{YCbCr, RGB};

use super::{utils::assset_bytestream_size_fits_resolution, PixelFormat};

pub struct RGB24;

impl PixelFormat for RGB24 {
    fn get_name(&self) -> &str {
        "rgb24"
    }

    fn parse_bytestream(
        &self,
        bytes: &[u8],
        resolution: crate::image::Resolution,
    ) -> Vec<Vec<YCbCr>> {
        assset_bytestream_size_fits_resolution(&bytes, resolution);

        let pixels = bytes
            .chunks(resolution.width * 3)
            .map(|row| {
                row.chunks(3)
                    .map(|vec| YCbCr::from(&RGB::new(vec[0], vec[1], vec[2])))
                    .collect()
            })
            .collect();

        pixels
    }

    fn to_bytestream(&self, pixels: Vec<Vec<YCbCr>>) -> Vec<u8> {
        let bytes: Vec<u8> = pixels
            .iter()
            .flat_map(|line| {
                line.iter().flat_map(|px| {
                    let rgb = RGB::from(px);
                    vec![rgb.r, rgb.g, rgb.b]
                })
            })
            .collect();

        bytes
    }
}
