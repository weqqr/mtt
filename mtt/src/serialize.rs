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
