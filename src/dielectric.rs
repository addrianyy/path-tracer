use crate::vec::Vec3;
use crate::ray::Ray;
use crate::material::Material;
use crate::traceable_object::HitRecord;
use crate::math;

use rand::Rng;

pub struct Dielectric {
    ref_idx: f32
}

impl Dielectric {
    pub fn new(ref_idx: f32) -> Self {
        Self {
            ref_idx
        }
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, record: &HitRecord) -> Option<(Vec3, Ray)> {
        let dir = ray.get_direction();

        let (outward_normal, ni_over_nt, cosine) = if Vec3::dot(dir, record.normal) > 0.0 {
            let cosine = self.ref_idx * Vec3::dot(dir, record.normal);
            (-record.normal, self.ref_idx, cosine)
        } else {
            let cosine = -Vec3::dot(dir, record.normal);
            (record.normal, 1.0 / self.ref_idx, cosine)
        };
        
        let reflected = math::reflect(dir, record.normal);

        if let Some(refracted) = math::refract(dir, outward_normal, ni_over_nt) {
            let reflect_prob = math::schlick(cosine, self.ref_idx);
            let rand: f32 = rand::thread_rng().gen();

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
