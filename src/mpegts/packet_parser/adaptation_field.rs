use crate::{
    binary::byte_reader::ByteReader, extract_bits,
    mpegts::{packet_parser::pcr::{PCR_BYTE_LENGTH, Pcr}, utils::read_marker_bit_seperated_33_bit_uint}, read_flag_bit,
};

#[derive(Default)]
pub struct AdaptationField<'a> {
    pcr: Option<Pcr>,
    opcr: Option<Pcr>,
    discontinuity_indicator: bool,
    random_access_indicator: bool,
    elementary_stream_priority_indicator: bool,
    splice_countdown: Option<i8>,
    private_data: Option<&'a [u8]>,
    adaptation_field_extension: Option<AdaptationFieldExtension>,
}

read_flag_bit!(read_discontinuity_indicator, 0);
read_flag_bit!(read_random_access_indicator, 1);
read_flag_bit!(read_elementary_stream_priority_indicator, 2);
read_flag_bit!(read_pcr_flag, 3);
read_flag_bit!(read_opcr_flag, 4);
read_flag_bit!(read_splicing_point_flag, 5);
read_flag_bit!(read_transport_private_data_flag, 6);
read_flag_bit!(read_adaptation_field_extension_flag, 7);

pub fn read_adaptation_field<'a>(reader: &mut ByteReader<'a>) -> AdaptationField<'a> {
    let adaptation_field_length = reader.read_byte().unwrap();

    if adaptation_field_length == 0 {
        return AdaptationField::default();
    }

    let adaptation_field_end_offset = reader.offset + adaptation_field_length as usize;
    let flags_byte = reader.read_byte().unwrap();
    let discontinuity_indicator = read_discontinuity_indicator(flags_byte);
    let random_access_indicator = read_random_access_indicator(flags_byte);
    let elementary_stream_priority_indicator =
        read_elementary_stream_priority_indicator(flags_byte);
    let pcr_flag = read_pcr_flag(flags_byte);
    let opcr_flag = read_opcr_flag(flags_byte);
    let splicing_point_flag = read_splicing_point_flag(flags_byte);
    let transport_private_data_flag = read_transport_private_data_flag(flags_byte);
    let adaptation_field_extension_flag = read_adaptation_field_extension_flag(flags_byte);

    let pcr = pcr_flag.then(|| Pcr::from_bytes(&reader.read_array::<PCR_BYTE_LENGTH>().unwrap()));
    let opcr = opcr_flag.then(|| Pcr::from_bytes(&reader.read_array::<PCR_BYTE_LENGTH>().unwrap()));
    let splice_countdown = splicing_point_flag.then(|| reader.read_byte().unwrap() as i8);

    let private_data = transport_private_data_flag
        .then(|| {
            let private_data_length = reader.read_byte().unwrap();

            reader.read_bytes(private_data_length.into())
        })
        .flatten();
    let adaptation_field_extension =
        adaptation_field_extension_flag.then(|| read_adaptation_field_extension(reader));
    reader.skip_bytes(adaptation_field_end_offset - reader.offset);

    AdaptationField {
        pcr,
        opcr,
        discontinuity_indicator,
        random_access_indicator,
        elementary_stream_priority_indicator,
        splice_countdown,
        private_data,
        adaptation_field_extension,
    }
}

#[derive(Default)]
struct AdaptationFieldExtension {
    lwt_offset: Option<u16>,
    piecewise_rate: Option<u32>,
    splice_info: Option<SpliceInfo>,
}

struct SpliceInfo {
    splice_type: u8,
    dts_next_au: u64,
}

read_flag_bit!(read_lwt_flag, 0);
read_flag_bit!(read_piecewise_rate_flag, 1);
read_flag_bit!(read_seamless_splice_flag, 2);
read_flag_bit!(read_lwt_valid_flag, 0);
extract_bits!(read_lwt_offset, u16, 1, 15);
extract_bits!(read_piecewise_rate, u32, 10, 22);
extract_bits!(read_splice_type, u8, 0, 4);

fn read_adaptation_field_extension(reader: &mut ByteReader) -> AdaptationFieldExtension {
    let extension_length = reader.read_byte().unwrap();

    if extension_length == 0 {
        return AdaptationFieldExtension::default();
    }

    let extension_end_offset = reader.offset + extension_length as usize;
    let flags_byte = reader.read_byte().unwrap();
    let lwt_flag = read_lwt_flag(flags_byte);
    let piecewise_rate_flag = read_piecewise_rate_flag(flags_byte);
    let seamless_splice_flag = read_seamless_splice_flag(flags_byte);

    let lwt_offset = lwt_flag
        .then(|| {
            let lwt_bytes = reader.read_u16_be().unwrap();
            let lwt_valid = read_lwt_valid_flag((lwt_bytes >> 8) as u8);

            lwt_valid.then(|| read_lwt_offset(lwt_bytes))
        })
        .flatten();

    let piecewise_rate = piecewise_rate_flag.then(|| {
        let piecewise_bytes = reader.read_bytes(3).unwrap();
        let piecewise_bytes_num = u32::from_be_bytes([
            0,
            piecewise_bytes[0],
            piecewise_bytes[1],
            piecewise_bytes[2],
        ]);

        read_piecewise_rate(piecewise_bytes_num)
    });
    let splice_info = seamless_splice_flag.then(|| {
        let dts_next_au_bytes = reader.read_array::<5>().unwrap();
        let splice_type = read_splice_type(dts_next_au_bytes[0]);
        let dts_next_au = read_marker_bit_seperated_33_bit_uint(&dts_next_au_bytes);

        SpliceInfo {
            splice_type,
            dts_next_au,
        }
    });
    reader.skip_bytes(extension_end_offset - reader.offset);

    AdaptationFieldExtension {
        lwt_offset,
        piecewise_rate,
        splice_info,
    }
}
