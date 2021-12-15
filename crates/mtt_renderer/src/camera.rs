use nalgebra_glm::{Vec3, Mat4};

pub struct Camera {
    position: Vec3,
    pitch: f32,
    yaw: f32,
    fov: f32,
}

impl Camera {
    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        let projection = Mat4::new_perspective(aspect, self.fov, 0.1, 1000.0);
        let position = Mat4::new_translation(&self.position);
        let rotation = Mat4::from_euler_angles(0.0, self.pitch, self.yaw);

        projection * position * rotation
    }
}
