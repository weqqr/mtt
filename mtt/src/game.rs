use std::collections::HashMap;

pub struct ItemDef {}

pub struct NodeDef {}

pub struct Game {
    items: HashMap<String, ItemDef>,
    nodes: HashMap<String, NodeDef>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            nodes: HashMap::new(),
        }
    }
}
