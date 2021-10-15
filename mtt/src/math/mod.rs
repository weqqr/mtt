use crate::serialize::Serialize;
use anyhow::Result;
use mtt_macros::Serialize;
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy)]
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

impl Serialize for Vector3 {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        self.x.serialize(w)?;
        self.y.serialize(w)?;
        self.z.serialize(w)
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        Ok(Self {
            x: f32::deserialize(r)?,
            y: f32::deserialize(r)?,
            z: f32::deserialize(r)?,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct Aabb {
    min: Vector3,
    max: Vector3,
}
