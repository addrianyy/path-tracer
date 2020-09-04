use super::{Material, SharedMaterial};
use crate::{Vec3, Ray};
use crate::traceable::HitRecord;
use crate::rng::Rng;
use crate::math;

pub struct Dielectric {
    ref_idx: f32
}

impl Dielectric {
    pub fn new(ref_idx: f32) -> SharedMaterial {
        super::make_shared(Self {
            ref_idx,
        })
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, record: &HitRecord, rng: &mut Rng) -> Option<(Vec3, Ray)> {
        let dir = ray.direction;
        let dot = Vec3::dot(dir, record.normal);

        let (outward_normal, ni_over_nt, cosine) = if dot > 0.0 {
            let cosine = self.ref_idx * dot;

            (-record.normal, self.ref_idx, cosine)
        } else {
            let cosine = -dot;

            (record.normal, 1.0 / self.ref_idx, cosine)
        };
        
        let reflected = math::reflect(dir, record.normal);

        if let Some(refracted) = math::refract(dir, outward_normal, ni_over_nt) {
            let reflect_prob = math::schlick(cosine, self.ref_idx);
            let rand: f32    = rng.rand();

            let new_dir = if rand < reflect_prob {
                reflected
            } else {
                refracted
            };

            Some((Vec3::fill(1.0), Ray::new(record.point, new_dir)))
        } else {
            Some((Vec3::fill(1.0), Ray::new(record.point, reflected)))
        }
    }
}
