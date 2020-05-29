use crate::ray::Ray;
use crate::aabb::AABB;
use crate::traceable_object::{HitRecord, DynTraceable};

use std::cmp::Ordering;

pub enum BvhNode {
    Leaf(AABB, Box<DynTraceable>),
    Split(AABB, Box<BvhNode>, Box<BvhNode>),
}

impl BvhNode {
    fn construct(objects: &mut [Option<Box<DynTraceable>>]) -> BvhNode {
        macro_rules! get_bbox {
            ($object:expr) => { $object.as_ref().unwrap().bounding_box().unwrap() }
        }

        match objects.len() {
            0 => panic!("Cannot build BVH without any objects."),
            1 => {
                let object = objects[0].take().unwrap();
                BvhNode::Leaf(object.bounding_box().unwrap(), object)
            },
            _ => {
                let get_enclosing_bbox = |objects: &[Option<Box<DynTraceable>>]| {
                    assert!(!objects.is_empty(), "Cannot get bounding box for empty object slice.");

                    objects.iter().skip(1).fold(get_bbox!(objects[0]),
                        |bbox, object| AABB::enclosing_box(&bbox, &get_bbox!(object)))
                };

                let whole_bbox = get_enclosing_bbox(&objects);

                let split_axis = {
                    let extent = whole_bbox.extent(); 

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

                    bbox_a.center()[split_axis].partial_cmp(&bbox_b.center()[split_axis])
                        .unwrap_or(Ordering::Equal)
                });

                let split_index = {
                    let mut lowest_cost = std::f32::MAX;
                    let mut lowest_cost_index = 0;

                    for i in 1..objects.len() {
                        let get_cost = |objects| {
                            let bbox = get_enclosing_bbox(objects);
                            let prob = bbox.volume() / whole_bbox.volume();

                            prob * objects.len() as f32
                        };

                        let (left, right) = objects.split_at_mut(i);
                        let split_cost = get_cost(left) + get_cost(right); 

                        if split_cost < lowest_cost {
                            lowest_cost = split_cost;
                            lowest_cost_index = i;
                        }
                    }

                    lowest_cost_index
                };

                let (left, right) = objects.split_at_mut(split_index);

                let left   = BvhNode::construct(left);
                let right  = BvhNode::construct(right);
                let bounds = AABB::enclosing_box(&left.bounding_box(), &right.bounding_box());

                BvhNode::Split(bounds, Box::new(left), Box::new(right))
            },
        }
    }

    pub fn new(objects: Vec<Box<DynTraceable>>) -> BvhNode {
        let mut objects: Vec<_> = objects.into_iter().map(Some).collect();
        BvhNode::construct(&mut objects)
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
