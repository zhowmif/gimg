use crate::{
    colors::{YCbCr, RGB},
    image::Image,
    stream::Stream,
};

use super::Filter;

pub struct GrayScaleFilter;

impl GrayScaleFilter {
    fn convert_to_grayscale(&self, mut image: Image) -> Image {
        for row in 0..image.resolution.height {
            for col in 0..image.resolution.width {
                let current_pixel = &image.pixels[row as usize][col as usize];
                let luma = YCbCr::from(current_pixel).y;

                image.pixels[row as usize][col as usize] = RGB::new(luma, luma, luma);
            }
        }

        image
    }
}

impl Filter for GrayScaleFilter {
    fn filter_stream(&self, input: Stream) -> Stream {
        //this is so dumb...
        let vec: Vec<Image> = input
            .iterator
            .map(|image| {
                self.convert_to_grayscale(image)
            })
            .collect();

        Stream::new(input.resolution, Box::new(vec.into_iter()))
    }
}
