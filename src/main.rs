mod vec;
mod ray;
mod traceable_object;
mod scene;
mod sphere;
mod camera;
mod material;
mod lambertian;
mod metal;
mod math;
mod dielectric;
mod aabb;
mod bvh;

use vec::Vec3;
use ray::Ray;
use scene::Scene;
use sphere::Sphere;
use camera::Camera;
use metal::Metal;
use lambertian::Lambertian;
use dielectric::Dielectric;

use std::sync::Arc;
use std::time::Instant;

use image::ImageBuffer;
use rand::Rng;

use material::Material;

fn trace_ray(ray: &Ray, scene: &Scene) -> Vec3 {
    let mut current_attenuation = Vec3::fill(1.0);
    let mut current_ray = *ray;

    const RECURSION_LIMIT: usize = 5;
    for _ in 0..RECURSION_LIMIT {
        if let Some(record) = scene.trace(&current_ray) {
            if let Some((attenuation, new_ray)) = record.material.scatter(&current_ray, &record) {
                current_ray = new_ray;
                current_attenuation *= attenuation;
            } else {
                return Vec3::fill(0.0);
            }
        } else {
            break;
        }
    }

    let t = 0.5 * (ray.get_direction().y + 1.0);
    let color = Vec3::new(1.0, 1.0, 1.0) * (1.0 - t) + Vec3::new(0.5, 0.7, 1.0) * t;

    color * current_attenuation
}


fn load_scene(scene: &mut Scene) {
    let matte1 = Lambertian::new(Vec3::new(0.0, 0.2, 0.5)).create();
    let matte2 = Lambertian::new(Vec3::new(0.3, 0.0, 0.0)).create();
    scene.create_object(Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5, &matte1));
    scene.create_object(Sphere::new(Vec3::new(0.0, -100.5, -1.0), 100.0, &matte2));

    let metal1 = Metal::new(Vec3::new(0.8, 0.6, 0.2), 0.0).create();
    let glass1 = Dielectric::new(1.8).create();
    let glass2 = Dielectric::new(0.4).create();
    scene.create_object(Sphere::new(Vec3::new( 1.5, 0.0, -2.0), 0.5, &metal1));
    scene.create_object(Sphere::new(Vec3::new(-1.5, 0.0, -2.0), 0.5, &glass1));
    scene.create_object(Sphere::new(Vec3::new( 3.5, 0.0, -2.0), 0.8, &glass2));

    let metal2 = Metal::new(Vec3::new(0.1, 1.0, 0.7), 0.1).create();
    scene.create_object(Sphere::new(Vec3::new(10.0, 0.0, -10.0), 3.0, &metal2));
}

fn random_scene(scene: &mut Scene) {
    scene.create_object(Sphere::new(Vec3::new(0.0, -1000.0, 0.0), 1000.0, 
        &Lambertian::new(Vec3::new(0.5, 0.5, 0.5)).create()));

    let mut rng = rand::thread_rng();

    let random_vec = |min: f32, max: f32| {
        let mut rng = rand::thread_rng();
        Vec3::new(
            rng.gen_range(min, max),
            rng.gen_range(min, max),
            rng.gen_range(min, max))
    };

    for a in -11..11 {
        for b in -11..11 {
            let center = Vec3::new(
                a as f32 + 0.9 * rng.gen::<f32>(), 
                0.2,
                b as f32 + 0.9 * rng.gen::<f32>()
            );

            if (center - Vec3::new(3.0, 0.2, 0.0)).length() > 0.9 {
                let choose_mat: f32 = rng.gen();

                let material = if choose_mat < 0.8 {
                    let albedo = random_vec(0.0, 1.0) * random_vec(0.0, 1.0);
                    Lambertian::new(albedo).create()
                } else if choose_mat < 0.95 {
                    let albedo = random_vec(0.5, 1.0);
                    let fuzz   = rng.gen_range(0.0, 0.5);
                    Metal::new(albedo, fuzz).create()
                } else {
                    Dielectric::new(1.5).create()
                };

                scene.create_object(Sphere::new(center, 0.2, &material));
            }
        }
    }

    scene.create_object(Sphere::new(Vec3::new(0.0, 1.0, 0.0), 1.0, 
        &Dielectric::new(1.5).create()));

    scene.create_object(Sphere::new(Vec3::new(-4.0, 1.0, 0.0), 1.0, 
        &Lambertian::new(Vec3::new(0.4, 0.2, 0.1)).create()));

    scene.create_object(Sphere::new(Vec3::new(4.0, 1.0, 0.0), 1.0, 
        &Metal::new(Vec3::new(0.7, 0.6, 0.5), 0.0).create()));
}

