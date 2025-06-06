use crate::colors::RGBA;

pub fn create_scanlines(pixels: &Vec<Vec<RGBA>>) -> Vec<Vec<u8>> {
    let mut scanlines: Vec<Vec<u8>> = Vec::with_capacity(pixels.len());

    for row in pixels {
        let mut scanline: Vec<u8> = Vec::with_capacity(row.len() * 4);

        for pixel in row {
            scanline.push(pixel.r);
            scanline.push(pixel.g);
            scanline.push(pixel.b);
            scanline.push(pixel.a);
        }

        scanlines.push(scanline);
    }

    scanlines
}
