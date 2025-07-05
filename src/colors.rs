use crate::algebra::{Matrix3, Vec3};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl From<&Rgb> for Vec3 {
    fn from(val: &Rgb) -> Self {
        Vec3([val.r as f32, val.g as f32, val.b as f32])
    }
}

impl From<Vec3> for Rgb {
    fn from(value: Vec3) -> Self {
        Rgb {
            r: value.0[0].round() as u8,
            g: value.0[1].round() as u8,
            b: value.0[2].round() as u8,
        }
    }
}

impl From<&Rgb> for Vec<u8> {
    fn from(rgb: &Rgb) -> Self {
        vec![rgb.r, rgb.g, rgb.b]
    }
}

#[derive(Debug, Clone, Copy)]
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

impl From<&YCbCr> for Vec3 {
    fn from(val: &YCbCr) -> Self {
        Vec3([val.y as f32, val.cb as f32, val.cr as f32])
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

impl From<Rgba> for YCbCr {
    fn from(value: Rgba) -> Self {
        let rgb: Rgb = value.into();

        (&rgb).into()
    }
}

impl From<&Rgba> for YCbCr {
    fn from(value: &Rgba) -> Self {
        let rgb: Rgb = value.into();

        (&rgb).into()
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

impl From<&Rgb> for YCbCr {
    fn from(rgb: &Rgb) -> Self {
        let rgb_vec: Vec3 = rgb.into();

        Self::from(RGB_TO_YCBCR_CONVERION_OFFSET + rgb_vec * RGB_TO_YCBCR_CONVERSION_TABLE)
    }
}

impl From<Rgb> for Rgba {
    fn from(val: Rgb) -> Self {
        Rgba {
            r: val.r,
            g: val.g,
            b: val.b,
            a: 255,
        }
    }
}

impl From<YCbCr> for Rgba {
    fn from(val: YCbCr) -> Self {
        let rgb: Rgb = Rgb::from(&val);

        rgb.into()
    }
}

impl From<&YCbCr> for Rgb {
    fn from(ycbcr: &YCbCr) -> Self {
        let vec: Vec3 = ycbcr.into();

        Rgb::from((vec - RGB_TO_YCBCR_CONVERION_OFFSET) * YCBCR_TO_RGB_CONVERSION_TABLE)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn is_opaque(&self) -> bool {
        self.a == u8::MAX
    }

    pub fn is_greyscale(&self) -> bool {
        self.r == self.g && self.r == self.b
    }
}

impl From<Rgba> for Rgb {
    fn from(val: Rgba) -> Self {
        Rgb {
            r: val.r,
            g: val.g,
            b: val.b,
        }
    }
}

impl From<&Rgba> for Rgb {
    fn from(val: &Rgba) -> Self {
        Rgb {
            r: val.r,
            g: val.g,
            b: val.b,
        }
    }
}
