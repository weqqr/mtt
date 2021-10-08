use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
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

impl Serialize for String {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        let size = self.len();
        assert!(size < u16::MAX as usize);
        (size as u16).serialize(w)?;
        w.write_all(self.as_bytes())?;
        Ok(())
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let size = u16::deserialize(r)? as usize;
        let mut data = vec![0; size];
        r.read_exact(&mut data)?;
        Ok(String::from_utf8(data)?)
    }
}

#[derive(Debug, Clone)]
pub struct RawBytes16(pub Vec<u8>);

impl Into<Vec<u8>> for RawBytes16 {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

impl Serialize for RawBytes16 {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        let size = self.0.len();
        assert!(size < u16::MAX as usize);
        (size as u16).serialize(w)?;
        w.write_all(&self.0)?;
        Ok(())
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let size = u16::deserialize(r)? as usize;
        let mut data = vec![0; size];
        r.read_exact(&mut data)?;
        Ok(RawBytes16(data))
    }
}

#[derive(Debug, Clone)]
pub struct RawBytes32(pub Vec<u8>);

impl Into<Vec<u8>> for RawBytes32 {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

impl Serialize for RawBytes32 {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        let size = self.0.len();
        assert!(size < u32::MAX as usize);
        (size as u32).serialize(w)?;
        w.write_all(&self.0)?;
        Ok(())
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let size = u32::deserialize(r)? as usize;
        let mut data = vec![0; size];
        r.read_exact(&mut data)?;
        Ok(RawBytes32(data))
    }
}
