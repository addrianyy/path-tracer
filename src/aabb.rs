use crate::vec::Vec3;
use crate::ray::Ray;

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

    pub fn hits_ray(&self, ray: &Ray, mut min_t: f32, mut max_t: f32) -> bool {
        for a in 0..3 {
            let inv_d  = 1.0 / ray.get_direction()[a];
            let mut t0 = (self.min[a] - ray.get_origin()[a]) * inv_d;
            let mut t1 = (self.max[a] - ray.get_origin()[a]) * inv_d;

            if inv_d < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }

            min_t = if t0 > min_t { t0 } else { min_t };
            max_t = if t1 < max_t { t1 } else { max_t };

            if max_t <= min_t {
                return false;
            }
        }

        true
    }

    pub fn surrounding_box(box0: &AABB, box1: &AABB) -> AABB {
        let min = |a: f32, b: f32| if a > b { b } else { a };
        let max = |a: f32, b: f32| if a < b { b } else { a };

        let bbmin = Vec3::new(
            min(box0.min.x, box1.min.x),
            min(box0.min.y, box1.min.y),
            min(box0.min.z, box1.min.z));

        let bbmax = Vec3::new(
            max(box0.max.x, box1.max.x),
            max(box0.max.y, box1.max.y),
            max(box0.max.z, box1.max.z));
        
        AABB::new(bbmin, bbmax)
    }
}
