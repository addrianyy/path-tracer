mod lambertian;
mod dielectric;
mod metal;

use std::sync::Arc;

use crate::{Vec3, Ray};
use crate::traceable::HitRecord;
use crate::rng::Rng;

pub use lambertian::Lambertian;
pub use dielectric::Dielectric;
pub use metal::Metal;

pub type SharedMaterial = Arc<dyn Material + Send + Sync>;

pub trait Material {
    fn scatter(&self, ray: &Ray, record: &HitRecord, rng: &mut Rng) -> Option<(Vec3, Ray)>;
}

fn make_shared(material: impl Material + Send + Sync + 'static) -> SharedMaterial {
    Arc::new(material)
}
