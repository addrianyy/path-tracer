use crate::{Vec3, Ray};
use crate::traceable::{HitRecord, Traceable, DynTraceable};
use crate::bvh::BvhNode;

use std::time::Instant;
use std::io::{self, Write};

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
        const T_MIN: f32 = 0.001;

        let mut closest_distance = std::f32::MAX;
        let mut closest_record   = None;

        if let Some(bvh_root) = self.bvh_root.as_ref() {
            let inv_direction = Vec3::fill(1.0) / ray.direction;

            bvh_root.trace(ray, inv_direction, T_MIN, closest_distance)
        } else {
            for object in &self.objects {
                if let Some(record) = object.trace(ray, T_MIN, closest_distance) {
                    closest_distance = record.t;
                    closest_record   = Some(record);
                }
            }
            
            closest_record
        }
    }

    pub fn add(&mut self, object: impl Traceable + Send + Sync + 'static) {
        self.objects.push(Box::new(object));
    }

    pub fn construct_bvh(&mut self) {
        print!("Constructing BVH for {} objects... ", self.objects.len());

        io::stdout().flush().unwrap();

        let start_time = Instant::now();

        self.bvh_root = Some(BvhNode::new(std::mem::take(&mut self.objects)));

        println!("done in {:.3}s.", start_time.elapsed().as_secs_f64());
    }
}
