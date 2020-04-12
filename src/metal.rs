use crate::vec::Vec3;
use crate::ray::Ray;
use crate::material::Material;
use crate::traceable_object::HitRecord;
use crate::math;

pub struct Metal {
    albedo:   Vec3,
    fuziness: f32,
}

impl Metal {
    pub fn new(albedo: Vec3, fuziness: f32) -> Self {
        Self {
            albedo,
            fuziness,
        }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, record: &HitRecord) -> Option<(Vec3, Ray)> {
        let reflected = math::reflect(ray.get_direction(), record.normal);
        if Vec3::dot(reflected, record.normal) > 0.0 {
            let fuzz = math::random_in_unit_sphere() * self.fuziness;
            Some((self.albedo, Ray::new(record.point, reflected + fuzz)))
        } else {
            None
        }
    }
}
