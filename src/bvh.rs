use crate::vec::Vec3;
use crate::ray::Ray;
use crate::aabb::AABB;
use crate::traceable_object::{HitRecord, DynTraceable};

use std::cmp::Ordering;
use rand::Rng;

pub enum BvhNode {
    Leaf(AABB, Box<DynTraceable>),
    Split(AABB, Box<BvhNode>, Box<BvhNode>),
}

impl BvhNode {
    pub fn new(objects: &mut [Option<Box<DynTraceable>>]) -> BvhNode {
        let axis = rand::thread_rng().gen_range(0, 3);

        match objects.len() {
            0 => panic!("Cannot build BVH without any objects"),
            1 => {
                let object = objects[0].take().unwrap();
                BvhNode::Leaf(object.bounding_box().unwrap(), object)
            },
            _ => {
                objects.sort_unstable_by(|a, b| {
                    let bbox_a = a.as_ref().unwrap().bounding_box().unwrap();
                    let bbox_b = b.as_ref().unwrap().bounding_box().unwrap();

                    bbox_a.center()[axis].partial_cmp(&bbox_b.center()[axis])
                        .unwrap_or(Ordering::Equal)
                });

                let (left, right) = objects.split_at_mut(objects.len() / 2);

                let left   = BvhNode::new(left);
                let right  = BvhNode::new(right);
                let bounds = AABB::enclosing_box(&left.bounding_box(), &right.bounding_box());

                BvhNode::Split(bounds, Box::new(left), Box::new(right))
            },
        }
    }

    pub fn new_better(objects: &mut [Option<Box<DynTraceable>>]) -> BvhNode {
        macro_rules! get_bbox {
            ($object:expr) => { $object.as_ref().unwrap().bounding_box().unwrap() }
        }

        match objects.len() {
            0 => panic!("Cannot build BVH without any objects"),
            1 => {
                let object = objects[0].take().unwrap();
                BvhNode::Leaf(object.bounding_box().unwrap(), object)
            },
            _ => {
                let whole_bbox = objects.iter().skip(1).fold(get_bbox!(objects[0]),
                    |bbox, object| AABB::enclosing_box(&bbox, &get_bbox!(object))); 
                let extent = whole_bbox.max - whole_bbox.min;

                let sort_axis = {
                    let mut longest_axis = 0;

                    for i in 1..3 {
                        if extent[i] > extent[longest_axis] {
                            longest_axis = i;
                        }
                    }

                    longest_axis
                };

                objects.sort_unstable_by(|a, b| {
                    let bbox_a = get_bbox!(a);
                    let bbox_b = get_bbox!(b);

                    bbox_a.center()[sort_axis].partial_cmp(&bbox_b.center()[sort_axis])
                        .unwrap_or(Ordering::Equal)
                });

                let (left, right) = objects.split_at_mut(objects.len() / 2);

                let left   = BvhNode::new_better(left);
                let right  = BvhNode::new_better(right);
                let bounds = AABB::enclosing_box(&left.bounding_box(), &right.bounding_box());

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

    pub fn trace(&self, ray: &Ray, min_t: f32, max_t: f32) -> Option<HitRecord> {
        match self {
            BvhNode::Leaf(_, traceable)    => traceable.trace(ray, min_t, max_t),
            BvhNode::Split(_, left, right) => {
                if self.bounding_box().intersect(ray, min_t, max_t) {
                    match left.trace(ray, min_t, max_t) {
                        Some(record) => match right.trace(ray, min_t, record.t) {
                            Some(record) => Some(record),
                            None         => Some(record),
                        },
                        None => right.trace(ray, min_t, max_t),
                    }
                } else {
                    None
                }
            },
        }
    }
}
