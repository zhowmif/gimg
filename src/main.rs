#![allow(dead_code)]

mod algebra;
mod bits;
mod colors;
mod ffmpeg;
mod image;
mod queue;
mod tree;

use ffmpeg::{convert_img_to_rgb, convert_rgb_to_img, display_image};
use image::{Image, Resolution};
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

    convert_img_to_rgb(INPUT_FILE, RGB_FILE);
    let file = fs::read(RGB_FILE).unwrap();
    let resolution = Resolution::new(750, 1125);
    let mut image = Image::from_raw_file(resolution, file);
    //image.convert_to_grayscale();
    //image.draw_red_circle();
    image.only_keep_blue_chroma();
    image.write_raw_to_file(RGB_FILE);
    convert_rgb_to_img(RGB_FILE, OUTPUT_FILE);
    let _ = fs::remove_file(RGB_FILE);
    display_image(OUTPUT_FILE);
    let _ = fs::remove_file(OUTPUT_FILE);
}
