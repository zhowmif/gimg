use crate::{
    colors::YCbCr,
    dct::DiscreteCosineTransformer,
    image::{Image, MACROBLOCKS_SIZE},
};

use super::{rle::encode_rle, Vec4, METADATA_LENGTH};

pub fn encode_image(img: Image, dct: &DiscreteCosineTransformer) -> Vec<u8> {
    let mut metadata = Vec::with_capacity(METADATA_LENGTH);
    let macroblocks = img.get_macroblocks(MACROBLOCKS_SIZE);

    metadata.extend_from_slice(&(img.resolution.width as u32).to_be_bytes());
    metadata.extend_from_slice(&(img.resolution.height as u32).to_be_bytes());
    metadata.extend_from_slice(&(macroblocks[0].len() as u32).to_be_bytes());
    metadata.extend_from_slice(&(macroblocks.len() as u32).to_be_bytes());

    let luma_macroblocks = get_channel_macroblocks(&macroblocks, |y| y.y);
    let luma_channel = encode_channel(&luma_macroblocks, dct);

    let cb_macroblocks = get_channel_macroblocks(&macroblocks, |y| y.cb);
    let cb_channel = encode_channel(&cb_macroblocks, dct);

    let cr_macroblocks = get_channel_macroblocks(&macroblocks, |y| y.cr);
    let cr_channel = encode_channel(&cr_macroblocks, dct);

    [metadata, luma_channel, cb_channel, cr_channel].concat()
}

fn encode_channel(macroblocks: &Vec4<u8>, dct: &DiscreteCosineTransformer) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();

    for row in macroblocks {
        for macroblock in row {
            let mut amplitudes = dct.dct(macroblock);
            let dc = amplitudes[0][0];
            amplitudes[0][0] = 1.;
            let (normalized_amplitudes, normilization) =
                DiscreteCosineTransformer::normalize_amplitudes(&amplitudes);
            let rle_encoded_macroblock = encode_rle(&normalized_amplitudes);

            result.extend_from_slice(&normilization.to_be_bytes());
            result.extend_from_slice(&dc.to_be_bytes());
            result.extend((rle_encoded_macroblock.len() as u16).to_be_bytes());
            result.extend(rle_encoded_macroblock);
        }
    }

    result
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
