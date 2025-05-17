#![allow(dead_code)]

mod algebra;
mod bits;
mod colors;
mod dct;
mod demuxers;
mod ffmpeg;
mod filters;
mod image;
mod muxers;
mod queue;
mod stream;
mod tree;

use demuxers::{
    image_demuxer::{self, ImageDemuxer},
    Demuxer,
};
use ffmpeg::convert_img_to_rgb;
use filters::{grayscale::GrayScaleFilter, Filter};
use image::{Image, Resolution, YCbCrImage, MACROBLOCKS_SIZE};
use muxers::{show_demuxer::ShowMuxer, Muxer};
use std::fs;

const INPUT_FILE: &str = "files/input.jpg";
const RGB_FILE: &str = "files/raw.rgb";
const OUTPUT_FILE: &str = "files/out.jpg";

fn main() {
    //let s = "iibbbbbbbaaaaaaaaaaacccccccccccccccccccc";
    //let f = get_letter_frequencies(s);
    //let pq = f.to_huffman_tree();
    //let huffman_codes = pq.get_huffman_codes();
    //pq.print();
    //dbg!(huffman_codes);

    // convert_img_to_rgb(INPUT_FILE, RGB_FILE);
    // let file = fs::read(RGB_FILE).unwrap();
    // let resolution = Resolution::new(750, 1125);
    // let mut image = Image::from_raw_file(resolution, file);
    // ////////////////////////////////////////////
    // image.crop(Resolution::new(736, 1120));
    // let dct = dct::DiscreteCosineTransformer::new();
    // let ycbcr_image = YCbCrImage::from(image);
    // let x = YCbCrImage::get_cb_macroblocks(&ycbcr_image.get_macroblocks(MACROBLOCKS_SIZE));
    // let amplitudes = dct.dct(&x[10][10]);
    // println!("{:?}", x[10][10]);
    // println!("");
    // println!("{:?}", amplitudes);
    ////////////////////////////////////////////
    let image_demuxer = ImageDemuxer::new(INPUT_FILE.to_string());
    let grayscale_filter = GrayScaleFilter::filter_stream(Box::new(image_demuxer));
    let show_muxer = ShowMuxer;
    show_muxer.consume_stream(grayscale_filter);

    //image.convert_to_grayscale();
    //image.draw_red_circle();
    // image.only_keep_blue_chroma();
    // image.write_raw_to_file(RGB_FILE);
    // convert_rgb_to_img(RGB_FILE, OUTPUT_FILE);
    // let _ = fs::remove_file(RGB_FILE);
    // display_image(OUTPUT_FILE);
    // let _ = fs::remove_file(OUTPUT_FILE);
    //
}
