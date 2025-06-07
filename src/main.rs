#![allow(dead_code)]

use std::{fs, io::Read};

use colors::RGBA;
use demuxers::{image_demuxer::ImageDemuxer, Demuxer};
use flate2::{
    read::{DeflateDecoder, DeflateEncoder},
    Compression,
};
use muxers::Muxer;
use png::{deflate, encode_png};
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
    // encode_test();
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

fn encode_test() {
    let mut my_encoder = deflate::DeflateEncoder::new(deflate::BlockType::None);
    let mut ret_vec: Vec<u8> = Vec::new();
    let data_length: u32 = u16::MAX as u32 + 43243;
    // let data_length: u32 = 0b0000000110000111;
    println!("{:#018b}", data_length);
    println!("{data_length}");
    let data: Vec<u8> = (0..=data_length).map(|_i| b'a' as u8).collect();
    let mut deflater = DeflateEncoder::new(&data[..], Compression::none());
    deflater.read_to_end(&mut ret_vec).unwrap();

    my_encoder.write_bytes(&data[0..10]);
    my_encoder.write_bytes(&data[10..]);
    let encoded = my_encoder.finish();
    fs::write("deflate.bin", ret_vec).unwrap();
    fs::write("mydeflate.bin", encoded.clone()).unwrap();

    let mut decode = DeflateDecoder::new(&encoded[..]);
    let mut res = Vec::new();
    decode.read_to_end(&mut res).unwrap();

    println!("equal? {:?}", res == data);
    // println!("res {:?}", res);
}
