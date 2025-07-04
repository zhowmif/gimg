use crate::stream::Stream;

pub mod image_demuxer;
pub mod raw_image_demuxer;

pub trait Demuxer: Stream {}
