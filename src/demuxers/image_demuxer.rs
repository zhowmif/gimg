use std::{fs, process::Command};

use crate::{
    image::{Image, Resolution},
    pixel_formats::{get_pixel_format, PixelFormat},
    stream::Stream,
};

use super::Demuxer;

pub(crate) struct ImageDemuxer {
    filename: String,
    is_consumed: bool,
    resolution: Resolution,
    pixel_format: Box<dyn PixelFormat>,
}

impl ImageDemuxer {
    pub fn new(filename: &str, pixel_format: &str) -> Self {
        let resolution = ImageDemuxer::calculate_resolotion(&filename);

        Self {
            filename: filename.to_string(),
            is_consumed: false,
            resolution,
            pixel_format: get_pixel_format(pixel_format),
        }
    }

    fn calculate_resolotion(filename: &str) -> Resolution {
        let ffprobe_output = String::from_utf8(
            Command::new("ffprobe")
                .args(&[
                    "-v",
                    "error",
                    "-select_streams",
                    "v:0",
                    "-show_entries",
                    "stream=width,height",
                    "-of",
                    "csv=s=x:p=0",
                    filename,
                ])
                .output()
                .expect(&format!("Failed to read resolution for file {}", filename))
                .stdout,
        )
        .unwrap();

        let (width, height) = ffprobe_output
            .split_once("x")
            .expect(&format!("Failed to read resolution for file {}", filename));
        Resolution::new(
            str::parse(width.trim()).unwrap(),
            str::parse(height.trim()).unwrap(),
        )
    }

    fn get_image_raw_pixels(&self) -> Vec<u8> {
        let output_file_name = "tmp/some_random_uuid";
        Command::new("ffmpeg")
            .args(&[
                "-i",
                &self.filename,
                "-vf",
                &format!("scale={}:{}", self.resolution.width, self.resolution.height),
                "-pix_fmt",
                &self.pixel_format.get_name(),
                "-f",
                "rawvideo",
                output_file_name,
            ])
            .status()
            .expect("Failed converting rgb to image");
        let bytes = fs::read(output_file_name).expect("failed to read raw rgb file");
        fs::remove_file(output_file_name).unwrap();

        bytes
    }
}

impl Stream for ImageDemuxer {
    fn get_next_image(&mut self) -> Option<Image> {
        if self.is_consumed {
            return None;
        }

        self.is_consumed = true;

        let value = Some(Image::new(
            self.resolution,
            self.pixel_format
                .parse_bytestream(&self.get_image_raw_pixels(), self.resolution),
        ));

        value
    }

    fn get_resolution(&self) -> Resolution {
        self.resolution
    }
}

impl Demuxer for ImageDemuxer {}
