mod vec;
mod ray;
mod aabb;

pub use vec::Vec3;
pub use ray::Ray;
pub use aabb::AABB;

use rand::Rng;

pub fn random_in_unit_sphere() -> Vec3 {
    let mut rng = rand::thread_rng();

    loop {
        let vec = (Vec3::new(rng.gen(), rng.gen(), rng.gen()) * 2.0) - Vec3::fill(1.0);

        if vec.length_sqr() < 1.0 {
            break vec
        }
    }
}

pub fn reflect(direction: Vec3, normal: Vec3) -> Vec3 {
    direction - (normal * Vec3::dot(direction, normal) * 2.0)
}

pub fn refract(direction: Vec3, normal: Vec3, ni_over_nt: f32) -> Option<Vec3> {
    let dt = Vec3::dot(direction, normal);
    let d  = 1.0 - ni_over_nt * ni_over_nt * (1.0 - dt * dt);

    if d > 0.0 {
        Some((direction - normal * dt) * ni_over_nt - normal * d.sqrt())
    } else {
        None
    }
}

pub fn schlick(cosine: f32, ref_idx: f32) -> f32 {
    let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    let r0 = r0 * r0;

    r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
}
