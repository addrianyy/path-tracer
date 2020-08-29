use super::{HitRecord, Traceable};
use crate::{Vec3, Ray};
use crate::math::AABB;
use crate::material::SharedMaterial;

pub struct Sphere {
    center:   Vec3,
    radius:   f32,
    material: SharedMaterial, 
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: &SharedMaterial) -> Self {
        Self {
            center,
            radius,
            material: material.clone(),
        }
    }
}

impl Sphere {
    fn record(&self, t: f32, ray: &Ray) -> HitRecord {
        let point = ray.point(t);

        HitRecord::new(t, point, (point - self.center).normalized(), &*self.material)
    }
}

impl Traceable for Sphere {
    fn trace(&self, ray: &Ray, min_t: f32, max_t: f32) -> Option<HitRecord> {
        let oc = ray.origin - self.center;

        let a = Vec3::dot(ray.direction, ray.direction);
        let b = Vec3::dot(oc, ray.direction);
        let c = Vec3::dot(oc, oc) - self.radius * self.radius;
        let d = b * b - a * c;

        if d > 0.0 {
            let sol = (-b - d.sqrt()) / a;

            if sol < max_t && sol > min_t {
                return Some(self.record(sol, ray));
            }

            let sol = (-b + d.sqrt()) / a;

            if sol < max_t && sol > min_t {
                return Some(self.record(sol, ray));
            }
        }

        None
    }

    fn bounding_box(&self) -> Option<AABB> {
        Some(AABB::new(
            self.center - Vec3::fill(self.radius),
            self.center + Vec3::fill(self.radius),
        ))
    }
}
