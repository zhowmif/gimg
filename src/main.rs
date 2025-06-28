#![allow(dead_code)]

use std::fs;

use colors::{YCbCr, RGBA};
use demuxers::{image_demuxer::ImageDemuxer, raw_image_demuxer::RawImageDemuxer};
use image::{Image, Resolution};
use muxers::{show_muxer::ShowMuxer, Muxer};
use png::{
    decode_png,
    deflate::{self, decode::decode_deflate},
    encode_png, InterlaceMethod,
};
use stream::Stream;

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
    adam7_test();
    // png_encode_test(None, None, Some(InterlaceMethod::Adam7));
    // png_decode_test();
}

fn adam7_test() {
    let mut pixels: Vec<Vec<RGBA>> = Vec::new();

    //problem occurs when width>height
    let height = 10;
    let width = 12;
    let mut n: u64 = 0;
    for i in 0..height {
        let mut row = Vec::new();
        // let r = i as f32 / height as f32 * 255.;
        for j in 0..width {
            let r = i * width + j;
            // let g = j as f32 / width as f32 * 255.;
            let g = 0;
            row.push(RGBA::new(r as u8, g as u8, 0, 255));
            n += 1;
        }

        pixels.push(row);
    }

    // let png_bytes = encode_png(pixels, None, None, Some(InterlaceMethod::Adam7));
    // fs::write("files/goop.png", png_bytes).expect("Failed to write my png");

    let interlace = InterlaceMethod::Adam7;
    let reduced_images = interlace.perform_pass_extraction(pixels);
    let reconstructed_image = interlace.deinterlace_image(reduced_images, height, width);

    for pass in reconstructed_image {
        println!("[");
        for row in pass {
            println!("  {:?}", row);
        }
        println!("[");
    }
}

fn png_encode_test(
    color_type: Option<png::ColorType>,
    bit_depth: Option<u8>,
    interlace_method: Option<png::InterlaceMethod>,
) {
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

    let png_bytes = encode_png(rgba_pixels, color_type, bit_depth, interlace_method);
    fs::write("files/mymountain.png", png_bytes).expect("Failed to write my png");
}

fn png_decode_test() {
    let png_file = fs::read("files/mymountain.png").unwrap();
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
    let input = &fs::read("save.txt").unwrap();
    // let input: Vec<u8> = repeat_n(1, 10000).collect();
    // let input = b"ABCDEABCD ABCDEABCD";
    // let input = b"AAC";
    let mut my_encoder = deflate::DeflateEncoder::new(deflate::DeflateBlockType::DynamicHuffman);
    my_encoder.write_bytes(&input[..]);
    let mut out = my_encoder.finish();
    let mut out_bytes = out.flush_to_bytes();
    print_bytes(&out_bytes[..10]);
    out_bytes[0] -= 8;
    print_bytes(&out_bytes[..10]);

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
    let decoded = decode_deflate(&out_bytes).unwrap();
    // println!("my out {:?}", String::from_utf8(decoded).unwrap());
}

fn print_bytes(bytes: &[u8]) {
    for b in bytes {
        print!("{:08b} ", b);
    }
    println!();
}
