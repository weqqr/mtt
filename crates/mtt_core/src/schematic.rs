use crate::math::Vector3u16;
use crate::world::node::Node;
use anyhow::{ensure, Result};
use flate2::read::ZlibDecoder;
use mtt_serialize::Serialize;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::Path;

#[allow(dead_code)]
pub struct Schematic {
    node_data: Vec<u8>,
    mappings: Vec<String>,
    size_x: usize,
    size_y: usize,
    size_z: usize,
    volume: usize,
}

impl Schematic {
    const MAGIC: u32 = 0x4D54534D;

    pub fn new(data: &[u8]) -> Result<Self> {
        let r = &mut Cursor::new(data);
        let magic = u32::deserialize(r)?;

        ensure!(magic == Schematic::MAGIC, "invalid schematic header: {:X}", magic);
        let version = u16::deserialize(r)?;
        ensure!(version == 4, "only schematic version 4 is supported");
        let size = Vector3u16::deserialize(r)?;
        let size_x = size.x.into();
        let size_y = size.y.into();
        let size_z = size.z.into();

        // Skip Y probabilities
        r.seek(SeekFrom::Current(size.y as i64))?;

        let mapping_count = u16::deserialize(r)?;
        let mut mappings = Vec::new();
        for _ in 0..mapping_count {
            let name = String::deserialize(r)?;
            mappings.push(name);
        }

        const BYTES_PER_NODE: usize = 4;
        let mut node_data = vec![0; BYTES_PER_NODE * size_x * size_y * size_z];

        let mut r = ZlibDecoder::new(r);
        r.read_exact(&mut node_data)?;

        Ok(Self {
            node_data,
            mappings,
            size_x,
            size_y,
            size_z,
            volume: size_x * size_y * size_z,
        })
    }

    pub fn open<P: AsRef<Path>>(p: P) -> Result<Self> {
        let data = std::fs::read(p)?;
        Schematic::new(&data)
    }

    pub fn node(&self, position: Vector3u16) -> Node {
        let z_stride = self.size_y * self.size_x;
        let y_stride = self.size_x;
        let index = position.z as usize * z_stride + position.y as usize * y_stride + position.x as usize;

        let id_hi = self.node_data[2 * index];
        let id_lo = self.node_data[2 * index + 1];
        let param1 = self.node_data[2 * self.volume + index];
        let param2 = self.node_data[3 * self.volume + index];

        Node {
            id: (id_hi as u16) << 8 | (id_lo as u16),
            param1,
            param2,
        }
    }

    pub fn node_name(&self, id: u16) -> Option<&String> {
        self.mappings.get(id as usize)
    }
}
