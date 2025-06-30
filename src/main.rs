#![allow(dead_code)]

use std::fs;

use colors::YCbCr;
use demuxers::raw_image_demuxer::RawImageDemuxer;
use image::{Image, Resolution};
use muxers::{show_muxer::ShowMuxer, Muxer};
use png::{decode_png, encode_png, PartialPngConfig};

mod algebra;
mod binary;
mod codec;
mod colors;
mod dct;
mod demuxers;
mod filters;
mod guy_format;
mod image;
mod muxers;
mod pixel_formats;
mod png;
mod ppm;
mod quantization;
mod queue;
mod stream;
mod tree;

fn main() {
    png_encode_test();
    // png_decode_test();
    // deflate_test();
}

fn png_encode_test() {
    // let mut dx = ImageDemuxer::new("files/drawing.png", "rgb24");
    // let img = dx.get_next_image().unwrap();
    //
    // let mut rgba_pixels: Vec<Vec<RGBA>> = Vec::with_capacity(img.resolution.height);
    // for row in img.pixels {
    //     let mut pixel_row: Vec<RGBA> = Vec::with_capacity(img.resolution.width);
    //
    //     for pixel in row {
    //         pixel_row.push(pixel.into());
    //     }
    //
    //     rgba_pixels.push(pixel_row);
    // }

    let png_file = fs::read("files/mountain.png").unwrap();
    let rgba_pixels = decode_png(&png_file).unwrap();

    let config = PartialPngConfig::new()
        .color_type(png::ColorType::Truecolor)
        .compression_level(png::CompressionLevel::Fast);
    let png_bytes = encode_png(rgba_pixels, config);
    fs::write("files/mymountain.png", png_bytes).expect("Failed to write my png");
}

fn png_decode_test() {
    let png_file = fs::read("files/drawing.png").unwrap();
    let decoded_png = decode_png(&png_file).unwrap();
    let ycbcr_pixels: Vec<Vec<YCbCr>> = decoded_png
        .into_iter()
        .map(|row| row.into_iter().map(|pixel| YCbCr::from(pixel)).collect())
        .collect();

    let img = Image::new(Resolution::from_vec(&ycbcr_pixels), ycbcr_pixels);
    let dx = RawImageDemuxer::new(img);
    let show = ShowMuxer::new("rgb24");
    show.consume_stream(dx);
}

fn deflate_test() {
    // let input = &fs::read("files/mountaindata.bin").unwrap();
    // let deflate_data = &input[2..][..input.len() - 4];

    // let mut buf = Vec::new();
    // let mut deflate_decoder = DeflateDecoder::new(deflate_data);
    // deflate_decoder.read_to_end(&mut buf).unwrap();
    // println!("Deflate uncompressed: {:?}", buf.len());
    //
    // let my_data = decode_deflate(deflate_data).unwrap();
    // println!("My data {}", my_data.len());

    // let mut zlib_decoder = ZlibDecoder::new(&input[..]);
    // zlib_decoder.read_to_end(&mut buf).unwrap();
    // println!("Decoded {:?}", &buf[..10]);

    // let uncompressed = decode_zlib(&input).unwrap();
    // println!("uncompressed: {}", uncompressed.len());

    // let input: Vec<u8> = repeat_n(1, 10000).collect();
    // let input = b"ABCDEABCD ABCDEABCD";
    // let input = b"AAC";
    // let mut my_encoder = deflate::DeflateEncoder::new(deflate::DeflateBlockType::DynamicHuffman);
    // my_encoder.write_bytes(&input[..]);
    // let mut out = my_encoder.finish();
    // let mut out_bytes = out.flush_to_bytes();
    // print_bytes(&out_bytes[..10]);
    // out_bytes[0] -= 8;
    // print_bytes(&out_bytes[..10]);

    // let mut flate2_encoder = DeflateEncoder::new(&input[..], Compression::best());
    // let mut out_bytes = Vec::new();
    // flate2_encoder.read_to_end(&mut out_bytes).unwrap();
    // print_bytes(&out_bytes);

    // print!("bytes ");
    // print_bytes(&out_bytes);

    // let mut decode = DeflateDecoder::new(&out_bytes[..]);
    // let mut out = Vec::new();
    // decode.read_to_end(&mut out).unwrap();
    // println!("flate2 out {:?}", String::from_utf8(out).unwrap());
    //
    // let _decoded = decode_deflate(&out_bytes).unwrap();
    // println!("my out {:?}", String::from_utf8(decoded).unwrap());
}

fn print_bytes(bytes: &[u8]) {
    for b in bytes {
        print!("{:08b} ", b);
    }
    println!();
}
