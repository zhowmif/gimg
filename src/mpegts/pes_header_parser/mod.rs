use crate::{
    binary::byte_reader::ByteReader,
    mpegts::{
        pes_header_parser::{
            media_pes::{parse_media_pes, MediaPesData},
            streamid::PesStreamId,
        },
        utils::MpegtsParseError,
    },
    mpegts_assert,
};

mod media_pes;
mod pes_extension;
mod streamid;
mod trick_mode;

const PES_PACKET_START_CODE_PREFIX: [u8; 3] = [0, 0, 1];

#[derive(Default, Debug)]
pub struct PesPacket<'a> {
    pub payload: Option<&'a [u8]>,
    pub streamid: PesStreamId,
    pub medie_pes_info: Option<MediaPesData<'a>>,
}

//code should be something like:
//*IMPORTANT*  - dont forget to handle packet length 0
//let expected length = get_pes_packet_length_from_header(first_packet)
//let entire_pes = bytes![first_packet]
// while entire_pes.length < expected_length {entire_pes.concat(new_packet)}
// decode_pes_packet_from_full_bytes(entire_pes[..expected_length])
pub fn get_pes_packet_length_from_header(bytes: &[u8]) -> u16 {
    todo!()
}

pub fn decode_pes_packet_from_full_bytes<'a>(
    bytes: &'a [u8],
) -> Result<PesPacket<'a>, MpegtsParseError> {
    let mut reader = ByteReader::new(bytes);
    let packet_start_code_prefix = reader.read_array::<3>().unwrap_or([0, 0, 0]);
    mpegts_assert!(
        packet_start_code_prefix == PES_PACKET_START_CODE_PREFIX,
        "invalid pes_packet_start_code_prefix"
    );
    let streamid = PesStreamId::from(reader.read_byte().unwrap());
    let pes_pakcet_length = reader.read_u16_be().unwrap();

    match streamid {
        PesStreamId::Audio(_) | PesStreamId::Video(_) | PesStreamId::Private1 => {
            let info = parse_media_pes(&mut reader)?;

            return Ok(PesPacket {
                medie_pes_info: Some(info),
                streamid,
                ..Default::default()
            });
        }
        PesStreamId::Padding => Ok(PesPacket {
            streamid,
            ..Default::default()
        }),
        _ => {
            let payload = reader.read_to_end();

            return Ok(PesPacket {
                streamid,
                payload: Some(payload),
                ..Default::default()
            });
        }
    }
}
