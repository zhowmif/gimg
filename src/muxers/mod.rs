use crate::stream::Stream;

pub mod show_demuxer;

pub trait Muxer {
    fn consume_stream(self, stream: impl Stream);
}
