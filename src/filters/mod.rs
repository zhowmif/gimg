use crate::stream::Stream;

pub mod grayscale;
// pub mod crop;

pub trait Filter: Stream {
    fn filter_stream(stream: Box<dyn Stream>) -> Self;
}
