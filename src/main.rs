#![allow(dead_code)]

mod traceable;
mod material;
mod texture;
mod math;

mod rng;
mod bvh;
mod scene;
mod camera;
mod parallel_renderer;

pub use math::{Vec3, Ray};

use rng::Rng;
use scene::Scene;
use camera::Camera;
use traceable::Sphere;
use material::{Metal, Lambertian, Dielectric};
use texture::PictureTexture;
use parallel_renderer::ParallelRenderer;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use std::io::{self, Write};
use std::sync::Arc;
use std::thread;

use image::RgbImage;

struct State {
    scene:       Scene,
    camera:      Camera,
    pixels_done: AtomicUsize,
}

impl State {
    fn new(scene: Scene, camera: Camera) -> Self {
        Self {
            scene,
            camera,
            pixels_done: AtomicUsize::new(0),
        }
    }
}

fn load_scene(scene: &mut Scene) {
    //let matte1 = Lambertian::new_solid(Vec3::new(0.0, 0.2, 0.5));
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

fn random_scene(scene: &mut Scene) {
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

fn trace_ray(ray: &Ray, scene: &Scene, rng: &mut Rng) -> Vec3 {
    let mut current_attenuation = Vec3::fill(1.0);
    let mut current_ray         = *ray;

    const MAX_TRACES: usize = 5;

    for _ in 0..MAX_TRACES {
        if let Some(record) = scene.trace(&current_ray) {
            let scattered = record.material.scatter(&current_ray, &record, rng);

            if let Some((attenuation, new_ray)) = scattered {
                current_attenuation *= attenuation;
                current_ray          = new_ray;
            } else {
                return Vec3::fill(0.0);
            }
        } else {
            break;
        }
    }

    let t     = 0.5 * (ray.direction.extract().1 + 1.0);
    let color = Vec3::new(1.0, 1.0, 1.0) * (1.0 - t) + Vec3::new(0.5, 0.7, 1.0) * t;

    color * current_attenuation
}

fn main() {
    let width  = 3840 - 200;
    let height = 2160 - 200;
    let samples_per_axis = 16;

    let camera = Camera::new(
        Vec3::new(12.0, 2.0, 3.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        20.0,
        width as f32 / height as f32,
    );

    let mut scene = Scene::new();

    load_scene(&mut scene);
    //random_scene(&mut scene);

    scene.construct_bvh();

    let state = Arc::new(State::new(scene, camera));

    let mut renderer = ParallelRenderer::new();
    let mut buffer   = vec![[0u8; 3]; width * height];

    let start_time  = Instant::now();

    let reporter = {
        let state = state.clone();

        thread::spawn(move || {
            let pixel_count = width * height;

            loop {
                let pixels_done = state.pixels_done.load(Ordering::Relaxed);
                let progress    = pixels_done as f64 / pixel_count as f64;

                let elapsed = start_time.elapsed().as_secs_f64();

                let max_bars = 50;
                let bars     = (progress * max_bars as f64) as u64;

                print!("\r  [");

                for i in 0..max_bars {
                    if i < bars {
                        print!("=");
                    } else {
                        print!("-");
                    }
                }

                print!("] {:.1}% | {:.3}s elapsed", progress * 100.0, elapsed);

                if pixels_done == pixel_count {
                    println!();
                    break;
                }

                io::stdout().flush().unwrap();
                thread::sleep(Duration::from_millis(100));
            }
        })
    };

    renderer.render(&state, &mut buffer, move |state, rng, start_pixel, pixels| {
        const PROGRESS_STEP: usize = 8192;

        let pixel_count = pixels.len();

        for (i, pixel) in pixels.iter_mut().enumerate() {
            let x = (i + start_pixel) % width;
            let y = (i + start_pixel) / width;

            let mut color_sum = Vec3::zero();

            for sx in 0..samples_per_axis {
                let x = x as f32 + (sx as f32 / (samples_per_axis - 1) as f32);
                let u = x / width as f32;

                for sy in 0..samples_per_axis {
                    let y = y as f32 + (sy as f32 / (samples_per_axis - 1) as f32);
                    let v = 1.0 - (y / height as f32);

                    let ray   = state.camera.ray(u, v);
                    let color = trace_ray(&ray, &state.scene, rng);

                    color_sum += color;
                }
            }

            if i > 0 {
                if i % PROGRESS_STEP == 0 {
                    state.pixels_done.fetch_add(PROGRESS_STEP, Ordering::Relaxed);
                } else if i == pixel_count - 1 {
                    state.pixels_done.fetch_add(pixel_count % PROGRESS_STEP,
                                                Ordering::Relaxed);
                }
            }

            let samples   = samples_per_axis * samples_per_axis;
            let color     = (color_sum / samples as f32).sqrt() * Vec3::fill(255.0);
            let (r, g, b) = color.extract();

            *pixel = [r as u8, g as u8, b as u8];
        }
    });

    reporter.join().unwrap();

    let mut image = Vec::with_capacity(width * height);

    for &[r, g, b] in &buffer {
        image.push(r);
        image.push(g);
        image.push(b);
    }

    RgbImage::from_raw(width as u32, height as u32, image)
        .expect("Failed to create image buffer for the PNG.")
        .save("output.png")
        .expect("Failed to save output image.");
}
