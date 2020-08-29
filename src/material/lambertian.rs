use super::{Material, SharedMaterial};
use crate::{Vec3, Ray};
use crate::traceable::HitRecord;
use crate::math;

pub struct Lambertian {
    albedo: Vec3,
}

impl Lambertian {
    pub fn new(albedo: Vec3) -> SharedMaterial {
        super::make_shared(Self {
            albedo,
        })
    }
}

impl Material for Lambertian {
    fn scatter(&self, _ray: &Ray, record: &HitRecord) -> Option<(Vec3, Ray)> {
        let target = record.point + record.normal + math::random_in_unit_sphere();

        Some((self.albedo, Ray::new(record.point, target - record.point)))
    }
}
