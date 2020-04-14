use crate::vec::Vec3;
use crate::ray::Ray;
use crate::aabb::AABB;
use crate::traceable_object::{HitRecord, TraceableObject};
use std::sync::Arc;
use std::ops::Deref;
use std::cmp::Ordering;

fn surrounding_box(box0: AABB, box1: AABB) -> AABB {
    let min = Vec3::new(
        box0.min.x.min(box1.min.x),
        box0.min.y.min(box1.min.y),
        box0.min.z.min(box1.min.z));

    let max = Vec3::new(
        box0.max.x.max(box1.max.x),
        box0.max.y.max(box1.max.y),
        box0.max.z.max(box1.max.z));
    
    AABB::new(min, max)
}

type Object = dyn TraceableObject + Send + Sync;

pub struct Node {
    bounds: AABB,
    left:   Arc<Object>,
    right:  Arc<Object>,
}

impl Node {
    pub fn build(objects: &[Arc<Object>]) -> Node {
        assert!(objects.len() > 0);

        let compare = |a: &dyn TraceableObject, b: &dyn TraceableObject| {
            a.bounding_box().unwrap().min.y < b.bounding_box().unwrap().min.y
        };
        
        let (left, right) = if objects.len() == 1 {
            let a = objects[0].clone();
            let b = objects[0].clone();


            println!("One element");
            (a, b)
        } else if objects.len() == 2 {
            let a = objects[0].clone();
            let b = objects[1].clone();
            
            if compare(a.deref(), b.deref()) {
                (a, b)
            } else {
                (b, a)
            }
        } else {
            let mut objects = objects.to_owned();
            
            objects.sort_unstable_by(|a, b| {
                if compare(a.deref(), b.deref()) { 
                    Ordering::Less 
                } else { 
                    Ordering::Greater 
                }
            });

            let mid   = objects.len() / 2;
            let left  = Node::build(&objects[..mid]);
            let right = Node::build(&objects[mid..]);

            let left:  Arc<Object> = Arc::new(left);
            let right: Arc<Object> = Arc::new(right);

            (left, right)
        };

        let bounds = surrounding_box(left.bounding_box().unwrap(),
            right.bounding_box().unwrap());

        Self {
            left,
            right,
            bounds,
        }
    }
}

impl TraceableObject for Node {
    fn trace(&self, ray: &Ray, min_t: f32, mut max_t: f32) -> Option<HitRecord> {
        if !self.bounds.hit(ray, min_t, max_t) {
            return None;
        }

        let left = self.left.trace(ray, min_t, max_t);
        if let Some(left) = left.as_ref() {
            max_t = left.t;
        }

        let right = self.right.trace(ray, min_t, max_t);

        right.or(left)
    }

    fn bounding_box(&self) -> Option<AABB> {
        Some(self.bounds)
    }
}
