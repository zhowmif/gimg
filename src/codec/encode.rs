use crate::{
    colors::YCbCr,
    dct::{DiscreteCosineTransformer, NUM_DCT_SIGNALS},
    image::{Image, MACROBLOCKS_SIZE},
};

use super::{
    Vec4, HEIGHT_RANGE, MACROBLOCK_HEIGHT_RANGE, MACROBLOCK_WIDTH_RANGE, METADATA_LENGTH,
    WIDTH_RANGE,
};

pub fn encode_image(img: Image, dct: &DiscreteCosineTransformer) -> Vec<u8> {
    let macroblocks = img.get_macroblocks(MACROBLOCKS_SIZE);
    let num_macroblocks = macroblocks.len() * macroblocks[0].len();
    let channel_size = num_macroblocks * (NUM_DCT_SIGNALS * NUM_DCT_SIGNALS + 4);
    let mut encoded_image: Vec<u8> = vec![0; METADATA_LENGTH + channel_size * 3];

    encoded_image[WIDTH_RANGE].clone_from_slice(&(img.resolution.width as u32).to_be_bytes());
    encoded_image[HEIGHT_RANGE].clone_from_slice(&(img.resolution.height as u32).to_be_bytes());
    encoded_image[MACROBLOCK_WIDTH_RANGE]
        .clone_from_slice(&(macroblocks[0].len() as u32).to_be_bytes());
    encoded_image[MACROBLOCK_HEIGHT_RANGE]
        .clone_from_slice(&(macroblocks.len() as u32).to_be_bytes());

    let luma_macroblocks = get_channel_macroblocks(&macroblocks, |y| y.y);
    encode_channel(
        &luma_macroblocks,
        dct,
        &mut encoded_image[METADATA_LENGTH..METADATA_LENGTH + channel_size],
    );

    let cb_macroblocks = get_channel_macroblocks(&macroblocks, |y| y.cb);
    encode_channel(
        &cb_macroblocks,
        dct,
        &mut encoded_image[METADATA_LENGTH + channel_size..METADATA_LENGTH + channel_size * 2],
    );

    let cr_macroblocks = get_channel_macroblocks(&macroblocks, |y| y.cr);
    encode_channel(
        &cr_macroblocks,
        dct,
        &mut encoded_image[METADATA_LENGTH + channel_size * 2..METADATA_LENGTH + channel_size * 3],
    );

    encoded_image
}

fn encode_channel(macroblocks: &Vec4<u8>, dct: &DiscreteCosineTransformer, target: &mut [u8]) {
    let mut target_counter = 0;

    for row in macroblocks {
        for macroblock in row {
            let amplitudes = dct.dct(macroblock);
            let (normalized_amplitudes, normilization) =
                DiscreteCosineTransformer::normalize_amplitudes(amplitudes);
            target[target_counter..target_counter + 4]
                .clone_from_slice(&normilization.to_be_bytes());
            target_counter += 4;

            for amplitude_row in normalized_amplitudes {
                for amplitude in amplitude_row {
                    target[target_counter] = amplitude as u8;
                    target_counter += 1;
                }
            }
        }
    }
}

fn get_channel_macroblocks(macroblocks: &Vec4<YCbCr>, get_channel: fn(&YCbCr) -> u8) -> Vec4<u8> {
    macroblocks
        .iter()
        .map(|macro_row| {
            macro_row
                .iter()
                .map(|macroblock| get_cb_macroblock(macroblock, get_channel))
                .collect()
        })
        .collect()
}

fn get_cb_macroblock(macroblock: &Vec<Vec<YCbCr>>, get_channel: fn(&YCbCr) -> u8) -> Vec<Vec<u8>> {
    macroblock
        .iter()
        .map(|row| row.iter().map(get_channel).collect())
        .collect()
}
