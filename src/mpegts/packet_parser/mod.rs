use crate::{
    binary::byte_reader::ByteReader,
    extract_bits,
    mpegts::{
        packet_parser::adaptation_field::{read_adaptation_field, AdaptationField},
        utils::MpegtsParseError,
    },
    mpegts_assert, read_flag_bit,
};

mod adaptation_field;
mod pcr;

pub struct MpegtsPacket<'a> {
    pid: u16,
    continuity_counter: u8,
    transport_scrambling_control: u8,
    transport_error_indicator: bool,
    payload_unit_start_indicator: bool,
    transport_priority: bool,
    adaptation_field: Option<AdaptationField<'a>>,
    payload: Option<&'a [u8]>,
}

const MPEGTS_PACKET_LENGTH: usize = 188;
const MPEGTS_SYNC_BYTE: u8 = 0x47;

read_flag_bit!(read_transport_error_indicator, 0);
read_flag_bit!(read_payload_unit_start_indicator, 1);
read_flag_bit!(read_transport_priority, 2);
extract_bits!(read_pid, u16, 3, 13);
extract_bits!(read_transport_scrambling_control, u8, 0, 2);
extract_bits!(read_adaptation_field_control, u8, 2, 2);
extract_bits!(read_continuity_counter, u8, 4, 4);

pub fn decode_mpegts_packet<'a>(bytes: &'a [u8]) -> Result<MpegtsPacket<'a>, MpegtsParseError> {
    mpegts_assert!(
        bytes.len() == MPEGTS_PACKET_LENGTH,
        "packet length must equal {}",
        MPEGTS_PACKET_LENGTH
    );
    let mut reader = ByteReader::new(bytes);
    let mpegts_header = reader.read_bytes(4).unwrap();
    let sync_byte = mpegts_header[0];
    mpegts_assert!(sync_byte == MPEGTS_SYNC_BYTE, "invalid sync byte");
    let transport_error_indicator = read_transport_error_indicator(mpegts_header[1]);
    let payload_unit_start_indicator = read_payload_unit_start_indicator(mpegts_header[1]);
    let transport_priority = read_transport_priority(mpegts_header[1]);
    let pid_bytes = u16::from_be_bytes([mpegts_header[1], mpegts_header[2]]);
    let pid = read_pid(pid_bytes);
    let transport_scrambling_control = read_transport_scrambling_control(mpegts_header[3]);
    let adaptation_field_control = read_adaptation_field_control(mpegts_header[3]);
    mpegts_assert!(
        adaptation_field_control != 0b00,
        "adaptation field control 00 is not allowed"
    );

    let continuity_counter = read_continuity_counter(mpegts_header[3]);

    let should_read_adaptation_field =
        adaptation_field_control == 0b10 || adaptation_field_control == 0b11;
    let adaptation_field = should_read_adaptation_field.then(|| read_adaptation_field(&mut reader));

    let should_read_payload = adaptation_field_control == 0b01 || adaptation_field_control == 0b11;
    let payload = should_read_payload.then(|| reader.read_to_end());

    return Ok(MpegtsPacket {
        transport_scrambling_control,
        continuity_counter,
        pid,
        transport_error_indicator,
        payload_unit_start_indicator,
        transport_priority,
        adaptation_field,
        payload,
    });
}
