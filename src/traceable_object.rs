use crate::vec::Vec3;
use crate::ray::Ray;
use crate::aabb::AABB;
use crate::material::Material;

pub type DynTraceable = dyn TraceableObject + Send + Sync;

pub struct HitRecord<'a> {
    pub t:        f32,
    pub point:    Vec3,
    pub normal:   Vec3,
    pub material: &'a dyn Material,
}

impl<'a> HitRecord<'a> {
    pub fn new(t: f32, point: Vec3, normal: Vec3, material: &'a dyn Material) -> Self {
        Self {
            t,
            point,
            normal,
            material,
        }
    }
}

pub trait TraceableObject {
    fn trace(&self, ray: &Ray, min_t: f32, max_t: f32) -> Option<HitRecord>; 
    fn bounding_box(&self) -> Option<AABB>;
}
