mod sphere;

use crate::{Vec3, Ray};
use crate::math::AABB;
use crate::material::Material;

pub use sphere::Sphere;

pub type DynTraceable = dyn Traceable + Send + Sync;

pub struct HitRecord<'a> {
    pub t:        f32,
    pub u:        f32,
    pub v:        f32,
    pub point:    Vec3,
    pub normal:   Vec3,
    pub material: &'a dyn Material,
}

impl<'a> HitRecord<'a> {
    pub fn new(t: f32, point: Vec3, normal: Vec3, material: &'a dyn Material,
               u: f32, v: f32) -> Self {
        Self {
            t,
            u,
            v,
            point,
            normal,
            material,
        }
    }
}

pub trait Traceable {
    fn trace(&self, ray: &Ray, min_t: f32, max_t: f32) -> Option<HitRecord>; 
    fn bounding_box(&self) -> Option<AABB>;
}
