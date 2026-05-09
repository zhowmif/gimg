use crate::{
    binary::byte_reader::ByteReader,
    extract_bits,
    mpegts::{pes_header_parser::streamid::PesStreamId, utils::read_33_bit_uint_with_extension},
    read_flag_bit,
};

#[derive(Debug)]
pub struct PesExtension<> {
    private_data: Option<[u8; 128]>,
    pack_header: Option<PackHeader>,
    program_packet_sequence: Option<ProgramPacketSequence>,
    p_std_total_buffer_size: Option<u32>,
}

read_flag_bit!(read_pes_private_data_flag, 0);
read_flag_bit!(read_pack_header_field_flag, 1);
read_flag_bit!(read_program_packet_sequence_counter_flag, 2);
read_flag_bit!(read_p_std_buffer_flag, 3);
read_flag_bit!(read_pes_extension_flag_2, 7);

const PES_PRIVATE_DATA_LENGTH: usize = 128;

pub fn read_pes_extension(reader: &mut ByteReader) -> PesExtension {
    let flags_byte = reader.read_byte().unwrap();
    let pes_private_data_flag = read_pes_private_data_flag(flags_byte);
    let pack_header_field_flag = read_pack_header_field_flag(flags_byte);
    let program_packet_sequence_counter_flag =
        read_program_packet_sequence_counter_flag(flags_byte);
    let p_std_buffer_flag = read_p_std_buffer_flag(flags_byte);
    let pes_extension_flag_2 = read_pes_extension_flag_2(flags_byte);

    let private_data =
        pes_private_data_flag.then(|| reader.read_array::<PES_PRIVATE_DATA_LENGTH>().unwrap());
    let pack_header = pack_header_field_flag.then(|| read_pack_header(reader));
    let program_packet_sequence =
        program_packet_sequence_counter_flag.then(|| read_program_packet_sequence(reader));
    let p_std_total_buffer_size = p_std_buffer_flag.then(|| read_p_std_total_buffer_size(reader));
    if pes_extension_flag_2 {
        let extension_field_length = reader.read_byte().unwrap() & 0x7f;
        reader.skip_bytes(extension_field_length as usize);
    }

    PesExtension {
        private_data,
        pack_header,
        program_packet_sequence,
        p_std_total_buffer_size,
    }
}

extract_bits!(read_p_std_buffer_scale, u16, 2, 1);
extract_bits!(read_p_std_buffer_size, u16, 3, 13);

fn read_p_std_total_buffer_size(reader: &mut ByteReader) -> u32 {
    let bytes = reader.read_u16_be().unwrap();
    let scale = read_p_std_buffer_scale(bytes);
    let size = read_p_std_buffer_size(bytes) as u32;

    size * if scale == 0 { 128 } else { 1024 }
}

#[derive(Debug)]
struct PackHeader {
    system_clock_reference: u64,
    program_mux_rate: u32,
    system_header: Option<SystemHeader>,
}

extract_bits!(read_program_mux_rate, u32, 8, 22);
extract_bits!(read_pack_stuffing_length, u8, 5, 3);

pub fn read_pack_header(reader: &mut ByteReader) -> PackHeader {
    //skip the start code
    reader.skip_bytes(4);
    let system_clock_reference =
        read_33_bit_uint_with_extension(&reader.read_array::<6>().unwrap());
    let mux_rate_bytes = reader.read_array::<3>().unwrap();
    let mux_rate_num =
        u32::from_be_bytes([0, mux_rate_bytes[0], mux_rate_bytes[1], mux_rate_bytes[2]]);
    let program_mux_rate = read_program_mux_rate(mux_rate_num);
    let pack_stuffing_length = read_pack_stuffing_length(reader.read_byte().unwrap());
    reader.skip_bytes(pack_stuffing_length as usize);
    let system_header_start_code_bytes = reader.peek_bytes(SYSTEM_HEADER_START_CODE.len());

    let system_header = (system_header_start_code_bytes == Some(SYSTEM_HEADER_START_CODE))
        .then(|| read_system_header(reader));

    PackHeader {
        system_clock_reference,
        program_mux_rate,
        system_header,
    }
}

