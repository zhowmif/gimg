use crate::algebra::{Matrix3, Vec3};

#[derive(Debug)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl Into<Vec3> for &RGB {
    fn into(self) -> Vec3 {
        Vec3([self.r as f32, self.g as f32, self.b as f32])
    }
}

impl From<Vec3> for RGB {
    fn from(value: Vec3) -> Self {
        RGB {
            r: value.0[0].round() as u8,
            g: value.0[1].round() as u8,
            b: value.0[2].round() as u8,
        }
    }
}

impl From<&RGB> for Vec<u8> {
    fn from(rgb: &RGB) -> Self {
        vec![rgb.r, rgb.g, rgb.b]
    }
}

#[derive(Debug)]
pub struct YCbCr {
    pub y: u8,
    pub cb: u8,
    pub cr: u8,
}

impl YCbCr {
    pub fn new(y: u8, cb: u8, cr: u8) -> Self {
        Self { y, cb, cr }
    }
}

impl Into<Vec3> for &YCbCr {
    fn into(self) -> Vec3 {
        Vec3([self.y as f32, self.cb as f32, self.cr as f32])
    }
}

impl From<Vec3> for YCbCr {
    fn from(value: Vec3) -> Self {
        YCbCr {
            y: value.0[0].round() as u8,
            cb: value.0[1].round() as u8,
            cr: value.0[2].round() as u8,
        }
    }
}

const RGB_TO_YCBCR_CONVERSION_TABLE: Matrix3 = Matrix3::new(
    [0.299, -0.168935, 0.499813],
    [0.587, -0.331665, -0.418531],
    [0.114, 0.50059, -0.081282],
);
const RGB_TO_YCBCR_CONVERION_OFFSET: Vec3 = Vec3::new(0., 128., 128.);

const YCBCR_TO_RGB_CONVERSION_TABLE: Matrix3 = Matrix3::new(
    [1., 1., 1.],
    [0., -0.343729, 1.769905],
    [1.402524, -0.714401, 0.],
);

impl From<&RGB> for YCbCr {
    fn from(rgb: &RGB) -> Self {
        let rgb_vec: Vec3 = rgb.into();

        Self::from(RGB_TO_YCBCR_CONVERION_OFFSET + rgb_vec * RGB_TO_YCBCR_CONVERSION_TABLE)
    }
}

impl From<&YCbCr> for RGB {
    fn from(ycbcr: &YCbCr) -> Self {
        let vec: Vec3 = ycbcr.into();

        RGB::from((vec - RGB_TO_YCBCR_CONVERION_OFFSET) * YCBCR_TO_RGB_CONVERSION_TABLE)
    }
}