fn main() {
    let width = 1920;
    let height = 1080;
    let num_samples_per_axis = 5;

    let camera = Camera::new(
        Vec3::new(12.0, 2.0, 3.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        20.0,
        width as f32 / height as f32
    );
    
    let mut scene = Scene::new();

    load_scene(&mut scene);
    scene.construct_bvh();

    let scene = Arc::new(scene);

    let thread_count      = num_cpus::get() * 8;
    let pixels_per_thread = (width * height + thread_count - 1) / thread_count;

    let mut threads = Vec::with_capacity(thread_count);
    
    println!("Using {} threads.", thread_count);

    let start_time = Instant::now();

    for thread_index in 0..thread_count {
        let scene       = scene.clone();
        let camera      = camera.clone();
        let start_pixel = pixels_per_thread * thread_index;
        
        let pixels_outside_screen = if thread_index + 1 == thread_count {
            (pixels_per_thread * thread_count).checked_sub(width * height).unwrap()
        } else {
            0
        };

        let pixels_in_this_thread = pixels_per_thread.checked_sub(pixels_outside_screen).unwrap();

        threads.push(std::thread::spawn(move || {
            let mut pixels = Vec::with_capacity(pixels_in_this_thread);

            for i in 0..pixels_in_this_thread {
                let x = (i + start_pixel) % width;
                let y = (i + start_pixel) / width;

                let mut color_sum = Vec3::zero();

                for sx in 0..num_samples_per_axis {
                    for sy in 0..num_samples_per_axis {
                        let x = x as f32 + (sx as f32 / (num_samples_per_axis - 1) as f32); 
                        let y = y as f32 + (sy as f32 / (num_samples_per_axis - 1) as f32);

                        let u = x / width as f32;
                        let v = 1.0 - (y / height as f32);

                        let ray   = camera.get_ray(u, v);
                        let color = trace_ray(&ray, &scene);

                        color_sum += color;
                    }
                }

                let num_samples = num_samples_per_axis * num_samples_per_axis;
                let color = color_sum / num_samples as f32;
                let color = Vec3::new(color.x.sqrt(), color.y.sqrt(), color.z.sqrt());

                pixels.push(color);
            }

            pixels
        }));
    }

    let pixels: Vec<_> = threads.into_iter().map(|x| x.join().unwrap()).collect();
    let pixel_count    = pixels.iter().fold(0, |acc, x| acc + x.len());

    assert_eq!(pixel_count, width * height, "Unexpected number of generated pixels.");

    let execution_time = Instant::now().duration_since(start_time);
    println!("Raytracing took {:.8} seconds.", execution_time.as_secs_f64());

    let mut imgbuf = ImageBuffer::new(width as u32, height as u32);

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let x = x as usize;
        let y = y as usize;

        let pixel_index  = x + y * width;
        let thread_index = pixel_index / pixels_per_thread;
        let index        = pixel_index % pixels_per_thread;

        let color = pixels[thread_index][index];

        let r = (color.x * 255.0) as u8;
        let g = (color.y * 255.0) as u8;
        let b = (color.z * 255.0) as u8;

        *pixel = image::Rgb([r, g, b]);
    }

    imgbuf.save("output.png").expect("Failed to save output image.");
}
