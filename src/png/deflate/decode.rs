use super::{new_bitsream::BitStreamReader, BlockType};

pub fn decode_deflate(bytes: &[u8]) -> Vec<u8> {
    let mut bitsream = BitStreamReader::new(bytes);
    let mut result = Vec::new();

    loop {
        let is_last = bitsream.read_bit_boolean();
        let btype = BlockType::from_number(bitsream.read_number_lsb(2) as u8);

        match btype {
            BlockType::None => parse_block_type_zero(&mut bitsream, &mut result),
            BlockType::FixedHuffman => parse_block_type_zero(&mut bitsream, &mut result),
            BlockType::DynamicHuffman => parse_block_type_zero(&mut bitsream, &mut result),
        }

        if is_last {
            break;
        }
    }

    result
}

fn parse_block_type_zero(reader: &mut BitStreamReader, target: &mut Vec<u8>) {
    reader.align_to_next_byte();
    let len = reader.read_u16_lsb_le();
    let _nlen = reader.read_u16_lsb_le();
    let bytes = reader.read_bytes_aligned(len as usize);

    target.extend_from_slice(bytes);
}

fn parse_block_type_one(reader: &mut BitStreamReader, target: &mut Vec<u8>) {}

fn parse_block_type_two(reader: &mut BitStreamReader, target: &mut Vec<u8>) {}
