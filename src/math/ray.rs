use super::Vec3;

#[derive(Copy, Clone)]
pub struct Ray {
    pub origin:    Vec3,
    pub direction: Vec3,
}

impl Ray {
    #[inline(always)]
    pub fn new_normalized(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction,
        }
    }

    #[inline(always)]
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalized(),
        }
    }

    pub fn point(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}
