use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use glam::{Vec3, vec3, i16vec3, I16Vec3};
use std::io::{Read, Write};

pub trait Serialize: Sized {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()>;
    fn deserialize<R: Read>(r: &mut R) -> Result<Self>;
}

macro_rules! impl_serialize_for_primitive {
    ($primitive:ty, $read_fn:ident, $write_fn:ident) => {
        impl Serialize for $primitive {
            fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
                Ok(w.$write_fn(*self)?)
            }

            fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
                Ok(r.$read_fn()?)
            }
        }
    };

    ($primitive:ty, $read_fn:ident, $write_fn:ident, $endianness:ident) => {
        impl Serialize for $primitive {
            fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
                Ok(w.$write_fn::<$endianness>(*self)?)
            }

            fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
                Ok(r.$read_fn::<$endianness>()?)
            }
        }
    };
}

impl_serialize_for_primitive!(u8, read_u8, write_u8);
impl_serialize_for_primitive!(u16, read_u16, write_u16, BigEndian);
impl_serialize_for_primitive!(u32, read_u32, write_u32, BigEndian);
impl_serialize_for_primitive!(u64, read_u64, write_u64, BigEndian);

impl_serialize_for_primitive!(i8, read_i8, write_i8);
impl_serialize_for_primitive!(i16, read_i16, write_i16, BigEndian);
impl_serialize_for_primitive!(i32, read_i32, write_i32, BigEndian);
impl_serialize_for_primitive!(i64, read_i64, write_i64, BigEndian);

impl_serialize_for_primitive!(f32, read_f32, write_f32, BigEndian);
impl_serialize_for_primitive!(f64, read_f64, write_f64, BigEndian);

impl Serialize for bool {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        (*self as u8).serialize(w)
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        u8::deserialize(r).map(|value| value != 0)
    }
}

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

pub trait StringSerializeExt: Sized {
    fn deserialize_utf16<R: Read>(r: &mut R) -> Result<Self>;
}

impl StringSerializeExt for String {
    fn deserialize_utf16<R: Read>(r: &mut R) -> Result<Self> {
        let len = u16::deserialize(r)? as usize;
        let mut data = Vec::with_capacity(len);

        for _ in 0..len {
            data.push(u16::deserialize(r)?);
        }

        Ok(String::from_utf16(&data)?)
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

impl Serialize for Vec3 {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        self.x.serialize(w)?;
        self.y.serialize(w)?;
        self.z.serialize(w)?;

        Ok(())
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let x = f32::deserialize(r)?;
        let y = f32::deserialize(r)?;
        let z = f32::deserialize(r)?;

        Ok(vec3(x, y, z))
    }
}

impl Serialize for I16Vec3 {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        self.x.serialize(w)?;
        self.y.serialize(w)?;
        self.z.serialize(w)?;

        Ok(())
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let x = i16::deserialize(r)?;
        let y = i16::deserialize(r)?;
        let z = i16::deserialize(r)?;

        Ok(i16vec3(x, y, z))
    }
}

