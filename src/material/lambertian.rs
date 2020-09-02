use super::{Material, SharedMaterial};
use crate::{Vec3, Ray};
use crate::traceable::HitRecord;
use crate::texture::SharedTexture;
use crate::rng::Rng;
use crate::math;

enum Albedo {
    Solid(Vec3),
    Texture(SharedTexture),
}

pub struct Lambertian {
    albedo: Albedo,
}

impl Lambertian {
    pub fn new(albedo: SharedTexture) -> SharedMaterial {
        super::make_shared(Self {
            albedo: Albedo::Texture(albedo),
        })
    }

    pub fn new_solid(albedo: Vec3) -> SharedMaterial {
        super::make_shared(Self {
            albedo: Albedo::Solid(albedo),
        })
    }
}

impl Material for Lambertian {
    fn scatter(&self, _ray: &Ray, record: &HitRecord, rng: &mut Rng) -> Option<(Vec3, Ray)> {
        let target = record.point + record.normal + math::random_in_unit_sphere(rng);

        let color = match &self.albedo {
            Albedo::Texture(albedo) => {
                let (u, v) = record.uv();

                albedo.color(u, v, record.point)
            }
            Albedo::Solid(albedo)   => *albedo,
        };

        Some((color, Ray::new(record.point, target - record.point)))
    }
}
