use std::{fs, process::Command};

use crate::{image::Resolution, stream::Stream};

use super::Muxer;

pub struct ShowMuxer;

impl ShowMuxer {
    fn convert_rgb_to_img(resolution: Resolution, src: &str, dst: &str) {
        Command::new("ffmpeg")
            .args(&[
                "-f",
                "rawvideo",
                "-video_size",
                &format!("{}x{}", resolution.width, resolution.height),
                "-pix_fmt",
                "rgb24",
                "-i",
                src,
                dst,
            ])
            .status()
            .expect("Failed converting image to rgb");
    }
}

impl Muxer for ShowMuxer {
    fn consume_stream(self, mut stream: impl Stream) {
        let tmp_filename = "tmp/some_output_uuid";
        let other_tmp_filename = "tmp/other_output_uuid.png";

        while let Some(image) = stream.get_next_image() {
            image.write_raw_to_file(tmp_filename);
            ShowMuxer::convert_rgb_to_img(
                stream.get_resolution(),
                tmp_filename,
                other_tmp_filename,
            );
            fs::remove_file(tmp_filename).unwrap();
            Command::new("feh")
                .arg(other_tmp_filename)
                .status()
                .expect("failed to run feh");
            fs::remove_file(other_tmp_filename).unwrap();
        }
    }
}
