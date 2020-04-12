use crate::vec::Vec3;
use crate::ray::Ray;
use crate::traceable_object::HitRecord;

pub trait Material {
    fn scatter(&self, ray: &Ray, record: &HitRecord) -> Option<(Vec3, Ray)>;
}
