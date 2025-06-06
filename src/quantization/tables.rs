use crate::dct::NUM_DCT_SIGNALS;

pub type QuantizationTable = [[f32; NUM_DCT_SIGNALS]; NUM_DCT_SIGNALS];

const Q_FACTOR: f32 = 1.;

pub const DEFAULT_LUMA_QUANTIZATION_TABLE: QuantizationTable = [
    [16., 11., 10., 16., 24., 40., 51., 61.],
    [12., 12., 14., 19., 26., 58., 60., 55.],
    [14., 13., 16., 24., 40., 57., 69., 56.],
    [14., 17., 22., 29., 51., 87., 80., 62.],
    [18., 22., 37., 56., 68., 109., 103., 77.],
    [24., 35., 55., 64., 81., 104., 113., 92.],
    [49., 64., 78., 87., 103., 121., 120., 101.],
    [72., 92., 95., 98., 112., 100., 103., 99.],
];

pub const DEFAULT_CHROMA_QUANTIZATION_TABLE: QuantizationTable = [
    [17., 18., 24., 47., 99., 99., 99., 99.],
    [18., 21., 26., 66., 99., 99., 99., 99.],
    [24., 26., 56., 99., 99., 99., 99., 99.],
    [47., 66., 99., 99., 99., 99., 99., 99.],
    [99., 99., 99., 99., 99., 99., 99., 99.],
    [99., 99., 99., 99., 99., 99., 99., 99.],
    [99., 99., 99., 99., 99., 99., 99., 99.],
    [99., 99., 99., 99., 99., 99., 99., 99.],
];

// struct Quantizer {
//     q_factor: f32,
//     luma_table: 
// }
//
// impl Quantizer {
//     fn new(q_factor: f32) -> Self {
//         Self { q_factor }
//     }
//
//     fn apply_q_factor(q_factor: f32, table: QuantizationTable) -> QuantizationTable {
//         let mut result = [[0.; NUM_DCT_SIGNALS]; NUM_DCT_SIGNALS];
//
//         for row in 0..NUM_DCT_SIGNALS {
//             for col in 0..NUM_DCT_SIGNALS {
//                 result[row][col] = table[row][col] * q_factor;
//             }
//         }
//
//         result
//     }
// }
//
