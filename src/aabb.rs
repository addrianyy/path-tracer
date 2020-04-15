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

    pub fn extent(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5 
    }

    pub fn intersect(&self, ray: &Ray, min_t: f32, max_t: f32) -> bool {
        let calc = |a, min_t, max_t| {
            let inv = 1.0 / ray.get_direction()[a];
            let t0  = (self.min[a] - ray.get_origin()[a]) * inv;
            let t1  = (self.max[a] - ray.get_origin()[a]) * inv;

            let (t0, t1) = if inv < 0.0 { (t1, t0) } else { (t0, t1) };

            (t0.max(min_t), t1.min(max_t))
        };

        let (tmin1, tmax1) = calc(0, min_t, max_t);
        if tmax1 <= tmin1 {
            false
        } else {
            let (tmin2, tmax2) = calc(1, tmin1, tmax1);
            if tmax2 <= tmin2 {
                false
            } else {
                let (tmin3, tmax3) = calc(2, tmin2, tmax2);
                tmax3 > tmin3
            }
        }
    }

    pub fn enclosing_box(box0: &AABB, box1: &AABB) -> AABB {
        let bbmin = Vec3::new(
            f32::min(box0.min.x, box1.min.x),
            f32::min(box0.min.y, box1.min.y),
            f32::min(box0.min.z, box1.min.z));

        let bbmax = Vec3::new(
            f32::max(box0.max.x, box1.max.x),
            f32::max(box0.max.y, box1.max.y),
            f32::max(box0.max.z, box1.max.z));
        
        AABB::new(bbmin, bbmax)
    }

}
