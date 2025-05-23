pub mod decode;
pub mod encode;

type Vec4<T> = Vec<Vec<Vec<Vec<T>>>>;

//Structure
//
//big endian
//
//4 byte - width
//4 byte - height
//4 - macroblock width
//4 - macroblock height
//luma channel
//  for each macroblock
//      4 byte normilization factor (f32)
//      NUM_DCT_SIGNALS*NUM_DCT_SIGNALS bytes amplitudes
//cb channel - identical to luma
//cr channel - identical to luma
//
const METADATA_LENGTH: usize = 16;
const WIDTH_RANGE: std::ops::Range<usize> = 0..4;
const HEIGHT_RANGE: std::ops::Range<usize> = 4..8;
const MACROBLOCK_WIDTH_RANGE: std::ops::Range<usize> = 8..12;
const MACROBLOCK_HEIGHT_RANGE: std::ops::Range<usize> = 12..16;

