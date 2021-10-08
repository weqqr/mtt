use crate::serialize::Serialize;
use anyhow::Result;
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
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
