use crate::image::Resolution;

pub fn assset_bytestream_size_fits_resolution(bytes: &Vec<u8>, resolution: Resolution) {
    assert!(
        bytes.len() == resolution.height * resolution.width * 3,
        "Tried parsing {}x{} image, but input bytes were length {}",
        resolution.width,
        resolution.height,
        bytes.len()
    );
}
