pub mod block;
pub mod map;
pub mod node;

pub use self::block::Block;
use crate::math::Vector3;
pub use crate::world::map::Map;

pub struct Player {
    pub position: Vector3,
    pub look_dir: Vector3,
}

impl Player {
    pub fn new() -> Self {
        Self {
            position: Vector3::new(-10.0, 10.0, -10.0),
            look_dir: Vector3::new(1.0, -1.0, 1.0).normalize(),
        }
    }
}

pub struct WorldState {
    pub player: Player,
    pub time: f32,
    pub time_speed: f32,
    pub map: Map,
}

impl WorldState {
    pub fn new() -> Self {
        Self {
            player: Player::new(),
            time: 0.0,
            time_speed: 0.0,
            map: Map::new(),
        }
    }
}
