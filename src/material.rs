use crate::vec::Vec3;
use crate::ray::Ray;
use crate::traceable_object::HitRecord;

use std::sync::Arc;

pub trait Material {
    fn scatter(&self, ray: &Ray, record: &HitRecord) -> Option<(Vec3, Ray)>;
}

pub type SharedMaterial = Arc<dyn Material + 'static + Send + Sync>;

pub fn create(material: impl Material + 'static + Send + Sync) -> SharedMaterial {
    Arc::new(material)
}
