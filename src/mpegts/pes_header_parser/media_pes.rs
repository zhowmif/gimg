use crate::{
    binary::byte_reader::ByteReader,
    extract_bits,
    mpegts::{
        pes_header_parser::{
            pes_extension::{read_pes_extension, PesExtension},
            trick_mode::PesTrickMode,
        },
        utils::{
            read_33_bit_uint_with_extension, read_marker_bit_seperated_33_bit_uint,
            MpegtsParseError,
        },
    },
    read_flag_bit,
};

extract_bits!(read_pes_scrambling_control, u8, 2, 2);
read_flag_bit!(read_pes_priority, 4);
read_flag_bit!(read_data_alignment_indicator, 5);
read_flag_bit!(read_copyright, 6);
read_flag_bit!(read_original_or_copy, 7);
extract_bits!(read_pts_dts_flags, u8, 0, 2);
read_flag_bit!(read_escr_flag, 2);
read_flag_bit!(read_es_rate_flag, 3);
read_flag_bit!(read_dsm_trick_mode_flag, 4);
read_flag_bit!(read_additional_copy_info_flag, 5);
read_flag_bit!(read_pes_crc_flag, 6);
read_flag_bit!(read_pes_extension_flag, 7);

extract_bits!(read_es_rate, u32, 1, 22);

#[derive(Debug)]
pub struct MediaPesData<'a> {
    pub pes_scrambling_control: u8,
    pub pes_priority: bool,
    pub data_alignment_indicator: bool,
    pub copyright: bool,
    pub original_or_copy: bool,
    pub pts: Option<u64>,
    pub dts: Option<u64>,
    pub escr: Option<u64>,
    pub es_rate: Option<u32>,
    pub trick_mode: Option<PesTrickMode>,
    pub additional_copy_info: Option<u8>,
    pub previous_pes_packet_crc: Option<u16>,
    pub extension: Option<PesExtension>,
    pub payload: &'a [u8],
}

pub fn parse_media_pes<'a>(reader: &mut ByteReader<'a>) -> Result<MediaPesData<'a>, MpegtsParseError> {
    let first_flag_byte = reader.read_byte().unwrap();
    let pes_scrambling_control = read_pes_scrambling_control(first_flag_byte);
    let pes_priority = read_pes_priority(first_flag_byte);
    let data_alignment_indicator = read_data_alignment_indicator(first_flag_byte);
    let copyright = read_copyright(first_flag_byte);
    let original_or_copy = read_original_or_copy(first_flag_byte);

    let second_flag_byte = reader.read_byte().unwrap();
    let pts_dts_flags = read_pts_dts_flags(second_flag_byte);
    let pts_enabled = pts_dts_flags == 0b10 || pts_dts_flags == 0b11;
    let dts_enabled = pts_dts_flags == 11;
    let escr_flag = read_escr_flag(second_flag_byte);
    let es_rate_flag = read_es_rate_flag(second_flag_byte);
    let dsm_trick_mode_flag = read_dsm_trick_mode_flag(second_flag_byte);
    let additional_copy_info_flag = read_additional_copy_info_flag(second_flag_byte);
    let pes_crc_flag = read_pes_crc_flag(second_flag_byte);
    let pes_extension_flag = read_pes_extension_flag(second_flag_byte);

    let pes_header_data_length = reader.read_byte().unwrap();
    let pes_header_end_offset = reader.offset + pes_header_data_length as usize;

    let pts = pts_enabled.then(|| {
        let pts_bytes = reader.read_array::<5>().unwrap();

        read_marker_bit_seperated_33_bit_uint(&pts_bytes)
    });
    let dts = dts_enabled.then(|| {
        let dts_bytes = reader.read_array::<5>().unwrap();

        read_marker_bit_seperated_33_bit_uint(&dts_bytes)
    });
    let escr =
        escr_flag.then(|| read_33_bit_uint_with_extension(&reader.read_array::<6>().unwrap()));
    let es_rate = es_rate_flag.then(|| {
        let es_rate_bytes = reader.read_array::<3>().unwrap();
        let as_num = u32::from_be_bytes([0, es_rate_bytes[0], es_rate_bytes[1], es_rate_bytes[2]]);

        read_es_rate(as_num)
    });
    let trick_mode = dsm_trick_mode_flag.then(|| PesTrickMode::from(reader.read_byte().unwrap()));
    let additional_copy_info =
        additional_copy_info_flag.then(|| reader.read_byte().unwrap() & 0x7f);
    let previous_pes_packet_crc = pes_crc_flag.then(|| reader.read_u16_be().unwrap());
    let extension = pes_extension_flag.then(|| read_pes_extension(reader));

    reader.skip_bytes(pes_header_end_offset - reader.offset);
    let payload = reader.read_to_end();

    Ok(MediaPesData {
        pes_scrambling_control,
        pes_priority,
        data_alignment_indicator,
        copyright,
        original_or_copy,
        pts,
        dts,
        escr,
        es_rate,
        trick_mode,
        additional_copy_info,
        previous_pes_packet_crc,
        extension,
        payload,
    })
}
