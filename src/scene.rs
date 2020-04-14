use crate::ray::Ray;
use crate::traceable_object::{HitRecord, TraceableObject};
use crate::bvh;
use std::sync::Arc;

type Object = dyn TraceableObject + Send + Sync;

pub struct Scene {
    objects:  Vec<Box<dyn TraceableObject + Send + Sync>>,
    bvh_root: Option<bvh::Node>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects:  Vec::new(),
            bvh_root: None,
        }
    }

    pub fn trace(&self, ray: &Ray) -> Option<HitRecord> {
        /*
        let mut closest_distance = std::f32::MAX;
        let mut closest_record   = None;

        for object in &self.objects {
            if let Some(record) = object.trace(ray, 0.001, closest_distance) {
                closest_distance = record.t;
                closest_record   = Some(record);
            }
        }
                
        closest_record
        */

        self.bvh_root.as_ref().unwrap().trace(ray, 0.001, std::f32::MAX)
    }

    pub fn create_object<T: 'static + TraceableObject + Send + Sync>(&mut self, object: T) {
        self.objects.push(Box::new(object));
    }

    pub fn build_uvh(&mut self) {
        let objects = self.objects.drain(..);
        let mut new_objects: Vec<Arc<Object>> = Vec::with_capacity(objects.len());

        for obj in objects.into_iter() {
            new_objects.push(obj.into());
        }

        println!("Building...");
        self.bvh_root = Some(bvh::Node::build(&new_objects));
        println!("Built...");
    }
}

