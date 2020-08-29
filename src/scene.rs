use crate::ray::Ray;
use crate::traceable_object::{HitRecord, TraceableObject, DynTraceable};
use crate::bvh::BvhNode;

use std::time::Instant;
use std::io::Write;

pub struct Scene {
    objects:  Vec<Box<DynTraceable>>,
    bvh_root: Option<BvhNode>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects:  Vec::new(),
            bvh_root: None,
        }
    }

    pub fn trace(&self, ray: &Ray) -> Option<HitRecord> {
        let t_min = 0.001;

        let mut closest_distance = std::f32::MAX;
        let mut closest_record   = None;

        if let Some(bvh_root) = self.bvh_root.as_ref() {
            bvh_root.trace(ray, t_min, closest_distance)
        } else {
            for object in &self.objects {
                if let Some(record) = object.trace(ray, t_min, closest_distance) {
                    closest_distance = record.t;
                    closest_record   = Some(record);
                }
            }
            
            closest_record
        }
    }

    pub fn add(&mut self, object: impl TraceableObject + Send + Sync + 'static) {
        self.objects.push(Box::new(object));
    }

    pub fn construct_bvh(&mut self) {
        print!("Constructing BVH for {} objects...", self.objects.len());

        std::io::stdout().flush().unwrap();

        let start_time = Instant::now();

        self.bvh_root = Some(BvhNode::new(std::mem::take(&mut self.objects)));

        println!("Done in {:.3}s.", start_time.elapsed().as_secs_f64());
    }
}
