pub mod backreference;
mod bitstream_lzss;
mod hash;

use hash::LzssHashTable;

#[derive(Debug)]
pub enum LzssSymbol {
    Literal(u8),
    Backreference(u16, u8),
    EndOfBlock
}

pub fn encode_lzss(bytes: &[u8], window_size: usize) -> Vec<LzssSymbol> {
    let mut cursor = 0;
    let mut stream = Vec::with_capacity(bytes.len() / 2);
    let mut table = LzssHashTable::new();

    while cursor < bytes.len() {
        stream.push(
            match find_backreference_with_table(bytes, cursor, window_size, &mut table) {
                Some(backreference) => {
                    let symbol = LzssSymbol::Backreference(backreference.0, backreference.1);
                    cursor += backreference.1 as usize;

                    symbol
                }
                None => {
                    let symbol = LzssSymbol::Literal(bytes[cursor]);
                    cursor += 1;

                    symbol
                }
            },
        );
    }

    stream
}

fn find_backreference_with_table(
    bytes: &[u8],
    cursor: usize,
    window_size: usize,
    table: &mut LzssHashTable,
) -> Option<(u16, u8)> {
    //TODO: this should only look ahead 258 bytes
    let best_match = table.search(bytes, cursor, cursor.max(window_size) - window_size);

    if cursor + 2 < bytes.len() {
        let key = (bytes[cursor], bytes[cursor + 1], bytes[cursor + 2]);
        table.insert(key, cursor);
    }

    best_match
}

pub fn decode_lzss(lzss_symbols: &[LzssSymbol]) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();

    for symbol in lzss_symbols {
        match symbol {
            LzssSymbol::Literal(literal) => result.push(*literal),
            LzssSymbol::Backreference(distance, length) => {
                let backreference_data_start = result.len() - *distance as usize;

                //we must do this byte by byte in case there are repetitions
                for i in backreference_data_start..backreference_data_start + *length as usize {
                    result.push(result[i]);
                }
            }
            LzssSymbol::EndOfBlock => todo!("What to do what to do"),
        }
    }

    result
}
