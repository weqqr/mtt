use glam::I16Vec3;

use crate::world::Block;
use std::collections::{HashMap, VecDeque};

pub struct Map {
    blocks: HashMap<I16Vec3, Block>,
    dirty_blocks: VecDeque<I16Vec3>,
}

impl Map {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            dirty_blocks: VecDeque::new(),
        }
    }

    pub fn get(&self, pos: &I16Vec3) -> Option<&Block> {
        self.blocks.get(pos)
    }

    pub fn update_or_set(&mut self, pos: I16Vec3, block: Block) {
        self.blocks.insert(pos, block);
        self.dirty_blocks.push_back(pos);
    }

    pub fn dirty_blocks(&mut self) -> VecDeque<I16Vec3> {
        let mut dirty = VecDeque::new();
        std::mem::swap(&mut dirty, &mut self.dirty_blocks);
        dirty
    }
}
