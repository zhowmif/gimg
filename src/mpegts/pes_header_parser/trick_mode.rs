use crate::{extract_bits, read_flag_bit};

#[derive(Debug)]
pub enum PesTrickMode {
    FastForward {
        field_id: u8,
        intra_slice_refresh: bool,
        frequency_truncation: u8,
    },
    SlowMotion {
        rep_cntrl: u8,
    },
    FreezeFrame {
        field_id: u8,
    },
    FastReverse {
        field_id: u8,
        intra_slice_refresh: bool,
        frequency_truncation: u8,
    },
    SlowReverse {
        rep_cntrl: u8,
    },
    Unknown,
}

extract_bits!(read_trick_mode_control, u8, 0, 3);
extract_bits!(read_field_id, u8, 3, 2);
read_flag_bit!(read_intra_slice_refresh, 5);
extract_bits!(read_frequency_truncation, u8, 6, 2);
extract_bits!(read_rep_cntrl, u8, 3, 5);

impl From<u8> for PesTrickMode {
    fn from(byte: u8) -> Self {
        let trick_mode_control = read_trick_mode_control(byte);
        let field_id = read_field_id(byte);
        let intra_slice_refresh = read_intra_slice_refresh(byte);
        let frequency_truncation = read_frequency_truncation(byte);
        let rep_cntrl = read_rep_cntrl(byte);

        match trick_mode_control {
            0b000 => Self::FastForward {
                field_id,
                intra_slice_refresh,
                frequency_truncation,
            },
            0b001 => Self::SlowMotion { rep_cntrl },
            0b010 => Self::FreezeFrame { field_id },
            0b011 => Self::FastReverse {
                field_id,
                intra_slice_refresh,
                frequency_truncation,
            },
            0b100 => Self::SlowReverse { rep_cntrl },
            _ => Self::Unknown,
        }
    }
}
