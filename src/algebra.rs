use std::{
    fmt::Display,
    ops::{Add, Mul, Sub},
};

pub fn align_up(n: usize, alignment: usize) -> usize {
    let remainder = n % alignment;

    if remainder == 0 {
        n
    } else {
        n - remainder + alignment
    }
}

pub struct Matrix3(pub [Vec3; 3]);

impl Matrix3 {
    pub const fn new(x: [f32; 3], y: [f32; 3], z: [f32; 3]) -> Self {
        Self([Vec3(x), Vec3(y), Vec3(z)])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Vec3(pub [f32; 3]);

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self([x, y, z])
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Self::Output {
        Self([self.0[0] * rhs, self.0[1] * rhs, self.0[2] * rhs])
    }
}

impl Add<Vec3> for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Self::Output {
        Self([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
        ])
    }
}

impl Sub<Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Self::Output {
        Self([
            self.0[0] - rhs.0[0],
            self.0[1] - rhs.0[1],
            self.0[2] - rhs.0[2],
        ])
    }
}

impl Mul<Matrix3> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Matrix3) -> Self::Output {
        rhs.0[0] * self.0[0] + rhs.0[1] * self.0[1] + rhs.0[2] * self.0[2]
    }
}

impl From<[f32; 3]> for Vec3 {
    fn from(value: [f32; 3]) -> Self {
        Self(value)
    }
}

impl From<[[f32; 3]; 3]> for Matrix3 {
    fn from(value: [[f32; 3]; 3]) -> Self {
        Self(value.map(|a| a.into()))
    }
}

impl Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}, {}]", self.0[0], self.0[1], self.0[2])
    }
}
