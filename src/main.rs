#![allow(dead_code)]

use core::str;
use std::{
    fs,
    io::{self, Read},
};

use colors::RGBA;
use demuxers::image_demuxer::ImageDemuxer;
use flate2::{
    read::{DeflateDecoder, DeflateEncoder},
    Compression,
};
use png::{
    deflate::{
        self,
        huffman::{
            construct_canonical_tree_from_lengths,
            package_merge::{self, PackageMergeEncoder},
            HuffmanEncoder,
        },
        lzss::{backreference::generate, decode_lzss, encode_lzss},
        new_bitsream::NewBitStream,
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
    // let mut a = NewBitStream::from_u32_msb(0b010, 3);
    // let b = NewBitStream::from_u32_msb(0b11110001, 8);
    // let c = NewBitStream::from_u32_msb(0b001011, 6);
    // a.extend(&b);
    // a.extend(&c);
    // println!("{}", a);

    // huffman_test();
    // package_merge_test();
    // lzss_test();
    // generate();
    encode_test();
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
    let input = [97u8; 1];
    let mut my_encoder = deflate::DeflateEncoder::new(deflate::BlockType::FixedHuffman);
    my_encoder.write_bytes(&input);
    let mut out = my_encoder.finish();
    // println!("out {}", out);
    let out_bytes = out.to_bytes();

    // let mut decode = DeflateDecoder::new(&out_bytes[..]);
    // let mut res = Vec::new();
    // decode.read(&mut res).unwrap();
    //
    // println!("Res {:?}", res);

    deflateencoder_read_hello_world(&[97]);

    // println!("{:?}", flate_out);
}

fn deflateencoder_read_hello_world(input: &[u8]) {
    let mut ret_vec = Vec::new();
    let mut deflater = DeflateEncoder::new(input, Compression::best());
    deflater.read_to_end(&mut ret_vec).unwrap();

    for b in ret_vec.iter() {
        print!("{:08b}", b);
    }
    println!();

    let mut inflater = DeflateDecoder::new(&ret_vec[..]);
    let mut b = Vec::new();
    inflater.read_to_end(&mut b).unwrap();
    println!("Got {:?}", b);
}

fn lzss_test() {
    // let input = "It was the best of times, it was the worst of times, it was the age of wisdom, it was the age of foolishness, it was the epoch of belief, it was the epoch of incredulity, it was the season of Light, it was the season of Darkness, it was the spring of hope, it was the winter of despair, we had everything before us, we had nothing before us, we were all going direct to Heaven, we were all going direct the other way—in short, the period was so far like the present period, that some of its noisiest authorities insisted on its being received, for good or for evil, in the superlative degree of comparison only.".as_bytes();
    let input = fs::read("save.txt").expect("failed to read input file");
    let lzss_encoded = encode_lzss(&input, 1000);
    let decoded_bytes = decode_lzss(&lzss_encoded);
    let decoded = str::from_utf8(&decoded_bytes).expect("Lzss decode produced invalid utf8");
    println!("is good {}", input == decoded_bytes);
}
