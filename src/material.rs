use crate::vec::Vec3;
use crate::ray::Ray;
use crate::traceable_object::HitRecord;

use std::sync::Arc;

pub trait Material {
    fn build(self) -> Arc<dyn Material + Send + Sync> 
        where Self: Sized + Send + Sync + 'static 
    {
        Arc::new(self)
    }
    
    fn scatter(&self, ray: &Ray, record: &HitRecord) -> Option<(Vec3, Ray)>;
}

pub type SharedMaterial = Arc<dyn Material + Send + Sync>;

pub fn create(material: impl Material + 'static + Send + Sync) -> SharedMaterial {
    Arc::new(material)
}
