use glam::Vec3;
use mtt_macros::Serialize;

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
    min: Vec3,
    max: Vec3,
}
