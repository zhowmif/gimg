use std::io::Write;

use flate2::{write::ZlibEncoder, Compression};

pub fn compress_scanlines(scanlines: &Vec<Vec<u8>>) -> Vec<u8> {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());

    for scanline in scanlines {
        e.write_all(&scanline).expect("Deflate writing failed");
    }

    let compressed = e.finish().unwrap();

    compressed
}
