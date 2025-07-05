use std::f32::consts::PI;

use crate::image::MACROBLOCKS_SIZE;

pub const NUM_DCT_SIGNALS: usize = 8;
pub struct DiscreteCosineTransformer {
    dct_signals: [[[[f32; MACROBLOCKS_SIZE]; MACROBLOCKS_SIZE]; NUM_DCT_SIGNALS]; NUM_DCT_SIGNALS],
}
impl DiscreteCosineTransformer {
    pub fn new() -> Self {
        let mut dct_signals =
            [[[[0.; MACROBLOCKS_SIZE]; MACROBLOCKS_SIZE]; NUM_DCT_SIGNALS]; NUM_DCT_SIGNALS];
        let zero_normilization = 1. / (MACROBLOCKS_SIZE as f32).sqrt();
        let non_zero_normilization = (2. / (MACROBLOCKS_SIZE as f32)).sqrt();

        #[allow(clippy::needless_range_loop)]
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

    pub fn dct(&self, signal: &[Vec<u8>]) -> Vec<Vec<f32>> {
        let mut amplitudes = vec![vec![0.; NUM_DCT_SIGNALS]; NUM_DCT_SIGNALS];

        #[allow(clippy::needless_range_loop)]
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

    pub fn idct(&self, amplitudes: &[Vec<f32>]) -> Vec<Vec<u8>> {
        assert_eq!(amplitudes.len(), NUM_DCT_SIGNALS);
        assert_eq!(amplitudes[0].len(), NUM_DCT_SIGNALS);

        let mut result = vec![vec![0.; MACROBLOCKS_SIZE]; MACROBLOCKS_SIZE];

        #[allow(clippy::needless_range_loop)]
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

    pub fn normalize_amplitudes(amplitudes: &[Vec<f32>]) -> (Vec<Vec<i8>>, f32) {
        let normalization_factor = (amplitudes
            .iter()
            .map(|row| {
                row.iter()
                    .max_by(|x, y| x.abs().total_cmp(&y.abs()))
                    .unwrap()
            })
            .max_by(|x, y| x.abs().total_cmp(&y.abs()))
            .unwrap())
        .abs()
            / 127.;

        (
            amplitudes
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|amplitude| (amplitude / normalization_factor) as i8)
                        .collect()
                })
                .collect(),
            normalization_factor,
        )
    }

    pub fn inverse_normalization(
        normalized_amplitudes: &[Vec<i8>],
        normalization_factor: f32,
    ) -> Vec<Vec<f32>> {
        normalized_amplitudes
            .iter()
            .map(|row| {
                row.iter()
                    .map(|amp| *amp as f32 * normalization_factor)
                    .collect()
            })
            .collect()
    }
}
