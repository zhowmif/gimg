#![allow(dead_code)]

use std::{fs, iter::once};

use colors::YCbCr;
use demuxers::raw_image_demuxer::RawImageDemuxer;
use image::{Image, Resolution};
use muxers::{show_muxer::ShowMuxer, Muxer};
use png::{decode_png, encode_png, PartialPngConfig};

mod algebra;
mod binary;
mod codec;
mod colors;
mod dct;
mod demuxers;
mod filters;
mod guy_format;
mod image;
mod muxers;
mod pixel_formats;
mod png;
mod ppm;
mod quantization;
mod queue;
mod stream;
mod tree;

fn main() {
    png_encode_test();
    // png_decode_test();
}

fn png_encode_test() {
    let png_file = fs::read("files/mountain.png").unwrap();
    let rgba_pixels = decode_png(&png_file).unwrap();

    let config = PartialPngConfig::new()
        .color_type(png::ColorType::IndexedColor)
        .bit_depth(4)
        .compression_level(png::CompressionLevel::Best);
    let png_bytes = encode_png(rgba_pixels, config);
    println!("Size {}", png_bytes.len());
    fs::write("files/mymountain.png", png_bytes).expect("Failed to write my png");
}

fn png_decode_test() {
    let png_file = fs::read("files/drawing.png").unwrap();
    let decoded_png = decode_png(&png_file).unwrap();
    let ycbcr_pixels: Vec<Vec<YCbCr>> = decoded_png
        .into_iter()
        .map(|row| row.into_iter().map(|pixel| YCbCr::from(pixel)).collect())
        .collect();

    let img = Image::new(Resolution::from_vec(&ycbcr_pixels), ycbcr_pixels);
    let dx = RawImageDemuxer::new(img);
    let show = ShowMuxer::new("rgb24");
    show.consume_stream(dx);
}
