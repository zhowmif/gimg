#![feature(portable_simd)]
#![allow(dead_code)]

use std::{fs, time::Instant};

use colors::{YCbCr, Rgba};
use demuxers::raw_image_demuxer::RawImageDemuxer;
use image::{Image, Resolution};
use muxers::{show_muxer::ShowMuxer, Muxer};
use png::{decode_png, deflate::DeflateEncoder, encode_png, CompressionLevel, PartialPngConfig};
use ppm::decode_ppm;

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
mod simd_utils;
mod stream;

fn main() {
    // println!("{:?}", subtract_simd(&vec![10, 5, 12], &vec![1, 2, 3]));
    png_encode_test();
    // png_decode_test();
    // deflate_test();
}

fn deflate_test() {
    let input = &fs::read("files/text.txt").unwrap();
    // let input = b"it was, it was";

    // compress(input, CompressionLevel::Fast);
    compress(input, CompressionLevel::Best);

    // let decoded = String::from_utf8(decode_deflate(&out).unwrap()).expect("utf8 failed");
    // if decoded.len() < 100 {
    //     println!("{}", decoded);
    // } else {
    //     println!("{}", &decoded[..100]);
    // }
}

fn compress(input: &[u8], compression_level: CompressionLevel) {
    let start = Instant::now();
    let mut enc = DeflateEncoder::new(compression_level);
    enc.write_bytes(input);
    let l = enc.finish().flush_to_bytes().len();
    let end = Instant::now();
    println!("{:?}, {l} {:?}", compression_level, end - start);
}

fn png_encode_test() {
    // let png_file = fs::read("files/mountain.png").unwrap();
    // let rgba_pixels = decode_png(&png_file).unwrap();
    let ppm_file = fs::read("files/mountain.ppm").unwrap();
    let rgba_pixels = decode_ppm(&ppm_file)
        .unwrap()
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|p| Rgba::new(p.r, p.g, p.b, u8::MAX))
                .collect()
        })
        .collect();

    let config = PartialPngConfig::new()
        // .color_type(png::ColorType::TrueColorAlpha)
        .interlace_method(png::InterlaceMethod::Adam7)
        .compression_level(png::CompressionLevel::Fast);
    let png_bytes = encode_png(rgba_pixels, config);
    println!("Size {}", png_bytes.len());
    fs::write("files/mymountain.png", png_bytes).expect("Failed to write my png");
}

fn png_decode_test() {
    let png_file = fs::read("files/drawing.png").unwrap();
    let decoded_png = decode_png(&png_file).unwrap();
    let ycbcr_pixels: Vec<Vec<YCbCr>> = decoded_png
        .into_iter()
        .map(|row| row.into_iter().map(YCbCr::from).collect())
        .collect();

    let img = Image::new(Resolution::from_vec(&ycbcr_pixels), ycbcr_pixels);
    let dx = RawImageDemuxer::new(img);
    let show = ShowMuxer::new("rgb24");
    show.consume_stream(dx);
}
