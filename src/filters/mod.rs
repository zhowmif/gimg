use crate::stream::Stream;

pub mod grayscale;

pub trait Filter {
    fn filter_stream(&self, input: Stream) -> Stream;
}
