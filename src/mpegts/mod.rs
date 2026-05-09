use std::fs;

use crate::mpegts::{
    packet_parser::decode_mpegts_packet, pes_header_parser::decode_pes_packet_from_full_bytes,
};

mod packet_parser;
mod pat;
mod pes_header_parser;
mod utils;

// const TEST_NUM_PACKETS: usize = 1000;

pub fn test_vid() {
    let input = &fs::read("files/input.ts").unwrap();
    // let input = &fs::read("files/input.ts").unwrap()[..188 * TEST_NUM_PACKETS];
    let mut current_pes_bytes: Vec<u8> = Vec::new();
    let mut entire_video_bytes: Vec<u8> = Vec::with_capacity(50_000_000);
    let mut packet_num = 0;
    while (packet_num + 1) * 188 <= input.len() {
        if packet_num % 100 == 0 {
            println!("{} out of {}", packet_num, input.len() / 188)
        }

        let packet_bytes = &input[packet_num * 188..(packet_num + 1) * 188];
        let parsed_packet = decode_mpegts_packet(packet_bytes).unwrap();

        match parsed_packet.pid {
            256 => {
                if parsed_packet.payload_unit_start_indicator && !current_pes_bytes.is_empty() {
                    let pes_packet = decode_pes_packet_from_full_bytes(&current_pes_bytes).unwrap();
                    // println!("{:?}", pes_packet);
                    let payload = pes_packet.medie_pes_info.map(|mpi| mpi.payload);
                    if let Some(bytes) = payload {
                        entire_video_bytes.extend_from_slice(bytes);
                    }

                    current_pes_bytes.clear();
                }
                current_pes_bytes.extend_from_slice(parsed_packet.payload.unwrap());

                // println!("{:?}", parsed_packet);
                // println!("{} - {}", packet_num, parsed_packet.payload_unit_start_indicator);
            }
            _ => {}
        }
        packet_num += 1;
    }

    fs::write("files/out.h264", entire_video_bytes).unwrap();
}
