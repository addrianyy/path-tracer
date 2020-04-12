use crate::vec::Vec3;
use crate::ray::Ray;
use crate::material::Material;
use crate::material_manager::{MaterialManager, MaterialHandle};

pub struct HitRecord {
    pub t:        f32,
    pub point:    Vec3,
    pub normal:   Vec3,
    pub material: MaterialHandle,
}

impl HitRecord {
    pub fn new(t: f32, point: Vec3, normal: Vec3, material: MaterialHandle) -> Self {
        Self {
            t,
            point,
            normal,
            material,
        }
    }

    pub fn get_material<'a>(&self, material_manager: &'a MaterialManager) -> &'a dyn Material {
        material_manager.get_material(self.material)
    }
}

pub trait TraceableObject {
    fn trace(&self, ray: &Ray, min_t: f32, max_t: f32) -> Option<HitRecord>; 
}
