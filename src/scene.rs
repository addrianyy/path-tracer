use crate::ray::Ray;
use crate::traceable_object::{HitRecord, TraceableObject};

pub struct Scene {
    objects: Vec<Box<dyn TraceableObject + Send + Sync>>
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects:   Vec::new(),
        }
    }

    pub fn trace(&self, ray: &Ray) -> Option<HitRecord> {
        let mut closest_distance = std::f32::MAX;
        let mut closest_record   = None;

        for object in &self.objects {
            if let Some(record) = object.trace(ray, 0.001, closest_distance) {
                closest_distance = record.t;
                closest_record   = Some(record);
            }
        }
                
        closest_record
    }

    pub fn create_object<T: 'static + TraceableObject + Send + Sync>(&mut self, object: T) {
        self.objects.push(Box::new(object));
    }
}
