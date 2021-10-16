use crate::math::Vector3i16;
use crate::world::Block;
use std::collections::{HashMap, VecDeque};

pub struct Map {
    blocks: HashMap<Vector3i16, Block>,
    dirty_blocks: VecDeque<Vector3i16>,
}

impl Map {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            dirty_blocks: VecDeque::new(),
        }
    }

    pub fn get(&self, pos: &Vector3i16) -> Option<&Block> {
        self.blocks.get(pos)
    }

    pub fn update_or_set(&mut self, pos: Vector3i16, block: Block) {
        self.blocks.insert(pos.clone(), block);
        self.dirty_blocks.push_back(pos);
    }

    pub fn dirty_blocks(&mut self) -> &mut VecDeque<Vector3i16> {
        &mut self.dirty_blocks
    }
}
