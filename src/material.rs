use crate::vec::Vec3;
use crate::ray::Ray;
use crate::traceable_object::HitRecord;

use std::sync::Arc;

pub type DynMaterial    = dyn Material + Send + Sync;
pub type SharedMaterial = Arc<DynMaterial>;

pub trait Material {
    fn create(self) -> SharedMaterial 
        where Self: Sized + Send + Sync + 'static 
    {
        Arc::new(self)
    }
    
    fn scatter(&self, ray: &Ray, record: &HitRecord) -> Option<(Vec3, Ray)>;
}