const SYSTEM_HEADER_START_CODE: &[u8] = &[0, 0, 1, 0xbb];

#[derive(Debug)]
struct SystemHeader {
    rate_bound: u32,
    audio_bound: u8,
    fixed_flag: bool,
    csps_flag: bool,
    system_audio_lock_flag: bool,
    system_video_lock_flag: bool,
    video_bound: u8,
    packet_rate_restriction_flag: bool,
    streams: Vec<PesStream>,
}

extract_bits!(read_rate_bound, u32, 9, 22);
extract_bits!(read_audio_bound, u8, 0, 6);
read_flag_bit!(read_fixed_flag, 6);
read_flag_bit!(read_csps_flag, 7);
read_flag_bit!(read_system_audio_lock_flag, 0);
read_flag_bit!(read_system_video_lock_flag, 1);
extract_bits!(read_video_bound, u8, 3, 5);
read_flag_bit!(read_packet_rate_restriction_flag, 0);

fn read_system_header(reader: &mut ByteReader) -> SystemHeader {
    reader.skip_bytes(SYSTEM_HEADER_START_CODE.len());
    let header_length = reader.read_u16_be().unwrap();
    let system_header_end_offset = reader.offset + header_length as usize;
    let rate_bound_bytes = reader.read_array::<3>().unwrap();
    let rate_bound_int = u32::from_be_bytes([
        0,
        rate_bound_bytes[0],
        rate_bound_bytes[1],
        rate_bound_bytes[2],
    ]);
    let rate_bound = read_rate_bound(rate_bound_int);
    let audio_bound_byte = reader.read_byte().unwrap();
    let audio_bound = read_audio_bound(audio_bound_byte);
    let fixed_flag = read_fixed_flag(audio_bound_byte);
    let csps_flag = read_csps_flag(audio_bound_byte);

    let video_bound_byte = reader.read_byte().unwrap();
    let system_audio_lock_flag = read_system_audio_lock_flag(video_bound_byte);
    let system_video_lock_flag = read_system_video_lock_flag(video_bound_byte);
    let video_bound = read_video_bound(video_bound_byte);

    let packet_rate_restriction_flag =
        read_packet_rate_restriction_flag(reader.read_byte().unwrap());

    let mut streams = Vec::with_capacity((system_header_end_offset - reader.offset) / 3);
    while reader.offset < system_header_end_offset {
        let stream_id = PesStreamId::from(reader.read_byte().unwrap());
        let total_buf_size = read_p_std_total_buffer_size(reader);

        streams.push(PesStream::new(stream_id, total_buf_size));
    }

    SystemHeader {
        rate_bound,
        audio_bound,
        fixed_flag,
        csps_flag,
        system_audio_lock_flag,
        system_video_lock_flag,
        video_bound,
        packet_rate_restriction_flag,
        streams,
    }
}

#[derive(Debug)]
struct PesStream {
    stream_id: PesStreamId,
    p_std_total_buffer_size: u32,
}

impl PesStream {
    fn new(stream_id: PesStreamId, p_std_total_buffer_size: u32) -> Self {
        Self {
            stream_id,
            p_std_total_buffer_size,
        }
    }
}

#[derive(Debug)]
struct ProgramPacketSequence {
    counter: u8,
    mpeg1_mpeg2_identifier: bool,
    original_stuff_length: u8,
}

extract_bits!(read_program_packet_sequence_counter, u8, 1, 7);
read_flag_bit!(read_mpeg1_mpeg2_identifier, 1);
extract_bits!(read_original_stuff_length, u8, 2, 6);

fn read_program_packet_sequence(reader: &mut ByteReader) -> ProgramPacketSequence {
    let counter = read_program_packet_sequence_counter(reader.read_byte().unwrap());
    let second_byte = reader.read_byte().unwrap();
    let mpeg1_mpeg2_identifier = read_mpeg1_mpeg2_identifier(second_byte);
    let original_stuff_length = read_original_stuff_length(second_byte);

    ProgramPacketSequence {
        counter,
        mpeg1_mpeg2_identifier,
        original_stuff_length,
    }
}
