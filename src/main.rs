#![allow(dead_code)]

use std::{fs, io::Read};

use colors::RGBA;
use demuxers::image_demuxer::ImageDemuxer;
use flate2::{
    read::{DeflateDecoder, ZlibDecoder, ZlibEncoder},
    Compression,
};
use png::{
    deflate::{self, zlib::zlib_encode},
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
    // let a = NewBitStream::from_u32_msb(0b01, 2);
    // let a = NewBitStream {
    //     stream: vec![111],
    //     working_byte: 0,
    //     current_bit_number: 1,
    // };
    // let mut g = NewBitStream::new();
    // g.extend(&a);
    // println!("{}", a);
    // println!("{}", g);
    // let b = NewBitStream::from_u32_msb(0b1111000110, 10);

    // let mut enc = PackageMergeEncoder::new();
    // enc.add_symbol(&8u32);
    // enc.add_symbol(&9u32);
    // enc.add_symbol(&10u32);
    // let lengths = enc.get_symbol_lengths(7);
    // println!("{:?}", lengths)

    // lzss_test();
    // generate();
    // encode_test();
    // prefix_test();
    // bug_test();
    // decode_test();
    // zlib_test();
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
    // let input = fs::read("save.txt").unwrap();
    let input = b"ABCDEABCD ABCDEABCD";
    let mut my_encoder = deflate::DeflateEncoder::new(deflate::BlockType::None);
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
    let input = &fs::read("save.txt").unwrap();
    // let input: Vec<u8> = repeat_n(1, 10000).collect();
    // let input = b"ABCDEABCD ABCDEABCD";
    // let input = b"AAC";
    let mut my_encoder = deflate::DeflateEncoder::new(deflate::BlockType::DynamicHuffman);
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

fn zlib_test() {
    // let input = &fs::read("save.txt").unwrap();
    let input = b"a";
    let mut my_encoder = deflate::DeflateEncoder::new(deflate::BlockType::DynamicHuffman);
    my_encoder.write_bytes(&input[..]);
    let out_bytes = zlib_encode(my_encoder);
    print_bytes(&out_bytes);

    // let mut flate2_out = vec![];
    // let mut flate2_encoder = ZlibEncoder::new(&input[..], Compression::best());
    // flate2_encoder.read_to_end(&mut flate2_out).unwrap();
    // print_bytes(&flate2_out);

    let mut zlib_deocde = ZlibDecoder::new(&out_bytes[..]);
    let mut s = String::new();
    zlib_deocde.read_to_string(&mut s).unwrap();
    println!("Out {}", s);
}

fn print_bytes(bytes: &[u8]) {
    for b in bytes {
        print!("{:08b} ", b);
    }
    println!();
}
