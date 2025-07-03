use std::simd::{cmp::SimdPartialOrd, i16x64, num::SimdInt, u8x64};

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

pub fn png_average_simd(x: &[u8], a: &[u8], b: &[u8]) -> Vec<u8> {
    let chunk_size = u8x64::LEN;
    let mut result = Vec::with_capacity(x.len());
    let mut i = 0;

    while i + chunk_size < x.len() {
        let x_chunk = u8x64::from_slice(&x[i..(i + chunk_size)]);
        let a_chunk = u8x64::from_slice(&a[i..(i + chunk_size)]);
        let b_chunk = u8x64::from_slice(&b[i..(i + chunk_size)]);
        let avg_chunk = (a_chunk / u8x64::splat(2))
            + (b_chunk / u8x64::splat(2))
            + (a_chunk & b_chunk & u8x64::splat(1));
        let res = x_chunk - avg_chunk;
        result.extend_from_slice(&res.to_array());
        i += chunk_size;
    }

    for j in i..x.len() {
        result.push(x[j] - (a[j] / 2 + b[j] / 2 + (a[j] & b[j] & 1)));
    }

    result
}

pub fn paeth_predictor_simd(x: &[u8], a: &[u8], b: &[u8], c: &[u8]) -> Vec<u8> {
    let vec_size = u8x64::LEN;
    let mut result = Vec::with_capacity(x.len());
    let mut i = 0;

    let (ai16, bi16, ci16): (Vec<_>, Vec<_>, Vec<_>) = (
        a.iter().map(|elem| *elem as i16).collect(),
        b.iter().map(|elem| *elem as i16).collect(),
        c.iter().map(|elem| *elem as i16).collect(),
    );

    while i + vec_size < x.len() {
        let a_vec = i16x64::from_slice(&ai16[i..(i + vec_size)]);
        let b_vec = i16x64::from_slice(&bi16[i..(i + vec_size)]);
        let c_vec = i16x64::from_slice(&ci16[i..(i + vec_size)]);

        let p = a_vec + b_vec - c_vec;
        let pa = p.abs_diff(a_vec);
        let pb = p.abs_diff(b_vec);
        let pc = p.abs_diff(c_vec);

        let pa_pb_mask = pa.simd_le(pb);
        let pa_pc_mask = pa.simd_le(pc);
        let pb_pc_mask = pb.simd_le(pc);

        let paeth_result = pa_pb_mask.select(
            pa_pc_mask.select(a_vec, c_vec),
            pb_pc_mask.select(b_vec, c_vec),
        );
        let paeth_vec: Vec<u8> = paeth_result
            .to_array()
            .into_iter()
            .map(|elem| elem as u8)
            .collect();
        let paeth_result = u8x64::from_slice(&paeth_vec);
        let x_vec = u8x64::from_slice(&x[i..(i + vec_size)]);
        let res = x_vec - paeth_result;
        result.extend_from_slice(&res.to_array());
        i += vec_size;
    }

    for j in i..x.len() {
        result.push(x[j].wrapping_sub(paeth_predictor(a[j], b[j], c[j])));
    }

    result
}

fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let (a, b, c) = (a as i16, b as i16, c as i16);
    let p = a + b - c;
    let pa = p.abs_diff(a);
    let pb = p.abs_diff(b);
    let pc = p.abs_diff(c);

    if pa <= pb && pa <= pc {
        a as u8
    } else if pb <= pc {
        b as u8
    } else {
        c as u8
    }
}

//TODO: try doing it in a more generic way
//
// macro_rules! simd_operation {
//     ($operands:expr, $vector_operation:tt, $scalar_operation:tt) => {{
//         let data_len = $operands[0].len();
//         let chunk_size = u8x64::LEN;
//         let mut result = Vec::with_capacity(data_len);
//         let mut i = 0;
//
//         while i + chunk_size < data_len {
//             let ops = $operands
//                 .iter()
//                 .map(|operand| u8x64::from_slice(&operand[i..(i + chunk_size)]));
//             let res = ($vector_operation);
//             result.extend_from_slice(&res.to_array());
//             i += chunk_size;
//         }
//
//         for j in i..data_len {
//             let ops = $operands.iter().map(|operand| operand[j]);
//             result.push($scalar_operation);
//         }
//
//         result
//     }};
// }
//
// pub fn subtract_simd(lhs: &[u8], rhs: &[u8]) -> Vec<u8> {
//     simd_operation!(vec![lhs, rhs], (ops[0] - ops[1]), (ops[0] - ops[1]))
// }
