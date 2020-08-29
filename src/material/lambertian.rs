use super::{Material, SharedMaterial};
use crate::{Vec3, Ray};
use crate::traceable::HitRecord;
use crate::texture::{SharedTexture, SolidTexture};
use crate::math;

pub struct Lambertian {
    albedo: SharedTexture,
}

impl Lambertian {
    pub fn new(albedo: SharedTexture) -> SharedMaterial {
        super::make_shared(Self {
            albedo,
        })
    }

    pub fn new_solid(albedo: Vec3) -> SharedMaterial {
        super::make_shared(Self {
            albedo: SolidTexture::new(albedo),
        })
    }
}

impl Material for Lambertian {
    fn scatter(&self, _ray: &Ray, record: &HitRecord) -> Option<(Vec3, Ray)> {
        let target = record.point + record.normal + math::random_in_unit_sphere();
        let color  = self.albedo.color(record.u, record.v, record.point);

        Some((color, Ray::new(record.point, target - record.point)))
    }
}
