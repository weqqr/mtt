use bytemuck::{Pod, Zeroable};
use mtt_macros::Serialize;
use std::ops::Div;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub const ZERO: Vector3 = Vector3::new(0.0, 0.0, 0.0);
    pub const UP: Vector3 = Vector3::new(0.0, 1.0, 0.0);

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn from_euler_angles(pitch: f32, yaw: f32) -> Self {
        // FIXME: figure out coordinate conversion
        let pitch = -pitch.to_radians();
        let yaw = (yaw + 90.0).to_radians();

        let x = yaw.cos() * pitch.cos();
        let y = pitch.sin();
        let z = yaw.sin() * pitch.cos();
        Self { x, y, z }.normalize()
    }

    pub fn normalize(&self) -> Self {
        let len = self.len();
        Self {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
        }
    }

    pub fn len(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn extend(&self, w: f32) -> Vector4 {
        Vector4 {
            x: self.x,
            y: self.y,
            z: self.z,
            w,
        }
    }
}

impl Div<f32> for Vector3 {
    type Output = Vector3;

    fn div(self, rhs: f32) -> Self::Output {
        let reciprocal = rhs.recip();
        Self {
            x: self.x * reciprocal,
            y: self.y * reciprocal,
            z: self.z * reciprocal,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Pod, Zeroable)]
#[repr(C)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vector4 {
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize)]
pub struct Vector3i16 {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

impl Vector3i16 {
    pub const fn new(x: i16, y: i16, z: i16) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize)]
pub struct Vector3u16 {
    pub x: u16,
    pub y: u16,
    pub z: u16,
}

impl Vector3u16 {
    pub const fn new(x: u16, y: u16, z: u16) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Aabb {
    min: Vector3,
    max: Vector3,
}
