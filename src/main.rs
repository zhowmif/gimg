#![allow(dead_code)]

use std::{
    fs,
    io::Read,
    sync::BarrierWaitResult,
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
        huffman::{construct_canonical_tree_from_lengths, package_merge::{self, PackageMergeEncoder}, HuffmanEncoder},
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
    // let b = NewBitStream::from_u32_msb(0b11101010000000000000000000000000, 8);
    // println!("{b}");
    // huffman_test();
    package_merge_test();
}

fn package_merge_test() {
    let bee_movie_script = fs::read_to_string("save.txt").expect("Failed to read input file");
    let mut encoder = PackageMergeEncoder::new();
    for chr in bee_movie_script.chars() {
        encoder.add_symbol(&chr);
    }
    let huffman_length_values = encoder.get_symbol_lengths(7);
    let g = huffman_length_values.clone();
    let huffman_tree = construct_canonical_tree_from_lengths(huffman_length_values);
    for (chr, code) in huffman_tree.into_iter() {
        println!("{} {}", chr, code);
    }
    let mut total_size = 0;
    for chr in bee_movie_script.chars() {
        total_size += g.iter().find(|(x, _l)| *x == chr).unwrap().1;
    }

    println!(
        "Original length {}, reduced: {}",
        bee_movie_script.len() * 8,
        total_size
    );
}

fn huffman_test() {
    let bee_movie_script = fs::read_to_string("save.txt").expect("Failed to read input file");
    // let input_string = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaabccceffdgfdgrethadzxcvfkdgldffdsfdsfdsfdfdfdfdt";
    let mut huffman_encoder = HuffmanEncoder::new();
    for chr in bee_movie_script.chars() {
        huffman_encoder.add_symbol(&chr);
    }
    let huffman_length_values = huffman_encoder.get_symbol_lengths();
    let g = huffman_length_values.clone();
    let huffman_tree = construct_canonical_tree_from_lengths(huffman_length_values);
    for (chr, code) in huffman_tree.into_iter() {
        println!("{} {}", chr, code);
    }
    let mut total_size = 0;
    for chr in bee_movie_script.chars() {
        total_size += g.iter().find(|(x, _l)| *x == chr).unwrap().1;
    }

    println!(
        "Original length {}, reduced: {}",
        bee_movie_script.len() * 8,
        total_size
    );
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
