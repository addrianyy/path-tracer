use super::{Material, SharedMaterial};
use crate::{Vec3, Ray};
use crate::traceable::HitRecord;
use crate::rng::Rng;
use crate::math;

pub struct Metal {
    albedo:   Vec3,
    fuziness: f32,
}

impl Metal {
    pub fn new(albedo: Vec3, fuziness: f32) -> SharedMaterial {
        super::make_shared(Self {
            albedo,
            fuziness,
        })
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, record: &HitRecord, rng: &mut Rng) -> Option<(Vec3, Ray)> {
        let reflected = math::reflect(ray.direction, record.normal);

        if Vec3::dot(reflected, record.normal) > 0.0 {
            let fuzz = math::random_in_unit_sphere(rng) * self.fuziness;

            Some((self.albedo, Ray::new(record.point, reflected + fuzz)))
        } else {
            None
        }
    }
}
