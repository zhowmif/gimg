use crate::stream::Stream;

pub mod image_demuxer;
pub mod droidcam;

pub trait Demuxer: Stream {}
