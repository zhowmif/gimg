use crate::{
    colors::{YCbCr, RGB},
    image::Resolution,
};

use super::PixelFormat;

pub struct YUV420p;

impl PixelFormat for YUV420p {
    fn get_name(&self) -> &str {
        "yuv420p"
    }

    fn parse_bytestream(&self, bytes: &[u8], resolution: Resolution) -> Vec<Vec<YCbCr>> {
        let total_number_of_pixels = resolution.width * resolution.height;
        let number_of_luma_bytes = total_number_of_pixels;
        let number_of_chroma_channel_bytes = number_of_luma_bytes / 4;

        let luma_bytes = &bytes[0..number_of_luma_bytes];
        let cb_bytes =
            &bytes[number_of_luma_bytes..number_of_luma_bytes + number_of_chroma_channel_bytes];
        let cr_bytes = &bytes[number_of_luma_bytes + number_of_chroma_channel_bytes
            ..number_of_luma_bytes + number_of_chroma_channel_bytes * 2];

        (0..resolution.height)
            .map(|row| {
                (0..resolution.width)
                    .map(|col| {
                        let pixel_number = row * col;
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
        todo!()
    }
}
