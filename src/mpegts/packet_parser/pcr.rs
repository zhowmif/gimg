use crate::extract_bits;

pub const PCR_BYTE_LENGTH: usize = 6;

//value is saved in 27MHz ticks
#[derive(Debug)]
pub(crate) struct Pcr(u64);

extract_bits!(read_pcr_base, u64, 16, 33);
extract_bits!(read_pcr_extension, u64, 55, 9);

impl Pcr {
    pub fn from_bytes(bytes: &[u8; PCR_BYTE_LENGTH]) -> Self {
        let num = Self::u48_slice_to_u64(bytes);
        let base = read_pcr_base(num);
        let extension = read_pcr_extension(num);

        Self(base * 300 + extension)
    }

    fn u48_slice_to_u64(bytes: &[u8; PCR_BYTE_LENGTH]) -> u64 {
        u64::from_be_bytes([
            //hehe
            0, 0, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
        ])
    }
}
