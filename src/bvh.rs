use crate::ray::Ray;
use crate::aabb::AABB;
use crate::traceable_object::{HitRecord, TraceableObject, DynTraceable};

use std::sync::Arc;
use std::ops::Deref;
use std::cmp::Ordering;
use rand::Rng;

pub struct BVHNode {
    bounds: AABB,
    left:   Arc<DynTraceable>,
    right:  Arc<DynTraceable>,
}

impl BVHNode {
    pub fn build(objects: &[Arc<DynTraceable>]) -> BVHNode {
        assert!(!objects.is_empty(), "Cannot build BVH without any objects");
        
        let axis = rand::thread_rng().gen_range(0, 3);

        let compare = |a: &DynTraceable, b: &DynTraceable| {
            a.bounding_box().unwrap().min[axis] < b.bounding_box().unwrap().min[axis]
        };
        
        let (left, right) = if objects.len() == 1 {
            let a = objects[0].clone();
            let b = objects[0].clone();

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
            let left  = BVHNode::build(&objects[..mid]);
            let right = BVHNode::build(&objects[mid..]);

            let left:  Arc<DynTraceable> = Arc::new(left);
            let right: Arc<DynTraceable> = Arc::new(right);

            (left, right)
        };

        let bounds = AABB::surrounding_box(
            &left.bounding_box().unwrap(),
            &right.bounding_box().unwrap());

        Self {
            left,
            right,
            bounds,
        }
    }
}

impl TraceableObject for BVHNode {
    fn trace(&self, ray: &Ray, min_t: f32, mut max_t: f32) -> Option<HitRecord> {
        if !self.bounds.hits_ray(ray, min_t, max_t) {
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

pub enum BvhNode {
    Leaf(AABB, Arc<DynTraceable>),
    Split(AABB, Box<BvhNode>, Box<BvhNode>),
}

impl BvhNode {
    pub fn new(objects: &[Arc<DynTraceable>]) -> BvhNode {
        let axis = rand::thread_rng().gen_range(0, 3);

        let compare = |a: &DynTraceable, b: &DynTraceable| {
            a.bounding_box().unwrap().min[axis] < b.bounding_box().unwrap().min[axis]
        };

        match objects.len() {
            0 => panic!("Cannot build BVH without any objects"),
            1 => BvhNode::Leaf(objects[0].bounding_box().unwrap(), objects[0].clone()),
            _ => {
                let mut objects = objects.to_owned();
                
                objects.sort_unstable_by(|a, b| {
                    if compare(a.deref(), b.deref()) { 
                        Ordering::Less 
                    } else { 
                        Ordering::Greater 
                    }
                });

                let mid   = objects.len() / 2;
                let left  = BvhNode::new(&objects[..mid]);
                let right = BvhNode::new(&objects[mid..]);

                let bounds = AABB::surrounding_box(
                    &left.bounding_box(),
                    &right.bounding_box());

                BvhNode::Split(bounds, Box::new(left), Box::new(right))
            },
        }
    }
    
    pub fn bounding_box(&self) -> AABB {
        match self {
            BvhNode::Leaf(bounding_box, ..)  => *bounding_box,
            BvhNode::Split(bounding_box, ..) => *bounding_box,
        }
    }

    pub fn trace(&self, ray: &Ray, min_t: f32, mut max_t: f32) -> Option<HitRecord> {
        if !self.bounding_box().hits_ray(ray, min_t, max_t) {
            return None;
        }

        match self {
            BvhNode::Leaf(bounding_box, traceable)    => traceable.trace(ray, min_t, max_t),
            BvhNode::Split(bounding_box, left, right) => {
                match left.trace(ray, min_t, max_t) {
                    Some(record) => match right.trace(ray, min_t, record.t) {
                        Some(record) => Some(record),
                        None         => Some(record),
                    },
                    None => right.trace(ray, min_t, max_t),
                }
            },
        }
    }
}
