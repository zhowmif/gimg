use std::{fs, process::Command};

use crate::{
    colors::RGB,
    pixel_formats::{get_pixel_format, PixelFormat},
    ppm::encode_ppm,
    stream::Stream,
};

use super::Muxer;

pub struct ShowMuxer {
    pixel_format: Box<dyn PixelFormat>,
}

impl ShowMuxer {
    pub fn new(pixel_format: &str) -> Self {
        let pixel_format = get_pixel_format(pixel_format);

        Self { pixel_format }
    }
}

impl Muxer for ShowMuxer {
    fn consume_stream(self, mut stream: impl Stream) {
        let tmp_filename = "tmp/some_output_uuid";

        while let Some(image) = stream.get_next_image() {
            let rgb_pixels: Vec<_> = image
                .pixels
                .into_iter()
                .map(|row| row.into_iter().map(|pix| RGB::from(&pix)).collect())
                .collect();
            let ppm_bytes = encode_ppm(&rgb_pixels);

            fs::write(tmp_filename, ppm_bytes).unwrap();
            Command::new("feh")
                .args(&["--force-aliasing", tmp_filename])
                .status()
                .expect("failed to run feh");
            fs::remove_file(tmp_filename).unwrap();
        }
    }
}
