use crate::{
    colors::{YCbCr, RGB},
    image::{Image, Resolution},
    stream::Stream,
};

use super::Filter;

pub struct CropFilter {
    target_resolution: Resolution,
    previous_stream: Box<dyn Stream>,
}

impl CropFilter {
    fn new(target_resolution: Resolution) -> Self {
        Self { target_resolution }
    }
}

impl Stream for CropFilter {
    fn get_next_image(&mut self) -> Option<Image> {
        let image = self.previous_stream.get_next_image();

        image.map(|image| {
            let grayscale_pixels: Vec<Vec<YCbCr>> = image
                .pixels
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|pixel| {
                            let new_value = YCbCr::from(&RGB::new(pixel.y, pixel.y, pixel.y));

                            new_value
                        })
                        .collect()
                })
                .collect();
            Image::new(image.resolution, grayscale_pixels)
        })
    }

    fn get_resolution(&self) -> Resolution {
        self.previous_stream.get_resolution()
    }
}

impl Filter for CropFilter {
    fn filter_stream(stream: Box<dyn Stream>) -> Self {
        Self {
            previous_stream: stream,
        }
    }
}
