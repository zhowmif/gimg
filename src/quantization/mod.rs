use crate::dct::NUM_DCT_SIGNALS;
use tables::QuantizationTable;

pub mod tables;

pub fn apply_quantization(
    amplitudes: &Vec<Vec<f32>>,
    quantization_table: QuantizationTable,
) -> Vec<Vec<f32>> {
    let mut result = vec![vec![0.; NUM_DCT_SIGNALS]; NUM_DCT_SIGNALS];

    for row in 0..NUM_DCT_SIGNALS {
        for col in 0..NUM_DCT_SIGNALS {
            result[row][col] = amplitudes[row][col] / quantization_table[row][col];
        }
    }

    result
}

pub fn apply_dequantization(
    amplitudes: &Vec<Vec<f32>>,
    quantization_table: QuantizationTable,
) -> Vec<Vec<f32>> {
    let mut result = vec![vec![0.; NUM_DCT_SIGNALS]; NUM_DCT_SIGNALS];

    for row in 0..NUM_DCT_SIGNALS {
        for col in 0..NUM_DCT_SIGNALS {
            result[row][col] = amplitudes[row][col] * quantization_table[row][col];
        }
    }

    result
}
