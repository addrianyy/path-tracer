use crate::Vec3;
use crate::rng::Rng;
use crate::traceable::Sphere;
use crate::texture::PictureTexture;
use crate::material::{Metal, Lambertian, Dielectric};
use super::Scene;

pub fn simple_scene(scene: &mut Scene) {
    let matte1 = Lambertian::new(PictureTexture::new("earthmap.jpg"));
    let matte2 = Lambertian::new_solid(Vec3::new(0.3, 0.0, 0.0));
    scene.add(Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5, &matte1));
    scene.add(Sphere::new(Vec3::new(0.0, -100.5, -1.0), 100.0, &matte2));

    let metal1 = Metal::new(Vec3::new(0.8, 0.6, 0.2), 0.0);
    let glass1 = Dielectric::new(1.8);
    let glass2 = Dielectric::new(0.4);
    scene.add(Sphere::new(Vec3::new( 1.5, 0.0, -2.0), 0.5, &metal1));
    scene.add(Sphere::new(Vec3::new(-1.5, 0.0, -2.0), 0.5, &glass1));
    scene.add(Sphere::new(Vec3::new( 3.5, 0.0, -2.0), 0.8, &glass2));

    let metal2 = Metal::new(Vec3::new(0.1, 1.0, 0.7), 0.1);
    scene.add(Sphere::new(Vec3::new(10.0, 0.0, -10.0), 3.0, &metal2));
}

pub fn random_scene(scene: &mut Scene) {
    scene.add(Sphere::new(
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        &Lambertian::new_solid(Vec3::new(0.5, 0.5, 0.5)),
    ));

    let mut rng = Rng::new();

    let random_vec = |rng: &mut Rng, min: f32, max: f32| {
        Vec3::new(
            rng.rand_range(min, max),
            rng.rand_range(min, max),
            rng.rand_range(min, max),
        )
    };

    for a in -22..22 {
        for b in -22..221 {
            let center = Vec3::new(
                a as f32 + 0.9 * rng.rand::<f32>(),
                0.2,
                b as f32 + 0.9 * rng.rand::<f32>(),
            );

            if (center - Vec3::new(3.0, 0.2, 0.0)).length() > 0.9 {
                let choose_mat: f32 = rng.rand();

                let material = if choose_mat < 0.8 {
                    let albedo = random_vec(&mut rng, 0.0, 1.0) * 
                        random_vec(&mut rng, 0.0, 1.0);

                    Lambertian::new_solid(albedo)
                } else if choose_mat < 0.95 {
                    let albedo = random_vec(&mut rng, 0.5, 1.0);
                    let fuzz   = rng.rand_range(0.0, 0.5);

                    Metal::new(albedo, fuzz)
                } else {
                    Dielectric::new(1.5)
                };

                scene.add(Sphere::new(center, 0.2, &material));
            }
        }
    }

    scene.add(Sphere::new(
        Vec3::new(0.0, 1.0, 0.0),
        1.0,
        &Dielectric::new(1.5),
    ));

    scene.add(Sphere::new(
        Vec3::new(-4.0, 1.0, 0.0),
        1.0, 
        &Lambertian::new_solid(Vec3::new(0.4, 0.2, 0.1)),
    ));

    scene.add(Sphere::new(
        Vec3::new(4.0, 1.0, 0.0),
        1.0,
        &Metal::new(Vec3::new(0.7, 0.6, 0.5), 0.0),
    ));
}
