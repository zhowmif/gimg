use std::{fs, process::Command};

use crate::{
    image::{Image, Resolution},
    stream::Stream,
};

use super::Demuxer;

pub(crate) struct ImageDemuxer {
    filename: String,
    is_consumed: bool,
    resolution: Resolution,
}

impl ImageDemuxer {
    pub fn new(filename: String) -> Self {
        let resolution = ImageDemuxer::calculate_image_resolution(&filename);

        Self {
            filename,
            is_consumed: false,
            resolution,
        }
    }

    fn calculate_image_resolution(filename: &str) -> Resolution {
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

        let (width, height) = ffprobe_output.split_once("x").unwrap();
        Resolution::new(
            str::parse(width.trim()).unwrap(),
            str::parse(height.trim()).unwrap(),
        )
    }

    fn get_image_raw_rgb(&self) -> Vec<u8> {
        let output_file_name = "tmp/some_random_uuid";
        Command::new("ffmpeg")
            .args(&[
                "-i",
                &self.filename,
                "-vf",
                &format!("scale={}:{}", self.resolution.width, self.resolution.height),
                "-pix_fmt",
                "rgb24",
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

impl Iterator for ImageDemuxer {
    type Item = Image;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_consumed {
            return None;
        }

        self.is_consumed = true;

        let value = Some(Image::from_raw_file(
            self.resolution,
            self.get_image_raw_rgb(),
        ));

        value
    }
}

impl Demuxer for ImageDemuxer {
    fn get_stream(self) -> Stream {
        Stream::new(self.resolution, Box::new(self))
    }
}
