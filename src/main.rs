#![allow(dead_code)]

use std::{
    fs,
    io::Read,
    time::{self, Instant},
};

use colors::RGBA;
use demuxers::{image_demuxer::ImageDemuxer, Demuxer};
use flate2::{
    read::{DeflateDecoder, DeflateEncoder, ZlibEncoder},
    Compression,
};
use muxers::Muxer;
use png::{
    deflate::{
        self,
        lzss::{backreference::generate, decode_lzss, encode_lzss},
        zlib::zlib_encode,
    },
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
    generate();
    // encode_test();
    // png_test();
    // let input = fs::read("bee_movie.txt").expect("Failed to read input file");
    // let start = Instant::now();
    // let encoded_data = encode_lzss(&input, (2 as usize).pow(15));
    // let end = Instant::now();
    // println!("{:?}", end - start);

    // let decoded = decode_lzss(&encoded_data);
    // let decoded_str = String::from_utf8(decoded).expect("Did not get valid utf-8");
    // println!("{decoded_str}");
    // println!("Original length: {}", input.len() * 8);
    // println!("data length: {}", encoded_data.len());
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
    let mut deflater = ZlibEncoder::new(&data[..], Compression::none());
    deflater.read_to_end(&mut ret_vec).unwrap();

    my_encoder.write_bytes(&data[0..10]);
    my_encoder.write_bytes(&data[10..]);
    let encoded = my_encoder.finish();
    let zlib_encoded = zlib_encode(my_encoder).to_bytes();
    fs::write("deflate.bin", ret_vec).unwrap();
    fs::write("mydeflate.bin", zlib_encoded.clone()).unwrap();

    let encoded_bytes = encoded.to_bytes().clone();
    let mut decode = DeflateDecoder::new(&encoded_bytes[..]);
    let mut res = Vec::new();
    decode.read_to_end(&mut res).unwrap();

    println!("equal? {:?}", res == data);
    // println!("res {:?}", res);
}
