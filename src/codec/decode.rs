use crate::{
    codec::{
        HEIGHT_RANGE, MACROBLOCK_HEIGHT_RANGE, MACROBLOCK_WIDTH_RANGE, METADATA_LENGTH, WIDTH_RANGE,
    },
    colors::YCbCr,
    dct::{DiscreteCosineTransformer, NUM_DCT_SIGNALS},
    image::{Image, Resolution, MACROBLOCKS_SIZE},
};

use super::Vec4;

pub fn decode_image(bytes: &[u8], dct: &DiscreteCosineTransformer) -> Image {
    let width = u32::from_be_bytes(bytes[WIDTH_RANGE].try_into().unwrap()) as usize;
    let height = u32::from_be_bytes(bytes[HEIGHT_RANGE].try_into().unwrap()) as usize;
    let mut pixels: Vec<Vec<YCbCr>> = Vec::with_capacity(height);
    let macroblock_width =
        u32::from_be_bytes(bytes[MACROBLOCK_WIDTH_RANGE].try_into().unwrap()) as usize;
    let macroblock_height =
        u32::from_be_bytes(bytes[MACROBLOCK_HEIGHT_RANGE].try_into().unwrap()) as usize;
    let channel_size =
        macroblock_height * macroblock_width * (NUM_DCT_SIGNALS * NUM_DCT_SIGNALS + 4);
    let luma_channel_bytes = &bytes[METADATA_LENGTH..METADATA_LENGTH + channel_size];
    let luma_channel = decode_channel(luma_channel_bytes, dct, macroblock_width, macroblock_height);
    println!("luma done!");
    let cb_channel_bytes =
        &bytes[METADATA_LENGTH + channel_size..METADATA_LENGTH + channel_size * 2];
    let cb_channel = decode_channel(cb_channel_bytes, dct, macroblock_width, macroblock_height);
    let cr_channel_bytes =
        &bytes[METADATA_LENGTH + channel_size * 2..METADATA_LENGTH + channel_size * 3];
    println!("cb done!");
    let cr_channel = decode_channel(cr_channel_bytes, dct, macroblock_width, macroblock_height);
    println!("cr done!");

    for row in 0..height {
        pixels.push(Vec::with_capacity(width));

        for col in 0..width {
            let mr = row / MACROBLOCKS_SIZE;
            let mc = col / MACROBLOCKS_SIZE;
            let row_in_mb = row % MACROBLOCKS_SIZE;
            let col_in_mb = col % MACROBLOCKS_SIZE;

            let y = luma_channel[mr][mc][row_in_mb][col_in_mb];
            let cb = cb_channel[mr][mc][row_in_mb][col_in_mb];
            let cr = cr_channel[mr][mc][row_in_mb][col_in_mb];

            pixels[row].push(YCbCr::new(y, cb, cr));
        }
    }

    println!("image creation done!");
    Image::new(Resolution { width, height }, pixels)
}

fn decode_channel(
    channel_bytes: &[u8],
    dct: &DiscreteCosineTransformer,
    macroblock_width: usize,
    macroblock_height: usize,
) -> Vec4<u8> {
    let mut result: Vec4<u8> = Vec::with_capacity(macroblock_height);
    let mut src_counter = 0;

    for mr in 0..macroblock_height {
        result.push(Vec::with_capacity(macroblock_width));

        for _mc in 0..macroblock_width {
            let normalization_factor = f32::from_be_bytes(
                channel_bytes[src_counter..src_counter + 4]
                    .try_into()
                    .unwrap(),
            );
            src_counter += 4;

            let macroblock_bytes =
                &channel_bytes[src_counter..src_counter + NUM_DCT_SIGNALS * NUM_DCT_SIGNALS];
            src_counter += NUM_DCT_SIGNALS * NUM_DCT_SIGNALS;
            let normalized_amplitudes: Vec<Vec<i8>> = macroblock_bytes
                .chunks(NUM_DCT_SIGNALS)
                .map(|chunk| chunk.iter().map(|amp| amp.clone() as i8).collect())
                .collect();
            let amplitudes = DiscreteCosineTransformer::inverse_normalization(
                &normalized_amplitudes,
                normalization_factor,
            );
            let pixel_values = dct.idct(&amplitudes);
            result[mr].push(pixel_values);
        }
    }

    result
}
