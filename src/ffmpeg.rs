use std::process::Command;

pub fn convert_rgb_to_img(src: &str, dst: &str) {
    Command::new("ffmpeg")
        .args(&[
            "-f",
            "rawvideo",
            "-video_size",
            "750x1125",
            "-pix_fmt",
            "rgb24",
            "-i",
            src,
            dst,
        ])
        .status()
        .expect("Failed converting image to rgb");
}

pub fn convert_img_to_rgb(src: &str, dst: &str) {
    Command::new("ffmpeg")
        .args(&[
            "-i",
            src,
            "-vf",
            "scale=750:1125",
            "-pix_fmt",
            "rgb24",
            "-f",
            "rawvideo",
            dst,
        ])
        .status()
        .expect("Failed converting rgb to image");
}

pub fn display_image(file: &str) {
    // Command::new("feh").arg(file).status().expect("failed to run feh");
    Command::new("start").arg(file).status().expect("failed to run feh");
}
