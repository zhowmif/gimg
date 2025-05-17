use crate::image::{Image, Resolution};

pub trait Stream {
    fn get_next_image(&mut self) -> Option<Image>;
    fn get_resolution(&self) -> Resolution;
}
