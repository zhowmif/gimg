use crate::codec::{decode::decode_image, encode::encode_image};
use crate::dct;
use crate::demuxers::{image_demuxer::ImageDemuxer, raw_image_demuxer::RawImageDemuxer};
use crate::muxers::{show_muxer::ShowMuxer, Muxer};
use crate::stream::Stream;
use std::fs;

const INPUT_FILE: &str = "files/input.jpg";
const RGB_FILE: &str = "files/raw.rgb";
const OUTPUT_FILE: &str = "files/out.jpg";

pub fn encode_test() {
    let dct = dct::DiscreteCosineTransformer::new();
    let mut image_demuxer = ImageDemuxer::new("files/mountain.png", "rgb24");
    let img = image_demuxer.get_next_image().unwrap();
    let encoded = encode_image(img, &dct);
    fs::write("files/encoded.guy", encoded).expect("Failed writing guy");
}

pub fn decode_test() {
    let dct = dct::DiscreteCosineTransformer::new();
    let bytes = fs::read("files/encoded.guy").unwrap();
    let img = decode_image(&bytes, &dct);
    let dx = RawImageDemuxer::new(img);
    let mx = ShowMuxer::new("rgb24");
    mx.consume_stream(dx);
}
