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
        let inv_direction = inv_direction.extract_array();
        let origin        = ray.origin.extract_array();
        let min           = self.min.extract_array();
        let max           = self.max.extract_array();

        let calc = |a, min_t, max_t| {
            let inv     = inv_direction[a];
            let t0: f32 = (min[a] - origin[a]) * inv;
            let t1: f32 = (max[a] - origin[a]) * inv;

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
        let bbmin = Vec3::min(box0.min, box1.min);
        let bbmax = Vec3::max(box0.max, box1.max);
        
        AABB::new(bbmin, bbmax)
    }

    pub fn volume(&self) -> f32 {
        let (x, y, z) = self.extent().extract();

        (x * y * z).abs()
    }
}
