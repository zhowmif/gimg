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
        let zero_normilization = 1. / (MACROBLOCKS_SIZE as f32).sqrt();
        let non_zero_normilization = (2. / (MACROBLOCKS_SIZE as f32)).sqrt();

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

                        let p_normilization = if p == 0 {
                            zero_normilization
                        } else {
                            non_zero_normilization
                        };
                        let q_normilization = if q == 0 {
                            zero_normilization
                        } else {
                            non_zero_normilization
                        };

                        dct_signals[p][q][row][col] =
                            p_normilization * q_normilization * x_component * y_component;
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

    pub fn idct(&self, amplitudes: [[f32; NUM_DCT_SIGNALS]; NUM_DCT_SIGNALS]) -> Vec<Vec<u8>> {
        let mut result = vec![vec![0.; MACROBLOCKS_SIZE]; MACROBLOCKS_SIZE];

        for p in 0..NUM_DCT_SIGNALS {
            for q in 0..NUM_DCT_SIGNALS {
                let curr_amplitude = amplitudes[p][q];
                let signal = self.dct_signals[p][q];

                for row in 0..MACROBLOCKS_SIZE {
                    for col in 0..MACROBLOCKS_SIZE {
                        result[row][col] += signal[row][col] * curr_amplitude;
                    }
                }
            }
        }

        result
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|x| x.clamp(u8::MIN as f32, u8::MAX as f32) as u8)
                    .collect()
            })
            .collect()
    }

    pub fn normalize_amplitudes(
        amplitudes: [[f32; NUM_DCT_SIGNALS]; NUM_DCT_SIGNALS],
    ) -> ([[i8; NUM_DCT_SIGNALS]; NUM_DCT_SIGNALS], f32) {
        let normalization_factor = (amplitudes
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .max_by(|x, y| x.abs().total_cmp(&y.abs()))
                    .unwrap()
            })
            .max_by(|x, y| x.total_cmp(&y))
            .unwrap())
            / 127.;

        (
            amplitudes.map(|row| row.map(|amplitude| (amplitude / normalization_factor) as i8)),
            normalization_factor,
        )
    }

    pub fn inverse_normalization(
        normalized_amplitudes: [[i8; NUM_DCT_SIGNALS]; NUM_DCT_SIGNALS],
        normalization_factor: f32,
    ) -> [[f32; NUM_DCT_SIGNALS]; NUM_DCT_SIGNALS] {
        normalized_amplitudes.map(|row| row.map(|amp| amp as f32 * normalization_factor))
    }
}
