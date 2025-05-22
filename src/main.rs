#![allow(dead_code)]

mod algebra;
mod bits;
mod colors;
mod dct;
mod demuxers;
mod ffmpeg;
mod filters;
mod image;
mod muxers;
mod pixel_formats;
mod queue;
mod stream;
mod tree;

use colors::{YCbCr, RGB};
use dct::DiscreteCosineTransformer;
use demuxers::{
    droidcam::DroidCamDemuxer,
    image_demuxer::{self, ImageDemuxer},
    Demuxer,
};
use ffmpeg::convert_img_to_rgb;
use filters::{grayscale::GrayScaleFilter, Filter};
use image::{Image, Resolution, MACROBLOCKS_SIZE};
use muxers::{
    show_muxer::{self, ShowMuxer},
    Muxer,
};
use pixel_formats::{yuv420p::YUV420p, PixelFormat};
use std::fs;
use stream::Stream;

const INPUT_FILE: &str = "files/input.jpg";
const RGB_FILE: &str = "files/raw.rgb";
const OUTPUT_FILE: &str = "files/out.jpg";

fn dct_test() {
    let mut image_demuxer = ImageDemuxer::new(INPUT_FILE, "rgb24");
    let first_image = image_demuxer.get_next_image().unwrap();
    let macroblocks = first_image.get_macroblocks(MACROBLOCKS_SIZE);
    let cb_macroblocks = Image::get_cb_macroblocks(&macroblocks);
    let dct = dct::DiscreteCosineTransformer::new();
    let amplitudes = dct.dct(&cb_macroblocks[10][10]);
    let (normalized, fact) = DiscreteCosineTransformer::normalize_amplitudes(amplitudes);
    let inversed = DiscreteCosineTransformer::inverse_normalization(normalized, fact);
    let cb_again = dct.idct(inversed);
    let macroblock = &macroblocks[10][10];
    let mut old_macroblock_bytes: Vec<u8> = vec![];
    let mut new_macroblock_bytes: Vec<u8> = vec![];

    for r in 0..MACROBLOCKS_SIZE {
        for c in 0..MACROBLOCKS_SIZE {
            let prev = macroblock[r][c];
            let prev_rgb = RGB::from(&prev);
            old_macroblock_bytes.push(prev_rgb.r);
            old_macroblock_bytes.push(prev_rgb.g);
            old_macroblock_bytes.push(prev_rgb.b);
            let new = RGB::from(&YCbCr::new(prev.y, cb_again[r][c], prev.cr));
            new_macroblock_bytes.push(new.r);
            new_macroblock_bytes.push(new.g);
            new_macroblock_bytes.push(new.b);
        }
    }
    fs::write("files/old_macro", old_macroblock_bytes).unwrap();
    fs::write("files/new_macro", new_macroblock_bytes).unwrap();

    println!("{:?}", cb_macroblocks[10][10]);
    println!("");
    println!("{:?}", amplitudes);
    println!("{:?}", normalized);
    println!("{:?}", inversed);
}

fn main() {
    let dx = ImageDemuxer::new("files/mountain.png", "yuv420p");
    // let img = dx.get_next_image().unwrap();
    // println!("First pixels: {:?}", &img.pixels[0][0..50]);
    let show_muxer = ShowMuxer::new("yuv420p");
    show_muxer.consume_stream(dx);
}
