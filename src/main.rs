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

fn trace_ray(ray: &Ray, scene: &Scene) -> Vec3 {
    let mut current_attenuation = Vec3::fill(1.0);
    let mut current_ray = *ray;

    const RECURSION_LIMIT: usize = 50;
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
    let matte1 = material::create(Lambertian::new(Vec3::new(0.0, 0.2, 0.5)));
    let matte2 = material::create(Lambertian::new(Vec3::new(0.3, 0.0, 0.0)));
    scene.create_object(Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5, &matte1));
    scene.create_object(Sphere::new(Vec3::new(0.0, -100.5, -1.0), 100.0, &matte2));

    let metal1 = material::create(Metal::new(Vec3::new(0.8, 0.6, 0.2), 0.0));
    let glass1 = material::create(Dielectric::new(1.8));
    let glass2 = material::create(Dielectric::new(0.4));
    scene.create_object(Sphere::new(Vec3::new( 1.5, 0.0, -2.0), 0.5, &metal1));
    scene.create_object(Sphere::new(Vec3::new(-1.5, 0.0, -2.0), 0.5, &glass1));
    scene.create_object(Sphere::new(Vec3::new( 3.5, 0.0, -2.0), 0.8, &glass2));

    let metal2 = material::create(Metal::new(Vec3::new(0.1, 1.0, 0.7), 0.1));
    scene.create_object(Sphere::new(Vec3::new(10.0, 0.0, -10.0), 3.0, &metal2));
}

fn main() {
    let width:  usize = 1920;
    let height: usize = 1080;

    let camera = Camera::new(
        Vec3::new(-2.0, 2.0, 1.0),
        Vec3::new(0.0, 0.0, -1.0),
        Vec3::new(0.0, 1.0, 0.0),
        90.0,
        width as f32 / height as f32
    );
    
    let mut scene = Scene::new();
    load_scene(&mut scene);
    let scene = Arc::new(scene);

    let thread_count      = num_cpus::get() * 8;
    let lines_per_thread  = (height + thread_count - 1) / thread_count;
    let pixels_per_thread = lines_per_thread * width;

    let mut threads = Vec::with_capacity(thread_count);
    
    println!("Using {} threads.", thread_count);
    let start_time = Instant::now();

    for i in 0..thread_count {
        let scene   = scene.clone();
        let camera  = camera.clone();
        let start_y = lines_per_thread * i;
        
        let pixels_outside_screen = if i + 1 == thread_count {
            let rem = height % thread_count;
            if  rem == 0 { 0 } else { thread_count - rem }
        } else {
            0
        } * width;

        let pixels_in_this_thread = pixels_per_thread - pixels_outside_screen;

        threads.push(std::thread::spawn(move || {
            let mut pixels = Vec::with_capacity(pixels_in_this_thread);
            let mut rng    = rand::thread_rng();

            for i in 0..pixels_in_this_thread {
                let x = i % width;
                let y = i / width + start_y;

                let num_samples = 50;
                let mut color_sum = Vec3::zero();

                for _ in 0..num_samples {
                    let x = x as f32 + (rng.gen::<f32>() * 2.0 - 1.0);
                    let y = y as f32 + (rng.gen::<f32>() * 2.0 - 1.0);

                    let u = x / width as f32;
                    let v = 1.0 - (y / height as f32);

                    let ray   = camera.get_ray(u, v);
                    let color = trace_ray(&ray, &scene);

                    color_sum += color;
                }

                let color = color_sum / num_samples as f32;
                let color = Vec3::new(color.x.sqrt(), color.y.sqrt(), color.z.sqrt());

                pixels.push(color);
            }

            pixels
        }));
    }

    let pixels: Vec<_> = threads.into_iter().map(|x| x.join().unwrap()).collect();
    let pixel_count    = pixels.iter().fold(0, |acc, x| acc + x.len()); 

    assert_eq!(pixel_count, width * height, "Unexprected number of generated pixels.");

    let execution_time = Instant::now().duration_since(start_time);
    println!("Raytracing took {:.8} seconds.", execution_time.as_secs_f64());

    let mut imgbuf = ImageBuffer::new(width as u32, height as u32);

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let x = x as usize;
        let y = y as usize;

        let thread_index = y / lines_per_thread;
        let index        = x + (y % lines_per_thread) * width;

        let color = pixels[thread_index][index];

        let r = (color.x * 255.0) as u8;
        let g = (color.y * 255.0) as u8;
        let b = (color.z * 255.0) as u8;

        *pixel = image::Rgb([r, g, b]);
    }

    imgbuf.save("output.png").expect("Failed to save output image.");
}
