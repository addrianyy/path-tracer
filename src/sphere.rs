use crate::vec::Vec3;
use crate::ray::Ray;
use crate::material_manager::MaterialHandle;
use crate::traceable_object::{HitRecord, TraceableObject};

pub struct Sphere {
    center:   Vec3,
    radius:   f32,
    material: MaterialHandle, 
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: MaterialHandle) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl TraceableObject for Sphere {
    fn trace(&self, ray: &Ray, min_t: f32, max_t: f32) -> Option<HitRecord> {
        let oc = ray.get_origin() - self.center;
        let a  = Vec3::dot(ray.get_direction(), ray.get_direction());
        let b  = Vec3::dot(oc, ray.get_direction());
        let c  = Vec3::dot(oc, oc) - self.radius * self.radius;
        let d  = b * b - a * c;

        if d > 0.0 {
            let sol = (-b - d.sqrt()) / a;
            if sol < max_t && sol > min_t {
                let point = ray.get_point(sol);
                return Some(HitRecord::new(sol, point,
                    (point - self.center).normalized(), self.material));
            }

            let sol = (-b + d.sqrt()) / a;
            if sol < max_t && sol > min_t {
                let point = ray.get_point(sol);
                return Some(HitRecord::new(sol, point, 
                    (point - self.center).normalized(), self.material));
            }
        }

        None
    }
}
