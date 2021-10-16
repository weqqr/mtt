use crate::serialize::Serialize;
use std::io::{Cursor, Read, Write};

#[derive(Debug, Clone)]
pub struct Block {
    node_data: Vec<u8>,
}

impl Block {
    pub const SIZE: usize = 16;
}

impl Serialize for Block {
    fn serialize<W: Write>(&self, _w: &mut W) -> anyhow::Result<()> {
        todo!()
    }

    fn deserialize<R: Read>(r: &mut R) -> anyhow::Result<Self> {
        // FIXME: take length directly from reader
        let mut data = Vec::new();
        r.read_to_end(&mut data)?;
        let len = data.len() as u64;

        // Due to the legendary amount of legacy, server sends an additional
        // uncompressed byte at the end. This byte confuses zstd reader into
        // reading an additional frame, and the reader fails to recognize that
        // there just isn't enough data to decompress anything.
        //
        // Since the byte isn't very important, cutting it off seems reasonable.
        let mut decoder = zstd::Decoder::new(Cursor::new(data).take(len - 1))?;
        let mut data = Vec::new();
        decoder.read_to_end(&mut data)?;

        let mut reader = Cursor::new(data);

        let _flags = u8::deserialize(&mut reader)?;
        let _lighting_complete = u16::deserialize(&mut reader)?;

        let content_width = u8::deserialize(&mut reader)?;
        let params_width = u8::deserialize(&mut reader)?;
        anyhow::ensure!(content_width == 2, "invalid content width");
        anyhow::ensure!(params_width == 2, "invalid params width");

        let mut node_data = vec![0; Block::SIZE.pow(3) * 4];
        reader.read_exact(&mut node_data)?;

        Ok(Self { node_data })
    }
}
