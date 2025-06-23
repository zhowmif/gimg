#![allow(dead_code)]

use std::{collections::HashMap, fs, io::Read, iter::repeat_n};

use colors::RGBA;
use demuxers::image_demuxer::ImageDemuxer;
use flate2::{
    read::{DeflateDecoder, DeflateEncoder},
    Compression,
};
use png::{
    deflate::{
        self,
        decode::{decode_compressed_block, decode_deflate},
        huffman::{
            calc_kraft_mcmillen_value, construct_canonical_tree_from_lengths,
            package_merge::PackageMergeEncoder,
        },
        lzss::LzssSymbol,
        new_bitsream::{BitStreamReader, NewBitStream},
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

    // print!("bytes ");
    // print_bytes(&out_bytes);

    let mut decode = DeflateDecoder::new(&out_bytes[..]);
    let mut out = Vec::new();
    decode.read_to_end(&mut out).unwrap();
    println!("flate2 out {:?}", String::from_utf8(out).unwrap());

    // let decoded = decode_deflate(&out_bytes);
    // println!("my out {:?}", String::from_utf8(decoded).unwrap());
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
    // let input = fs::read("save.txt").expect("failed to read input file");
    // let lzss_encoded = encode_lzss(&input, 1000);
    // let decoded_bytes = decode_lzss(&lzss_encoded);
    // let decoded = str::from_utf8(&decoded_bytes).expect("Lzss decode produced invalid utf8");
    // println!("is good {}", input == decoded_bytes);
}

fn prefix_test() {
    let lengths: HashMap<u32, u32> = [].into_iter().collect();
    let tree = construct_canonical_tree_from_lengths(&lengths);
    // is_tree_prefix_free(&tree);
    for (s, c) in tree {
        if s == 263 || s == 78 {
            println!("{s} {c}");
        }
    }
}

fn is_tree_prefix_free(tree: &HashMap<u32, NewBitStream>) {
    let v: Vec<(u32, NewBitStream)> = tree.clone().into_iter().collect();
    let mut is_prefix_free = true;
    println!("{} and {}", tree.get(&85).unwrap(), tree.get(&271).unwrap());
    // println!("tree {:?}", tree);

    for i in 0..v.len() {
        for j in (i + 1)..v.len() {
            // println!("Cmping {} {}", v[i].1, v[j].1);
            if is_prefix(&v[i].1, &v[j].1) {
                is_prefix_free = false;
                println!(
                    "({}, {}) is a prefix of ({}, {})",
                    v[i].0, v[i].1, v[j].0, v[j].1
                );
            }
        }
    }

    println!("is prefix free - {is_prefix_free}");
}

fn is_prefix(a: &NewBitStream, b: &NewBitStream) -> bool {
    let a_str = a.to_string();
    let b_str = b.to_string();

    b_str.starts_with(&a_str)
}

fn bug_test() {
    let encode_ll_codes: HashMap<u16, NewBitStream> = vec![
        (
            272,
            NewBitStream {
                stream: vec![255],
                working_byte: 208,
                current_bit_number: 4,
            },
        ),
        (
            105,
            NewBitStream {
                stream: vec![],
                working_byte: 180,
                current_bit_number: 6,
            },
        ),
        (
            53,
            NewBitStream {
                stream: vec![255],
                working_byte: 160,
                current_bit_number: 4,
            },
        ),
        (
            260,
            NewBitStream {
                stream: vec![],
                working_byte: 16,
                current_bit_number: 4,
            },
        ),
        (
            87,
            NewBitStream {
                stream: vec![191],
                working_byte: 128,
                current_bit_number: 2,
            },
        ),
        (
            120,
            NewBitStream {
                stream: vec![191],
                working_byte: 192,
                current_bit_number: 2,
            },
        ),
        (
            97,
            NewBitStream {
                stream: vec![],
                working_byte: 212,
                current_bit_number: 6,
            },
        ),
        (
            89,
            NewBitStream {
                stream: vec![255],
                working_byte: 0,
                current_bit_number: 3,
            },
        ),
        (
            102,
            NewBitStream {
                stream: vec![],
                working_byte: 182,
                current_bit_number: 7,
            },
        ),
        (
            58,
            NewBitStream {
                stream: vec![255],
                working_byte: 216,
                current_bit_number: 5,
            },
        ),
        (
            46,
            NewBitStream {
                stream: vec![55],
                working_byte: 0,
                current_bit_number: 0,
            },
        ),
        (
            44,
            NewBitStream {
                stream: vec![87],
                working_byte: 0,
                current_bit_number: 0,
            },
        ),
        (
            79,
            NewBitStream {
                stream: vec![119],
                working_byte: 0,
                current_bit_number: 0,
            },
        ),
        (
            45,
            NewBitStream {
                stream: vec![215],
                working_byte: 0,
                current_bit_number: 0,
            },
        ),
        (
            68,
            NewBitStream {
                stream: vec![223],
                working_byte: 128,
                current_bit_number: 2,
            },
        ),
        (
            78,
            NewBitStream {
                stream: vec![63],
                working_byte: 64,
                current_bit_number: 2,
            },
        ),
        (
            50,
            NewBitStream {
                stream: vec![255],
                working_byte: 64,
                current_bit_number: 4,
            },
        ),
        (
            262,
            NewBitStream {
                stream: vec![],
                working_byte: 40,
                current_bit_number: 5,
            },
        ),
        (
            259,
            NewBitStream {
                stream: vec![],
                working_byte: 192,
                current_bit_number: 3,
            },
        ),
        (
            103,
            NewBitStream {
                stream: vec![143],
                working_byte: 0,
                current_bit_number: 0,
            },
        ),
        (
            70,
            NewBitStream {
                stream: vec![239],
                working_byte: 0,
                current_bit_number: 1,
            },
        ),
        (
            111,
            NewBitStream {
                stream: vec![],
                working_byte: 244,
                current_bit_number: 6,
            },
        ),
        (
            104,
            NewBitStream {
                stream: vec![],
                working_byte: 118,
                current_bit_number: 7,
            },
        ),
        (
            267,
            NewBitStream {
                stream: vec![95],
                working_byte: 0,
                current_bit_number: 1,
            },
        ),
        (
            277,
            NewBitStream {
                stream: vec![255],
                working_byte: 120,
                current_bit_number: 5,
            },
        ),
        (
            52,
            NewBitStream {
                stream: vec![255],
                working_byte: 32,
                current_bit_number: 4,
            },
        ),
        (
            85,
            NewBitStream {
                stream: vec![127],
                working_byte: 96,
                current_bit_number: 3,
            },
        ),
        (
            34,
            NewBitStream {
                stream: vec![95],
                working_byte: 64,
                current_bit_number: 2,
            },
        ),
        (
            108,
            NewBitStream {
                stream: vec![],
                working_byte: 246,
                current_bit_number: 7,
            },
        ),
        (
            57,
            NewBitStream {
                stream: vec![255],
                working_byte: 88,
                current_bit_number: 5,
            },
        ),
        (
            54,
            NewBitStream {
                stream: vec![255],
                working_byte: 124,
                current_bit_number: 6,
            },
        ),
        (
            49,
            NewBitStream {
                stream: vec![127],
                working_byte: 32,
                current_bit_number: 3,
            },
        ),
        (
            33,
            NewBitStream {
                stream: vec![175],
                working_byte: 0,
                current_bit_number: 1,
            },
        ),
        (
            258,
            NewBitStream {
                stream: vec![],
                working_byte: 64,
                current_bit_number: 3,
            },
        ),
        (
            81,
            NewBitStream {
                stream: vec![255],
                working_byte: 252,
                current_bit_number: 6,
            },
        ),
        (
            100,
            NewBitStream {
                stream: vec![],
                working_byte: 54,
                current_bit_number: 7,
            },
        ),
        (
            74,
            NewBitStream {
                stream: vec![223],
                working_byte: 192,
                current_bit_number: 2,
            },
        ),
        (
            51,
            NewBitStream {
                stream: vec![255],
                working_byte: 192,
                current_bit_number: 4,
            },
        ),
        (
            55,
            NewBitStream {
                stream: vec![255],
                working_byte: 96,
                current_bit_number: 4,
            },
        ),
        (
            270,
            NewBitStream {
                stream: vec![255],
                working_byte: 144,
                current_bit_number: 4,
            },
        ),
        (
            10,
            NewBitStream {
                stream: vec![],
                working_byte: 86,
                current_bit_number: 7,
            },
        ),
        (
            75,
            NewBitStream {
                stream: vec![63],
                working_byte: 0,
                current_bit_number: 2,
            },
        ),
        (
            101,
            NewBitStream {
                stream: vec![],
                working_byte: 52,
                current_bit_number: 6,
            },
        ),
        (
            112,
            NewBitStream {
                stream: vec![],
                working_byte: 142,
                current_bit_number: 7,
            },
        ),
        (
            116,
            NewBitStream {
                stream: vec![],
                working_byte: 78,
                current_bit_number: 7,
            },
        ),
        (
            73,
            NewBitStream {
                stream: vec![183],
                working_byte: 0,
                current_bit_number: 0,
            },
        ),
        (
            107,
            NewBitStream {
                stream: vec![79],
                working_byte: 0,
                current_bit_number: 0,
            },
        ),
        (
            82,
            NewBitStream {
                stream: vec![191],
                working_byte: 0,
                current_bit_number: 2,
            },
        ),
        (
            83,
            NewBitStream {
                stream: vec![247],
                working_byte: 0,
                current_bit_number: 0,
            },
        ),
        (
            117,
            NewBitStream {
                stream: vec![],
                working_byte: 206,
                current_bit_number: 7,
            },
        ),
        (
            66,
            NewBitStream {
                stream: vec![111],
                working_byte: 128,
                current_bit_number: 1,
            },
        ),
        (
            39,
            NewBitStream {
                stream: vec![95],
                working_byte: 192,
                current_bit_number: 2,
            },
        ),
        (
            65,
            NewBitStream {
                stream: vec![111],
                working_byte: 0,
                current_bit_number: 1,
            },
        ),
        (
            106,
            NewBitStream {
                stream: vec![191],
                working_byte: 64,
                current_bit_number: 2,
            },
        ),
        (
            265,
            NewBitStream {
                stream: vec![],
                working_byte: 44,
                current_bit_number: 6,
            },
        ),
        (
            115,
            NewBitStream {
                stream: vec![],
                working_byte: 140,
                current_bit_number: 6,
            },
        ),
        (
            86,
            NewBitStream {
                stream: vec![127],
                working_byte: 224,
                current_bit_number: 3,
            },
        ),
        (
            118,
            NewBitStream {
                stream: vec![159],
                working_byte: 0,
                current_bit_number: 1,
            },
        ),
        (
            263,
            NewBitStream {
                stream: vec![],
                working_byte: 76,
                current_bit_number: 6,
            },
        ),
        (
            113,
            NewBitStream {
                stream: vec![255],
                working_byte: 16,
                current_bit_number: 4,
            },
        ),
        (
            110,
            NewBitStream {
                stream: vec![],
                working_byte: 116,
                current_bit_number: 6,
            },
        ),
        (
            63,
            NewBitStream {
                stream: vec![175],
                working_byte: 128,
                current_bit_number: 1,
            },
        ),
        (
            122,
            NewBitStream {
                stream: vec![159],
                working_byte: 128,
                current_bit_number: 1,
            },
        ),
        (
            114,
            NewBitStream {
                stream: vec![],
                working_byte: 12,
                current_bit_number: 6,
            },
        ),
        (
            56,
            NewBitStream {
                stream: vec![255],
                working_byte: 224,
                current_bit_number: 4,
            },
        ),
        (
            109,
            NewBitStream {
                stream: vec![],
                working_byte: 14,
                current_bit_number: 7,
            },
        ),
        (
            48,
            NewBitStream {
                stream: vec![223],
                working_byte: 0,
                current_bit_number: 2,
            },
        ),
        (
            271,
            NewBitStream {
                stream: vec![255],
                working_byte: 80,
                current_bit_number: 4,
            },
        ),
        (
            80,
            NewBitStream {
                stream: vec![63],
                working_byte: 192,
                current_bit_number: 2,
            },
        ),
        (
            274,
            NewBitStream {
                stream: vec![255],
                working_byte: 48,
                current_bit_number: 4,
            },
        ),
        (
            119,
            NewBitStream {
                stream: vec![207],
                working_byte: 0,
                current_bit_number: 0,
            },
        ),
        (
            121,
            NewBitStream {
                stream: vec![],
                working_byte: 46,
                current_bit_number: 7,
            },
        ),
        (
            264,
            NewBitStream {
                stream: vec![],
                working_byte: 204,
                current_bit_number: 6,
            },
        ),
        (
            98,
            NewBitStream {
                stream: vec![15],
                working_byte: 0,
                current_bit_number: 0,
            },
        ),
        (
            69,
            NewBitStream {
                stream: vec![127],
                working_byte: 160,
                current_bit_number: 3,
            },
        ),
        (
            261,
            NewBitStream {
                stream: vec![],
                working_byte: 144,
                current_bit_number: 4,
            },
        ),
        (
            84,
            NewBitStream {
                stream: vec![31],
                working_byte: 128,
                current_bit_number: 1,
            },
        ),
        (
            71,
            NewBitStream {
                stream: vec![223],
                working_byte: 64,
                current_bit_number: 2,
            },
        ),
        (
            77,
            NewBitStream {
                stream: vec![31],
                working_byte: 0,
                current_bit_number: 1,
            },
        ),
        (
            32,
            NewBitStream {
                stream: vec![],
                working_byte: 84,
                current_bit_number: 6,
            },
        ),
        (
            76,
            NewBitStream {
                stream: vec![63],
                working_byte: 128,
                current_bit_number: 2,
            },
        ),
        (
            257,
            NewBitStream {
                stream: vec![],
                working_byte: 0,
                current_bit_number: 2,
            },
        ),
        (
            268,
            NewBitStream {
                stream: vec![127],
                working_byte: 0,
                current_bit_number: 2,
            },
        ),
        (
            273,
            NewBitStream {
                stream: vec![255],
                working_byte: 184,
                current_bit_number: 5,
            },
        ),
        (
            256,
            NewBitStream {
                stream: vec![255],
                working_byte: 56,
                current_bit_number: 5,
            },
        ),
        (
            266,
            NewBitStream {
                stream: vec![47],
                working_byte: 0,
                current_bit_number: 0,
            },
        ),
        (
            269,
            NewBitStream {
                stream: vec![127],
                working_byte: 128,
                current_bit_number: 2,
            },
        ),
        (
            99,
            NewBitStream {
                stream: vec![],
                working_byte: 214,
                current_bit_number: 7,
            },
        ),
        (
            72,
            NewBitStream {
                stream: vec![239],
                working_byte: 128,
                current_bit_number: 1,
            },
        ),
    ]
    .into_iter()
    .collect();
    let decode_ll_codes: HashMap<NewBitStream, u16> = vec![
        (
            NewBitStream {
                stream: vec![255],
                working_byte: 48,
                current_bit_number: 4,
            },
            274,
        ),
        (
            NewBitStream {
                stream: vec![191],
                working_byte: 128,
                current_bit_number: 2,
            },
            87,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 118,
                current_bit_number: 7,
            },
            104,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 86,
                current_bit_number: 7,
            },
            10,
        ),
        (
            NewBitStream {
                stream: vec![255],
                working_byte: 120,
                current_bit_number: 5,
            },
            277,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 182,
                current_bit_number: 7,
            },
            102,
        ),
        (
            NewBitStream {
                stream: vec![47],
                working_byte: 0,
                current_bit_number: 0,
            },
            266,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 52,
                current_bit_number: 6,
            },
            101,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 144,
                current_bit_number: 4,
            },
            261,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 84,
                current_bit_number: 6,
            },
            32,
        ),
        (
            NewBitStream {
                stream: vec![207],
                working_byte: 0,
                current_bit_number: 0,
            },
            119,
        ),
        (
            NewBitStream {
                stream: vec![215],
                working_byte: 0,
                current_bit_number: 0,
            },
            45,
        ),
        (
            NewBitStream {
                stream: vec![63],
                working_byte: 192,
                current_bit_number: 2,
            },
            80,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 40,
                current_bit_number: 5,
            },
            262,
        ),
        (
            NewBitStream {
                stream: vec![255],
                working_byte: 184,
                current_bit_number: 5,
            },
            273,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 76,
                current_bit_number: 6,
            },
            263,
        ),
        (
            NewBitStream {
                stream: vec![95],
                working_byte: 64,
                current_bit_number: 2,
            },
            34,
        ),
        (
            NewBitStream {
                stream: vec![183],
                working_byte: 0,
                current_bit_number: 0,
            },
            73,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 192,
                current_bit_number: 3,
            },
            259,
        ),
        (
            NewBitStream {
                stream: vec![255],
                working_byte: 80,
                current_bit_number: 4,
            },
            271,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 0,
                current_bit_number: 2,
            },
            257,
        ),
        (
            NewBitStream {
                stream: vec![255],
                working_byte: 216,
                current_bit_number: 5,
            },
            58,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 212,
                current_bit_number: 6,
            },
            97,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 12,
                current_bit_number: 6,
            },
            114,
        ),
        (
            NewBitStream {
                stream: vec![95],
                working_byte: 192,
                current_bit_number: 2,
            },
            39,
        ),
        (
            NewBitStream {
                stream: vec![111],
                working_byte: 0,
                current_bit_number: 1,
            },
            65,
        ),
        (
            NewBitStream {
                stream: vec![95],
                working_byte: 0,
                current_bit_number: 1,
            },
            267,
        ),
        (
            NewBitStream {
                stream: vec![159],
                working_byte: 128,
                current_bit_number: 1,
            },
            122,
        ),
        (
            NewBitStream {
                stream: vec![255],
                working_byte: 208,
                current_bit_number: 4,
            },
            272,
        ),
        (
            NewBitStream {
                stream: vec![127],
                working_byte: 32,
                current_bit_number: 3,
            },
            49,
        ),
        (
            NewBitStream {
                stream: vec![255],
                working_byte: 64,
                current_bit_number: 4,
            },
            50,
        ),
        (
            NewBitStream {
                stream: vec![255],
                working_byte: 96,
                current_bit_number: 4,
            },
            55,
        ),
        (
            NewBitStream {
                stream: vec![111],
                working_byte: 128,
                current_bit_number: 1,
            },
            66,
        ),
        (
            NewBitStream {
                stream: vec![255],
                working_byte: 32,
                current_bit_number: 4,
            },
            52,
        ),
        (
            NewBitStream {
                stream: vec![255],
                working_byte: 252,
                current_bit_number: 6,
            },
            81,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 44,
                current_bit_number: 6,
            },
            265,
        ),
        (
            NewBitStream {
                stream: vec![31],
                working_byte: 0,
                current_bit_number: 1,
            },
            77,
        ),
        (
            NewBitStream {
                stream: vec![255],
                working_byte: 144,
                current_bit_number: 4,
            },
            270,
        ),
        (
            NewBitStream {
                stream: vec![255],
                working_byte: 16,
                current_bit_number: 4,
            },
            113,
        ),
        (
            NewBitStream {
                stream: vec![255],
                working_byte: 224,
                current_bit_number: 4,
            },
            56,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 214,
                current_bit_number: 7,
            },
            99,
        ),
        (
            NewBitStream {
                stream: vec![127],
                working_byte: 224,
                current_bit_number: 3,
            },
            86,
        ),
        (
            NewBitStream {
                stream: vec![223],
                working_byte: 192,
                current_bit_number: 2,
            },
            74,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 116,
                current_bit_number: 6,
            },
            110,
        ),
        (
            NewBitStream {
                stream: vec![159],
                working_byte: 0,
                current_bit_number: 1,
            },
            118,
        ),
        (
            NewBitStream {
                stream: vec![223],
                working_byte: 0,
                current_bit_number: 2,
            },
            48,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 204,
                current_bit_number: 6,
            },
            264,
        ),
        (
            NewBitStream {
                stream: vec![223],
                working_byte: 128,
                current_bit_number: 2,
            },
            68,
        ),
        (
            NewBitStream {
                stream: vec![255],
                working_byte: 124,
                current_bit_number: 6,
            },
            54,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 14,
                current_bit_number: 7,
            },
            109,
        ),
        (
            NewBitStream {
                stream: vec![15],
                working_byte: 0,
                current_bit_number: 0,
            },
            98,
        ),
        (
            NewBitStream {
                stream: vec![55],
                working_byte: 0,
                current_bit_number: 0,
            },
            46,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 54,
                current_bit_number: 7,
            },
            100,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 142,
                current_bit_number: 7,
            },
            112,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 64,
                current_bit_number: 3,
            },
            258,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 244,
                current_bit_number: 6,
            },
            111,
        ),
        (
            NewBitStream {
                stream: vec![31],
                working_byte: 128,
                current_bit_number: 1,
            },
            84,
        ),
        (
            NewBitStream {
                stream: vec![127],
                working_byte: 160,
                current_bit_number: 3,
            },
            69,
        ),
        (
            NewBitStream {
                stream: vec![191],
                working_byte: 0,
                current_bit_number: 2,
            },
            82,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 140,
                current_bit_number: 6,
            },
            115,
        ),
        (
            NewBitStream {
                stream: vec![255],
                working_byte: 56,
                current_bit_number: 5,
            },
            256,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 180,
                current_bit_number: 6,
            },
            105,
        ),
        (
            NewBitStream {
                stream: vec![143],
                working_byte: 0,
                current_bit_number: 0,
            },
            103,
        ),
        (
            NewBitStream {
                stream: vec![127],
                working_byte: 128,
                current_bit_number: 2,
            },
            269,
        ),
        (
            NewBitStream {
                stream: vec![239],
                working_byte: 128,
                current_bit_number: 1,
            },
            72,
        ),
        (
            NewBitStream {
                stream: vec![255],
                working_byte: 88,
                current_bit_number: 5,
            },
            57,
        ),
        (
            NewBitStream {
                stream: vec![87],
                working_byte: 0,
                current_bit_number: 0,
            },
            44,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 206,
                current_bit_number: 7,
            },
            117,
        ),
        (
            NewBitStream {
                stream: vec![119],
                working_byte: 0,
                current_bit_number: 0,
            },
            79,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 46,
                current_bit_number: 7,
            },
            121,
        ),
        (
            NewBitStream {
                stream: vec![63],
                working_byte: 128,
                current_bit_number: 2,
            },
            76,
        ),
        (
            NewBitStream {
                stream: vec![127],
                working_byte: 96,
                current_bit_number: 3,
            },
            85,
        ),
        (
            NewBitStream {
                stream: vec![223],
                working_byte: 64,
                current_bit_number: 2,
            },
            71,
        ),
        (
            NewBitStream {
                stream: vec![63],
                working_byte: 0,
                current_bit_number: 2,
            },
            75,
        ),
        (
            NewBitStream {
                stream: vec![255],
                working_byte: 0,
                current_bit_number: 3,
            },
            89,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 246,
                current_bit_number: 7,
            },
            108,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 16,
                current_bit_number: 4,
            },
            260,
        ),
        (
            NewBitStream {
                stream: vec![127],
                working_byte: 0,
                current_bit_number: 2,
            },
            268,
        ),
        (
            NewBitStream {
                stream: vec![191],
                working_byte: 64,
                current_bit_number: 2,
            },
            106,
        ),
        (
            NewBitStream {
                stream: vec![239],
                working_byte: 0,
                current_bit_number: 1,
            },
            70,
        ),
        (
            NewBitStream {
                stream: vec![255],
                working_byte: 160,
                current_bit_number: 4,
            },
            53,
        ),
        (
            NewBitStream {
                stream: vec![175],
                working_byte: 0,
                current_bit_number: 1,
            },
            33,
        ),
        (
            NewBitStream {
                stream: vec![79],
                working_byte: 0,
                current_bit_number: 0,
            },
            107,
        ),
        (
            NewBitStream {
                stream: vec![],
                working_byte: 78,
                current_bit_number: 7,
            },
            116,
        ),
        (
            NewBitStream {
                stream: vec![175],
                working_byte: 128,
                current_bit_number: 1,
            },
            63,
        ),
        (
            NewBitStream {
                stream: vec![255],
                working_byte: 192,
                current_bit_number: 4,
            },
            51,
        ),
        (
            NewBitStream {
                stream: vec![247],
                working_byte: 0,
                current_bit_number: 0,
            },
            83,
        ),
        (
            NewBitStream {
                stream: vec![63],
                working_byte: 64,
                current_bit_number: 2,
            },
            78,
        ),
        (
            NewBitStream {
                stream: vec![191],
                working_byte: 192,
                current_bit_number: 2,
            },
            120,
        ),
    ]
    .into_iter()
    .collect();

    let dec_ll_codes_rev: HashMap<_, _> = decode_ll_codes
        .clone()
        .into_iter()
        .map(|(k, v)| (v, k))
        .collect();

    // for (k, v) in encode_ll_codes.clone() {
    //     print!("({k}, {v})");
    // }

    let mut target = NewBitStream::new();
    let lzzs_stream = vec![LzssSymbol::Literal(65), LzssSymbol::EndOfBlock];
    let dist_codes = HashMap::new();
    let dist_codes_rev = HashMap::new();
    deflate::DeflateEncoder::encode_lzss_stream(
        &lzzs_stream,
        &encode_ll_codes,
        &dist_codes,
        &mut target,
    );
    let mut dec_target = vec![];
    let to_bytes = target.flush_to_bytes();
    print_bytes(&to_bytes);
    let mut reader = BitStreamReader::new(&to_bytes);
    decode_compressed_block(
        &mut reader,
        &mut dec_target,
        &decode_ll_codes,
        &dist_codes_rev,
    );
}
//(81, 11111111111111)(73, 11101101)(58, 1111111111011)(106, 1111110110)(54, 11111111111110)(82, 1111110100)(102, 1101101)(52, 111111110100)(257, 00)(79, 11101110)(262, 10100)(108, 1101111)(75, 1111110000)(107, 11110010)(86, 11111110111)(109, 1110000)(78, 1111110010)(85, 11111110110)(57, 1111111111010)(116, 1110010)(39, 1111101011)(98, 11110000)(87, 1111110101)(50, 111111110010)(49, 11111110100)(71, 1111101110)(258, 010)(266, 11110100)(119, 11110011)(51, 111111110011)(53, 111111110101)(68, 1111101101)(271, 111111111010)(104, 1101110)(115, 110001)(114, 110000)(83, 11101111)(270, 111111111001)(69, 11111110101)(45, 11101011)(277, 1111111111110)(272, 111111111011)(105, 101101)(103, 11110001)(34, 1111101010)(259, 011)(10, 1101010)(76, 1111110001)(263, 110010)(256, 1111111111100)(70, 111101110)(99, 1101011)(33, 111101010)(44, 11101010)(55, 111111110110)(120, 1111110111)(101, 101100)(122, 111110011)(110, 101110)(261, 1001)(267, 111110100)(80, 1111110011)(63, 111101011)(269, 1111111001)(268, 1111111000)(56, 111111110111)(117, 1110011)(121, 1110100)(46, 11101100)(260, 1000)(118, 111110010)(97, 101011)(100, 1101100)(48, 1111101100)(113, 111111111000)(274, 111111111100)(77, 111110000)(273, 1111111111101)(66, 111101101)(65, 111101100)(72, 111101111)(111, 101111)(89, 11111111000)(74, 1111101111)(112, 1110001)(264, 110011)(84, 111110001)(32, 101010)(265, 110100)11110110 00000000
