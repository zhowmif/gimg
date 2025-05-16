use crate::image::{Image, Resolution};

pub struct Stream {
    pub resolution: Resolution,
    pub iterator: Box<dyn Iterator<Item = Image>>,
}

impl Stream {
    pub fn new(resolution: Resolution, iterator: Box<dyn Iterator<Item = Image>>) -> Self {
        Self {
            resolution,
            iterator,
        }
    }
}
