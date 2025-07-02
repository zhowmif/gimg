use std::simd::u8x64;

macro_rules! simd_operation {
    ($lhs:expr, $rhs:expr, $op:tt) => {
        {
            let chunk_size = u8x64::LEN;
            let mut result = Vec::with_capacity($lhs.len());
            let mut i = 0;

            while i + chunk_size < $lhs.len() {
                let left_chunk = u8x64::from_slice(&$lhs[i..(i + chunk_size)]);
                let right_chunk = u8x64::from_slice(&$rhs[i..(i + chunk_size)]);
                let res = left_chunk $op right_chunk;
                result.extend_from_slice(&res.to_array());
                i += chunk_size;
            }

            for j in i..$lhs.len() {
                result.push($lhs[j] $op $rhs[j]);
            }

            result
        }
    };
}

pub fn subtract_simd(lhs: &[u8], rhs: &[u8]) -> Vec<u8> {
    simd_operation!(lhs, rhs, -)
}
