#![feature(portable_simd)]
#![allow(dead_code)]

use std::{fs, time::Instant};

use cli::parse_args;
use colors::{Rgba, YCbCr};
use image::{Image, Resolution};
use png::{
    decode_png,
    deflate::{decode::decode_deflate, DeflateEncoder},
    encode_png, CompressionLevel, PartialPngConfig,
};
use ppm::decode_ppm;
// use simd_utils::{CALLS, MATCHING_BYTES};

mod algebra;
mod binary;
mod cli;
mod colors;
mod image;
mod pixel_formats;
mod png;
mod ppm;
mod queue;
mod simd_utils;

fn main() {
    parse_args();

    // png_encode_test();
    // png_decode_test();
    // deflate_test();
}

fn deflate_test() {
    // let input = &fs::read("files/text.txt").unwrap();
    let input = b"helhelaaaaaaaaaaa";

    // compress(input, CompressionLevel::Fast);
    let cmp = compress(input, CompressionLevel::Best);
    let dec = String::from_utf8(decode_deflate(&cmp).unwrap()).unwrap();
    println!("Decoded: {dec}");

    // let decoded = String::from_utf8(decode_deflate(&out).unwrap()).expect("utf8 failed");
    // if decoded.len() < 100 {
    //     println!("{}", decoded);
    // } else {
    //     println!("{}", &decoded[..100]);
    // }
}

fn compress(input: &[u8], compression_level: CompressionLevel) -> Vec<u8> {
    let start = Instant::now();
    let mut enc = DeflateEncoder::new(compression_level);
    enc.write_bytes(input);
    let bytes = enc.finish().flush_to_bytes();
    let end = Instant::now();
    println!("{:?}, {} {:?}", compression_level, bytes.len(), end - start);
    bytes
}

fn png_encode_test() {
    // let png_file = fs::read("files/mountain.png").unwrap();
    // let rgba_pixels = decode_png(&png_file).unwrap();
    let ppm_file = fs::read("files/small.ppm").unwrap();
    let rgba_pixels = decode_ppm(&ppm_file)
        .unwrap()
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|p| Rgba::new(p.r, p.g, p.b, u8::MAX))
                .collect()
        })
        .collect();

    let config = PartialPngConfig::new().compression_level(png::CompressionLevel::Best);
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
}
