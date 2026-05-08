use crate::{binary::byte_reader::ByteReader, mpegts::utils::MpegtsParseError, mpegts_assert};

mod streamid;

const PES_PACKET_START_CODE_PREFIX: [u8; 3] = [0, 0, 1];

pub struct DesPacket {}

pub fn decode_des_packet(bytes: &[u8]) -> Result<DesPacket, MpegtsParseError> {
    let mut reader = ByteReader::new(bytes);
    let packet_start_code_prefix = reader.read_array::<3>().unwrap_or([0, 0, 0]);
    mpegts_assert!(
        packet_start_code_prefix == PES_PACKET_START_CODE_PREFIX,
        "invalid pes_packet_start_code_prefix"
    );

    todo!()
}

