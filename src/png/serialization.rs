use crate::colors::RGBA;

pub fn scanline_to_pixels(scanlines: &Vec<Vec<u8>>) -> Vec<Vec<RGBA>> {
    let mut pixels = Vec::with_capacity(scanlines.len());

    for scanline in scanlines {
        let mut pixel_row = Vec::with_capacity(scanline.len() / 4);

        for pixel_bytes in scanline.chunks_exact(4) {
            let pixel = RGBA::new(
                pixel_bytes[0],
                pixel_bytes[1],
                pixel_bytes[2],
                pixel_bytes[3],
            );

            pixel_row.push(pixel);
        }

        pixels.push(pixel_row);
    }

    pixels
}
