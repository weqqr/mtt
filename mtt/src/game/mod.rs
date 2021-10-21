pub mod node;

use crate::game::node::{Argb, DrawType, Lighting, Node, Rgb};
use crate::serialize::Serialize;
use anyhow::Result;
use flate2::read::ZlibDecoder;
use log::info;
use std::collections::HashMap;
use std::io::{Cursor, Read};

pub struct Item {}

pub struct Game {
    pub items: Vec<Item>,
    pub nodes: Vec<Node>,
    air: Node,
}

impl Game {
    pub fn new() -> Self {
        let air = Node {
            name: "air".to_string(),
            groups: HashMap::new(),
            param_type1: 0,
            param_type2: 0,
            draw_type: DrawType::AirLike,
            mesh: None,
            visual_scale: 1.0,
            tiles: Vec::new(),
            tiles_overlay: Vec::new(),
            tiles_special: Vec::new(),
            color: Rgb { r: 0, g: 0, b: 0 },
            palette_name: "".to_string(),
            waving: 0,
            connect_sides: 0,
            connects_to: Vec::new(),
            post_effect_color: Argb { a: 0, r: 0, g: 0, b: 0 },
            leveled: 0,
            lighting: Lighting {
                light_propagates: true,
                sunlight_propagates: true,
                light_source: 0,
            },
            is_ground_content: true,
        };

        Self {
            items: Vec::new(),
            nodes: Vec::new(),
            air,
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
            let id = u16::deserialize(&mut reader)? as usize;
            let node = Node::deserialize(&mut reader)?;

            if id >= game.nodes.len() {
                game.nodes.resize(id + 1, game.air.clone());
            }

            game.nodes[id] = node;
        }

        Ok(game)
    }
}
