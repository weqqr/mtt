use crate::world::node::Node;
use mtt_serialize::Serialize;
use std::io::{Cursor, Read, Write};

#[derive(Debug, Clone)]
pub struct Block {
    node_data: Vec<u8>,
}

impl Block {
    pub const SIZE: usize = 16;
    pub const VOLUME: usize = Block::SIZE.pow(3);

    pub fn node(&self, x: usize, y: usize, z: usize) -> Node {
        let index = z * Block::SIZE * Block::SIZE + y * Block::SIZE + x;
        let id_hi = self.node_data[2 * index];
        let id_lo = self.node_data[2 * index + 1];
        let param1 = self.node_data[2 * Block::VOLUME + index];
        let param2 = self.node_data[3 * Block::VOLUME + index];

        Node {
            id: ((id_hi as u16) << 8) | id_lo as u16,
            param1,
            param2,
        }
    }
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

        let r = &mut Cursor::new(data);

        let _flags = u8::deserialize(r)?;
        let _lighting_complete = u16::deserialize(r)?;

        let content_width = u8::deserialize(r)?;
        let params_width = u8::deserialize(r)?;
        anyhow::ensure!(content_width == 2, "invalid content width");
        anyhow::ensure!(params_width == 2, "invalid params width");

        let mut node_data = vec![0; Block::VOLUME * 4];
        r.read_exact(&mut node_data)?;

        Ok(Self { node_data })
    }
}
