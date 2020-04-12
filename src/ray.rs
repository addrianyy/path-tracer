use crate::vec::Vec3;

#[derive(Copy, Clone)]
pub struct Ray {
    origin:    Vec3,
    direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalized(),
        }
    }

    pub fn get_point(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

    pub fn get_origin(&self) -> Vec3 {
        self.origin
    }

    pub fn get_direction(&self) -> Vec3 {
        self.direction
    }
}
