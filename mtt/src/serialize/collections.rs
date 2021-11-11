use anyhow::Result;
use std::io::{Read, Write};
use crate::serialize::Serialize;

impl Serialize for String {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        let len = self.len();
        assert!(len < u16::MAX as usize);
        (len as u16).serialize(w)?;
        w.write_all(self.as_bytes())?;
        Ok(())
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let len = u16::deserialize(r)? as usize;
        let mut data = vec![0; len];
        r.read_exact(&mut data)?;
        Ok(String::from_utf8(data)?)
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        let len = self.len();
        anyhow::ensure!(len < u16::MAX as usize, "vec is too long to serialize");
        (len as u16).serialize(w)?;
        for value in self {
            value.serialize(w)?;
        }
        Ok(())
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let len = u16::deserialize(r)? as usize;
        let mut values = Vec::with_capacity(len);
        for _ in 0..len {
            values.push(T::deserialize(r)?);
        }
        Ok(values)
    }
}

#[derive(Debug, Clone)]
pub struct RawBytes16(pub Vec<u8>);

impl From<RawBytes16> for Vec<u8> {
    fn from(bytes: RawBytes16) -> Vec<u8> {
        bytes.0
    }
}

impl From<Vec<u8>> for RawBytes16 {
    fn from(bytes: Vec<u8>) -> Self {
        RawBytes16(bytes)
    }
}

impl Serialize for RawBytes16 {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        let len = self.0.len();
        assert!(len < u16::MAX as usize);
        (len as u16).serialize(w)?;
        w.write_all(&self.0)?;
        Ok(())
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let len = u16::deserialize(r)? as usize;
        let mut data = vec![0; len];
        r.read_exact(&mut data)?;
        Ok(Self(data))
    }
}

#[derive(Debug, Clone)]
pub struct RawBytes32(pub Vec<u8>);

impl From<RawBytes32> for Vec<u8> {
    fn from(bytes: RawBytes32) -> Vec<u8> {
        bytes.0
    }
}

impl Serialize for RawBytes32 {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        let len = self.0.len();
        assert!(len < u32::MAX as usize);
        (len as u32).serialize(w)?;
        w.write_all(&self.0)?;
        Ok(())
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let len = u32::deserialize(r)? as usize;
        let mut data = vec![0; len];
        r.read_exact(&mut data)?;
        Ok(Self(data))
    }
}

#[derive(Debug, Clone)]
pub struct RawBytesUnsized(pub Vec<u8>);

impl From<RawBytesUnsized> for Vec<u8> {
    fn from(bytes: RawBytesUnsized) -> Vec<u8> {
        bytes.0
    }
}

impl Serialize for RawBytesUnsized {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        w.write_all(&self.0)?;
        Ok(())
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let mut data = Vec::new();
        r.read_to_end(&mut data)?;
        Ok(Self(data))
    }
}
