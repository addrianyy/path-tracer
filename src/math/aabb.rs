use super::{Vec3, Ray};

#[derive(Copy, Clone)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self {
            min,
            max,
        }
    }

    pub fn extent(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5 
    }

    pub fn intersect(&self, ray: &Ray, inv_direction: Vec3, min_t: f32, max_t: f32) -> bool {
        let v0 = (self.min - ray.origin) * inv_direction;
        let v1 = (self.max - ray.origin) * inv_direction;

        let min = Vec3::min(v0, v1).extract();
        let max = Vec3::max(v0, v1).extract();

        let min_t = f32::max(min_t, f32::max(min.0, f32::max(min.1, min.2)));
        let max_t = f32::min(max_t, f32::min(max.0, f32::min(max.1, max.2)));

        min_t < max_t
    }

    pub fn enclosing_box(box0: &AABB, box1: &AABB) -> AABB {
        let bbmin = Vec3::min(box0.min, box1.min);
        let bbmax = Vec3::max(box0.max, box1.max);
        
        AABB::new(bbmin, bbmax)
    }

    pub fn volume(&self) -> f32 {
        let (x, y, z) = self.extent().extract();

        (x * y * z).abs()
    }
}
