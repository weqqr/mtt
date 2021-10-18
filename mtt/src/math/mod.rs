use bytemuck::{Pod, Zeroable};
use mtt_macros::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub const ZERO: Vector3 = Vector3::new(0.0, 0.0, 0.0);

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
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

#[derive(Debug, Clone, Serialize)]
pub struct Aabb {
    min: Vector3,
    max: Vector3,
}
