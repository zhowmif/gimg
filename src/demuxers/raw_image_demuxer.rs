use crate::{image::{Image, Resolution}, stream::Stream};

pub struct RawImageDemuxer {
    image: Image,
    is_consumed: bool,
}

impl RawImageDemuxer {
    pub fn new(image: Image) -> Self {
        Self {
            image,
            is_consumed: false,
        }
    }
}

impl Stream for RawImageDemuxer {
    fn get_next_image(&mut self) -> Option<Image> {
        if self.is_consumed {
            return None;
        }
        self.is_consumed = true;

        Some(self.image.clone())
    }

    fn get_resolution(&self) -> Resolution {
        self.image.resolution
    }
}
