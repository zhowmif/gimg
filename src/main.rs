#![allow(dead_code)]

use std::{fs, io::Read};

use colors::RGBA;
use demuxers::image_demuxer::ImageDemuxer;
use flate2::read::DeflateDecoder;
use png::{
    deflate::{self},
    encode_png,
};
use stream::Stream;

mod algebra;
mod bits;
mod codec;
mod colors;
mod dct;
mod demuxers;
mod ffmpeg;
mod filters;
mod guy_format;
mod image;
mod muxers;
mod pixel_formats;
mod png;
mod quantization;
mod queue;
mod stream;
mod tree;

fn main() {
    png_test();
}

fn png_test() {
    let mut dx = ImageDemuxer::new("files/mountain.png", "rgb24");
    let img = dx.get_next_image().unwrap();

    let mut rgba_pixels: Vec<Vec<RGBA>> = Vec::with_capacity(img.resolution.height);
    for row in img.pixels {
        let mut pixel_row: Vec<RGBA> = Vec::with_capacity(img.resolution.width);

        for pixel in row {
            pixel_row.push(pixel.into());
        }

        rgba_pixels.push(pixel_row);
    }

    let png_bytes = encode_png(rgba_pixels);
    fs::write("files/mymountain.png", png_bytes).expect("Failed to write my png");
}

fn deflate_test() {
    let input = &fs::read("save.txt").unwrap();
    // let input: Vec<u8> = repeat_n(1, 10000).collect();
    // let input = b"ABCDEABCD ABCDEABCD";
    // let input = b"AAC";
    let mut my_encoder = deflate::DeflateEncoder::new(deflate::DeflateBlockType::DynamicHuffman);
    my_encoder.write_bytes(&input[..]);
    let mut out = my_encoder.finish();
    let out_bytes = out.flush_to_bytes();

    // let mut flate2_encoder = DeflateEncoder::new(&input[..], Compression::best());
    // let mut out_bytes = Vec::new();
    // flate2_encoder.read_to_end(&mut out_bytes).unwrap();
    // print_bytes(&out_bytes);

    // print!("bytes ");
    // print_bytes(&out_bytes);

    let mut decode = DeflateDecoder::new(&out_bytes[..]);
    let mut out = Vec::new();
    decode.read_to_end(&mut out).unwrap();
    println!("flate2 out {:?}", String::from_utf8(out).unwrap());

    // let decoded = decode_deflate(&out_bytes);
    // println!("my out {:?}", String::from_utf8(decoded).unwrap());
}

fn print_bytes(bytes: &[u8]) {
    for b in bytes {
        print!("{:08b} ", b);
    }
    println!();
}
