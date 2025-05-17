use std::{
    fs,
    process::{Child, Command},
};

use v4l::{
    buffer::Type,
    format::FieldOrder,
    io::{mmap, traits::OutputStream},
    prelude::MmapStream,
    video::Output,
    Device, Format, FourCC,
};

use crate::{
    colors::RGB,
    image::{Image, Resolution},
    pixel_formats::{yuv420p::YUV420p, PixelFormat},
    stream::Stream,
};

pub(crate) struct DroidCamDemuxer<'a> {
    device: Device,
    format: Format,
    stream: mmap::Stream<'a>,
    droidcam_process: Child,
}

impl<'a> DroidCamDemuxer<'a> {
    pub fn new() -> Self {
        let droidcam_process = DroidCamDemuxer::start_streaming_videoo();
        let device = Device::new(0).expect("Failed to open device");
        let format = device.format().expect("Failed to read format");
        // format.field_order = FieldOrder::Progressive;
        // format.fourcc = FourCC::new()
        // let format = device.set_format(&format).expect("Failed to set format");
        let stream = mmap::Stream::with_buffers(&device, Type::VideoCapture, 4)
            .expect("Failed to create stream");

        Self {
            device,
            format,
            stream,
            droidcam_process,
        }
    }

    fn start_streaming_videoo() -> Child {
        Command::new("sudo")
            .args(["modprobe", "v4l2loopback"])
            .status()
            .expect("Failed to load v4l2loopback module");
        Command::new("droidcam-cli")
            .args(["adb", "4747"])
            .spawn()
            .expect("Failed to start droidcam")
    }
}

impl<'a> Drop for DroidCamDemuxer<'a> {
    fn drop(&mut self) {
        self.droidcam_process.kill().unwrap();
    }
}

impl<'a> Stream for DroidCamDemuxer<'a> {
    fn get_next_image(&mut self) -> Option<Image> {
        let resolution = self.get_resolution();
        let (buf, _meta) = self.stream.next().unwrap();
        // let (buf, _meta) = self.stream.next().unwrap();
        // println!("format {:?}", self.format);
        // println!("byte length! {:?}", buf.len());
        // let mut res: Vec<u8> = Vec::with_capacity(buf.len());
        // for i in 0..(buf.len() / 3) {
        //     res.push(buf[i]);
        //     res.push(buf[i + buf.len() / 3]);
        //     res.push(buf[i + buf.len() / 3]);
        // }
        fs::write("sup.yuv", &buf).unwrap();
        let pix_fmt = YUV420p;

        let pixels = pix_fmt.parse_bytestream(buf, resolution);
        println!(
            "first pixel {:?} {:?}",
            pixels[0][0],
            RGB::from(&pixels[0][0]),
        );

        Some(Image::new(self.get_resolution(), pixels))
    }

    fn get_resolution(&self) -> Resolution {
        Resolution {
            width: self.format.width as usize,
            height: self.format.height as usize,
        }
    }
}
