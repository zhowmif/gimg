use crate::{binary::byte_reader::ByteReader, extract_bits, read_flag_bit};

pub struct MpegtsProgramAssociationtable {}

extract_bits!(read_section_length, u16, 4, 12);
extract_bits!(read_version_number, u8, 5, 2);
read_flag_bit!(read_current_next_indicator, 7);
extract_bits!(read_pat_entry_pid, u16, 3, 13);

pub fn parse_pat(bytes: &[u8]) -> MpegtsProgramAssociationtable {
    let mut reader = ByteReader::new(bytes);
    let table_id = reader.read_byte().unwrap();
    let section_length_bytes = reader.read_u16_be().unwrap();
    let section_length = read_section_length(section_length_bytes);
    let transport_stream_id = reader.read_u16_be().unwrap();
    let version_byte = reader.read_byte().unwrap();
    let version_number = read_version_number(version_byte);
    let current_next_indicator = read_current_next_indicator(version_byte);
    let section_number = reader.read_byte().unwrap();
    let last_section_number = reader.read_byte().unwrap();
    let number_of_programs = (bytes.len() - reader.offset - 32) / 32;

    let entries: Vec<PatEntry> = (0..number_of_programs)
        .map(|_| {
            let program_number = reader.read_u16_be().unwrap();
            let pid = read_pat_entry_pid(reader.read_u16_be().unwrap());

            match program_number {
                0 => PatEntry::NetworkPid(pid),
                _ => PatEntry::ProgramMap(pid),
            }
        })
        .collect();

    todo!()
}

enum PatEntry {
    ProgramMap(u16),
    NetworkPid(u16),
}
