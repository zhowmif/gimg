use crate::{
    binary::byte_reader::ByteReader,
    mpegts::{pes_header_parser::streamid::PesStreamId, utils::MpegtsParseError},
    mpegts_assert,
};

mod media_pes;
mod pes_extension;
mod streamid;
mod trick_mode;

const PES_PACKET_START_CODE_PREFIX: [u8; 3] = [0, 0, 1];

#[derive(Default)]
pub struct DesPacket<'a> {
    payload: Option<&'a [u8]>,
    streamid: PesStreamId,
}

//code should be something like:
//*IMPORTANT*  - dont forget to handle packet length 0
//let expected length = get_des_packet_length_from_header(first_packet)
//let entire_pes = bytes![first_packet]
// while entire_pes.length < expected_length {entire_pes.concat(new_packet)}
// decode_des_packet_from_full_bytes(entire_pes[..expected_length])
pub fn get_des_packet_length_from_header(bytes: &[u8]) -> u16 {
    todo!()
}

pub fn decode_des_packet_from_full_bytes<'a>(
    bytes: &'a [u8],
) -> Result<DesPacket<'a>, MpegtsParseError> {
    let mut reader = ByteReader::new(bytes);
    let packet_start_code_prefix = reader.read_array::<3>().unwrap_or([0, 0, 0]);
    mpegts_assert!(
        packet_start_code_prefix == PES_PACKET_START_CODE_PREFIX,
        "invalid pes_packet_start_code_prefix"
    );
    let streamid = PesStreamId::from(reader.read_byte().unwrap());
    let pes_pakcet_length = reader.read_u16_be().unwrap();

    match streamid {
        PesStreamId::Audio(_) => todo!(),
        PesStreamId::Video(_) => todo!(),
        PesStreamId::Private1 => todo!(),
        PesStreamId::Padding => Ok(DesPacket {
            streamid,
            ..Default::default()
        }),
        _ => {
            let payload = reader.read_to_end();

            return Ok(DesPacket {
                streamid,
                payload: Some(payload),
            });
        }
    }
}
