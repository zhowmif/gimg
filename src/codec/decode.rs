use crate::{
    codec::{
        HEIGHT_RANGE, MACROBLOCK_HEIGHT_RANGE, MACROBLOCK_WIDTH_RANGE, METADATA_LENGTH, WIDTH_RANGE,
    },
    colors::YCbCr,
    dct::DiscreteCosineTransformer,
    image::{Image, Resolution, MACROBLOCKS_SIZE},
};

use super::{rle::decode_rle, Vec4};

pub fn decode_image(bytes: &[u8], dct: &DiscreteCosineTransformer) -> Image {
    let width = u32::from_be_bytes(bytes[WIDTH_RANGE].try_into().unwrap()) as usize;
    let height = u32::from_be_bytes(bytes[HEIGHT_RANGE].try_into().unwrap()) as usize;
    let mut pixels: Vec<Vec<YCbCr>> = Vec::with_capacity(height);
    let macroblock_width =
        u32::from_be_bytes(bytes[MACROBLOCK_WIDTH_RANGE].try_into().unwrap()) as usize;
    let macroblock_height =
        u32::from_be_bytes(bytes[MACROBLOCK_HEIGHT_RANGE].try_into().unwrap()) as usize;
    let mut offset = METADATA_LENGTH;

    let luma_channel = decode_channel(bytes, &mut offset, dct, macroblock_width, macroblock_height);
    let cb_channel = decode_channel(bytes, &mut offset, dct, macroblock_width, macroblock_height);
    let cr_channel = decode_channel(bytes, &mut offset, dct, macroblock_width, macroblock_height);

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
    bytes: &[u8],
    offset: &mut usize,
    dct: &DiscreteCosineTransformer,
    macroblock_width: usize,
    macroblock_height: usize,
) -> Vec4<u8> {
    let mut result: Vec4<u8> = Vec::with_capacity(macroblock_height);

    for mr in 0..macroblock_height {
        result.push(Vec::with_capacity(macroblock_width));

        for _mc in 0..macroblock_width {
            let normalization_factor =
                f32::from_be_bytes(bytes[*offset..*offset + 4].try_into().unwrap());
            *offset += 4;
            let dc = f32::from_be_bytes(bytes[*offset..*offset + 4].try_into().unwrap());
            *offset += 4;
            let macroblock_size =
                u16::from_be_bytes(bytes[*offset..*offset + 2].try_into().unwrap()) as usize;
            *offset += 2;

            let macroblock_bytes = &bytes[*offset..*offset + macroblock_size];
            *offset += macroblock_size;
            let normalized_amplitudes: Vec<Vec<i8>> = decode_rle(macroblock_bytes);
            let mut amplitudes = DiscreteCosineTransformer::inverse_normalization(
                &normalized_amplitudes,
                normalization_factor,
            );
            amplitudes[0][0] = dc;
            let pixel_values = dct.idct(&amplitudes);
            result[mr].push(pixel_values);
        }
    }

    result
}
