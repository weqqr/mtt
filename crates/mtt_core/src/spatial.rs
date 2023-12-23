use glam::Vec3;
use mtt_macros::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Aabb {
    min: Vec3,
    max: Vec3,
}
