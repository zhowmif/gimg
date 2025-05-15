use std::f32::consts::PI;

use crate::image::MACROBLOCKS_SIZE;

const NUM_DCT_SIGNALS: usize = MACROBLOCKS_SIZE;
pub struct DiscreteCosineTransformer {
    dct_signals: [[[[f32; MACROBLOCKS_SIZE]; MACROBLOCKS_SIZE]; NUM_DCT_SIGNALS]; NUM_DCT_SIGNALS],
}
impl DiscreteCosineTransformer {
    pub fn new() -> Self {
        let mut dct_signals =
            [[[[0.; MACROBLOCKS_SIZE]; MACROBLOCKS_SIZE]; NUM_DCT_SIGNALS]; NUM_DCT_SIGNALS];

        for p in 0..NUM_DCT_SIGNALS {
            for q in 0..NUM_DCT_SIGNALS {
                for row in 0..MACROBLOCKS_SIZE {
                    for col in 0..MACROBLOCKS_SIZE {
                        let x_component = f32::cos(
                            p as f32 * PI * (2. * row as f32 + 1.) / (2. * MACROBLOCKS_SIZE as f32),
                        );
                        let y_component = f32::cos(
                            q as f32 * PI * (2. * col as f32 + 1.) / (2. * MACROBLOCKS_SIZE as f32),
                        );

                        dct_signals[p][q][row][col] = x_component * y_component;
                    }
                }
            }
        }
        Self { dct_signals }
    }

    pub fn dct(&self, signal: &Vec<Vec<u8>>) -> [[f32; NUM_DCT_SIGNALS]; NUM_DCT_SIGNALS] {
        let mut amplitudes = [[0.; NUM_DCT_SIGNALS]; NUM_DCT_SIGNALS];
        for p in 0..NUM_DCT_SIGNALS {
            for q in 0..NUM_DCT_SIGNALS {
                let curr_dct_signal = self.dct_signals[p][q];
                amplitudes[p][q] = (0..MACROBLOCKS_SIZE)
                    .map(|row| {
                        (0..MACROBLOCKS_SIZE)
                            .map(|col| signal[row][col] as f32 * curr_dct_signal[row][col])
                            .sum::<f32>()
                    })
                    .sum();
            }
        }
        amplitudes
    }
}
