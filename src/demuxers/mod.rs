use crate::stream::Stream;

pub mod image_demuxer;

pub trait Demuxer {
    fn get_stream(self) -> Stream;
}
