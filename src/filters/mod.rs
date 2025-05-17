use crate::stream::Stream;

pub mod grayscale;

pub trait Filter: Stream {
    fn filter_stream(stream: Box<dyn Stream>) -> Self;
}
