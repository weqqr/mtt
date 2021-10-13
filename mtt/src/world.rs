use crate::math::Vector3;

pub struct Player {
    position: Vector3,
}

impl Player {
    pub fn new() -> Self {
        Self {
            position: Vector3::ZERO,
        }
    }
}

pub struct World {
    pub player: Player,
    pub time: f32,
    pub time_speed: f32,
}

impl World {
    pub fn new() -> Self {
        Self {
            player: Player::new(),
            time: 0.0,
            time_speed: 0.0,
        }
    }
}
