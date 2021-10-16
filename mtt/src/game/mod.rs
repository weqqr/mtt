pub mod node;

use crate::game::node::Node;
use crate::serialize::Serialize;
use anyhow::Result;
use flate2::read::ZlibDecoder;
use log::info;
use std::io::{Cursor, Read};

pub struct Item {}

pub struct Game {
    items: Vec<Item>,
    nodes: Vec<Node>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            nodes: Vec::new(),
        }
    }

    pub fn deserialize_nodes(data: &[u8]) -> Result<Game> {
        let mut game = Game::new();

        let mut reader = ZlibDecoder::new(data);
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        let mut reader = Cursor::new(data);
        let version = u8::deserialize(&mut reader)?;
        anyhow::ensure!(version == 1);

        let count = u16::deserialize(&mut reader)?;
        info!("Deserializing {} nodes", count);

        // Total length of serialized nodes
        let _ = u32::deserialize(&mut reader)?;

        for _ in 0..count {
            let _id = u16::deserialize(&mut reader)?;
            let node = Node::deserialize(&mut reader)?;

            // if id as usize >= game.nodes.len() {
            //     game.nodes.resize(id as usize + 1, AIR);
            // }

            game.nodes.push(node);
        }

        Ok(game)
    }
}
