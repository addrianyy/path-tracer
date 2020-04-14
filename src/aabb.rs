use crate::vec::Vec3;
use crate::ray::Ray;

#[derive(Default, Copy, Clone)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> AABB {
        AABB {
            min,
            max,
        }
    }

    pub fn hit(&self, ray: &Ray, mut min_t: f32, mut max_t: f32) -> bool {
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
}
