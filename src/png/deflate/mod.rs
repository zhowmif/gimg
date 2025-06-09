mod bitstream;
mod consts;
pub mod lzss;
pub mod new_bitsream;
pub mod zlib;

use consts::MAX_UNCOMPRESSED_BLOCK_SIZE;
use new_bitsream::NewBitStream;
use zlib::zlib_encode;

pub fn compress_scanlines(scanlines: &Vec<Vec<u8>>) -> Vec<u8> {
    let mut encoder = DeflateEncoder::new(BlockType::None);

    for scanline in scanlines {
        encoder.write_bytes(&scanline);
    }

    let zlib_encoded = zlib_encode(encoder);
    let compressed = zlib_encoded.to_bytes();

    compressed
}

// pub fn compress_scanlines(scanlines: &Vec<Vec<u8>>) -> Vec<u8> {
//     let mut e = ZlibEncoder::new(Vec::new(), Compression::none());
//
//     for scanline in scanlines {
//         e.write_all(&scanline).expect("Deflate writing failed");
//     }
//
//     let compressed = e.finish().unwrap();
//
//     compressed
// }

pub enum BlockType {
    None,
    FixedHuffman,
    DynamicHuffman,
}

impl BlockType {
    fn to_number(&self) -> u8 {
        match self {
            BlockType::None => 0,
            BlockType::FixedHuffman => 1,
            BlockType::DynamicHuffman => 2,
        }
    }
}

pub struct DeflateEncoder {
    block_type: BlockType,
    bytes: Vec<u8>,
}

impl DeflateEncoder {
    pub fn new(block_type: BlockType) -> Self {
        if !matches!(block_type, BlockType::None) {
            println!("Actual compression is not supported yet :)");
        }

        Self {
            block_type,
            bytes: vec![],
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    pub fn uncompreseed(&self) -> &[u8] {
        &self.bytes
    }

    pub fn finish(&mut self) -> NewBitStream {
        let mut bitstream = NewBitStream::new();

        for (block_index, block_bytes) in self
            .bytes
            .chunks(MAX_UNCOMPRESSED_BLOCK_SIZE.into())
            .enumerate()
        {
            let is_last = block_index == self.bytes.len() / MAX_UNCOMPRESSED_BLOCK_SIZE as usize;

            if is_last {
                bitstream.push_one();
            } else {
                bitstream.push_zero();
            }
            bitstream.push_u8_lsb(BlockType::None.to_number().into(), 2);
            //padding
            bitstream.push_u8_lsb(0, 5);

            let len = block_bytes.len() as u16;
            bitstream.push_u16_lsb(len);
            bitstream.push_u16_lsb(!len);

            for byte in block_bytes {
                bitstream.push_byte_lsb(*byte);
            }
        }

        bitstream
    }
}
