#![allow(dead_code)]

mod algebra;
mod bits;
mod codec;
mod colors;
mod dct;
mod demuxers;
mod ffmpeg;
mod filters;
mod image;
mod muxers;
mod pixel_formats;
mod queue;
mod stream;
mod tree;

use codec::{decode::decode_image, encode::encode_image};
use demuxers::{image_demuxer::ImageDemuxer, raw_image_demuxer::RawImageDemuxer};
use muxers::{show_muxer::ShowMuxer, Muxer};
use std::fs;
use stream::Stream;

const INPUT_FILE: &str = "files/input.jpg";
const RGB_FILE: &str = "files/raw.rgb";
const OUTPUT_FILE: &str = "files/out.jpg";

fn main() {
    // encode_test();
    decode_test();
}

fn encode_test() {
    let dct = dct::DiscreteCosineTransformer::new();
    let mut image_demuxer = ImageDemuxer::new("files/mountain.png", "rgb24");
    let img = image_demuxer.get_next_image().unwrap();
    let encoded = encode_image(img, &dct);
    fs::write("files/encoded.guy", encoded).expect("Failde writing guy");
}

fn decode_test() {
    let dct = dct::DiscreteCosineTransformer::new();
    let bytes = fs::read("files/encoded.guy").unwrap();
    let img = decode_image(&bytes, &dct);
    let dx = RawImageDemuxer::new(img);
    let mx = ShowMuxer::new("rgb24");
    mx.consume_stream(dx);
}
