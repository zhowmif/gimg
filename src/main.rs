#![allow(dead_code)]

use core::str;
use std::{
    collections::HashMap,
    fs,
    io::{self, Read},
    iter::{repeat, repeat_n},
    u16,
};

use colors::RGBA;
use demuxers::image_demuxer::ImageDemuxer;
use flate2::{
    read::{DeflateDecoder, DeflateEncoder, ZlibEncoder},
    Compression,
};
use png::{
    deflate::{
        self,
        decode::decode_deflate,
        huffman::{
            construct_canonical_tree_from_lengths,
            package_merge::{self, PackageMergeEncoder},
            HuffmanEncoder,
        },
        lzss::{backreference::generate, decode_lzss, encode_lzss},
        new_bitsream::{BitStreamReader, NewBitStream},
        prefix_table::get_cl_codes_for_code_lengths,
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
    // encode_test();
    decode_test();
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
    // let input = fs::read("save.txt").unwrap();
    let input = b"ABCDEABCD ABCDEABCD";
    let mut my_encoder = deflate::DeflateEncoder::new(deflate::BlockType::DynamicHuffman);
    my_encoder.write_bytes(input);
    let mut out = my_encoder.finish();
    let out_bytes = out.flush_to_bytes();
    // // print_bytes(&out_bytes);
    //
    let mut decode = DeflateDecoder::new(&out_bytes[..]);

    // let mut out = Vec::new();
    // decode.read_to_end(&mut out).unwrap();
    // println!("Out {:?}", out);

    let mut s = String::new();
    decode.read_to_string(&mut s).unwrap();
    println!("Res {:?}", &s);

    // println!("printed {:?}", &s[1080..1095]);
    // println!("at the start it is {:?}", &input[44..47]);
    // println!("at the end it is {:?}", &input[1088..1091]);

    // deflateencoder_read_hello_world(&input);
    // deflateencoder_read_hello_world(&[0, 1]);
    // // deflateencoder_read_hello_world(&[0, 255]);
    // deflateencoder_read_hello_world(&[239]);

    // println!("{:?}", flate_out);
}

fn decode_test() {
    let input = fs::read("save.txt").unwrap();
    // let input: Vec<u8> = repeat_n(1, 10_000).collect();
    // let input = b"ABCDEABCD ABCDEABCD";
    let mut my_encoder = deflate::DeflateEncoder::new(deflate::BlockType::DynamicHuffman);
    my_encoder.write_bytes(&input[..]);
    let mut out = my_encoder.finish();
    let out_bytes = out.flush_to_bytes();

    // let mut flate2_encoder = DeflateEncoder::new(&input[..], Compression::best());
    // let mut out_bytes = Vec::new();
    // flate2_encoder.read_to_end(&mut out_bytes).unwrap();

    // print!("bytes ");
    // print_bytes(&out_bytes);

    // let mut decode = DeflateDecoder::new(&out_bytes[..]);
    // let mut out = Vec::new();
    // decode.read_to_end(&mut out).unwrap();
    // println!("flate2 out {:?}", String::from_utf8(out).unwrap());

    let decoded = decode_deflate(&out_bytes);
    println!("my out {:?}", String::from_utf8(decoded).unwrap());
}

fn deflateencoder_read_hello_world(input: &[u8]) {
    let mut ret_vec = Vec::new();
    let mut deflater = DeflateEncoder::new(input, Compression::best());
    deflater.read_to_end(&mut ret_vec).unwrap();
    print_bytes(&ret_vec);
}

fn print_bytes(bytes: &[u8]) {
    for b in bytes {
        print!("{:08b} ", b);
    }
    println!();
}

fn lzss_test() {
    // let input = "It was the best of times, it was the worst of times, it was the age of wisdom, it was the age of foolishness, it was the epoch of belief, it was the epoch of incredulity, it was the season of Light, it was the season of Darkness, it was the spring of hope, it was the winter of despair, we had everything before us, we had nothing before us, we were all going direct to Heaven, we were all going direct the other way—in short, the period was so far like the present period, that some of its noisiest authorities insisted on its being received, for good or for evil, in the superlative degree of comparison only.".as_bytes();
    let input = fs::read("save.txt").expect("failed to read input file");
    let lzss_encoded = encode_lzss(&input, 1000);
    let decoded_bytes = decode_lzss(&lzss_encoded);
    let decoded = str::from_utf8(&decoded_bytes).expect("Lzss decode produced invalid utf8");
    println!("is good {}", input == decoded_bytes);
}
