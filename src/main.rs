mod traceable;
mod material;
mod math;

mod scene;
mod camera;
mod bvh;
mod pin;

pub use math::{Vec3, Ray};

use scene::Scene;
use camera::Camera;
use traceable::Sphere;
use material::{Metal, Lambertian, Dielectric};

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use std::io::{self, Write};
use std::thread;

use image::ImageBuffer;
use rand::Rng;

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

    let t = 0.5 * (ray.direction.y + 1.0);
    let color = Vec3::new(1.0, 1.0, 1.0) * (1.0 - t) + Vec3::new(0.5, 0.7, 1.0) * t;

    color * current_attenuation
}


fn load_scene(scene: &mut Scene) {
    let matte1 = Lambertian::new(Vec3::new(0.0, 0.2, 0.5));
    let matte2 = Lambertian::new(Vec3::new(0.3, 0.0, 0.0));
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
    scene.add(Sphere::new(Vec3::new(0.0, -1000.0, 0.0), 1000.0,
        &Lambertian::new(Vec3::new(0.5, 0.5, 0.5))));

    let mut rng = rand::thread_rng();

    let random_vec = |min: f32, max: f32| {
        let mut rng = rand::thread_rng();
        Vec3::new(
            rng.gen_range(min, max),
            rng.gen_range(min, max),
            rng.gen_range(min, max))
    };

    for a in -22..22 {
        for b in -22..221 {
            let center = Vec3::new(
                a as f32 + 0.9 * rng.gen::<f32>(),
                0.2,
                b as f32 + 0.9 * rng.gen::<f32>()
            );

            if (center - Vec3::new(3.0, 0.2, 0.0)).length() > 0.9 {
                let choose_mat: f32 = rng.gen();

                let material = if choose_mat < 0.8 {
                    let albedo = random_vec(0.0, 1.0) * random_vec(0.0, 1.0);
                    Lambertian::new(albedo)
                } else if choose_mat < 0.95 {
                    let albedo = random_vec(0.5, 1.0);
                    let fuzz   = rng.gen_range(0.0, 0.5);
                    Metal::new(albedo, fuzz)
                } else {
                    Dielectric::new(1.5)
                };

                scene.add(Sphere::new(center, 0.2, &material));
            }
        }
    }

    scene.add(Sphere::new(Vec3::new(0.0, 1.0, 0.0), 1.0,
        &Dielectric::new(1.5)));

    scene.add(Sphere::new(Vec3::new(-4.0, 1.0, 0.0), 1.0,
        &Lambertian::new(Vec3::new(0.4, 0.2, 0.1))));

    scene.add(Sphere::new(Vec3::new(4.0, 1.0, 0.0), 1.0,
        &Metal::new(Vec3::new(0.7, 0.6, 0.5), 0.0)));
}

struct WorkQueue<T: Copy> {
    queue: Vec<T>,
    index: AtomicUsize,
}

impl<T: Copy> WorkQueue<T> {
    pub fn new(queue: Vec<T>) -> Self {
        Self {
            queue,
            index: AtomicUsize::new(0),
        }
    }

    pub fn take(&self) -> Option<T> {
        let index = self.index.fetch_add(1, Ordering::SeqCst);

        if index < self.queue.len() {
            Some(self.queue[index])
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)]
struct PixelRange {
    start: usize,
    size:  usize,
}

fn main() {
    const RANGES_PER_THREAD: usize = 64;
    const PROGRESS_STEP:     usize = 8192;

    let width  = 3840;
    let height = 2160;
    let samples_per_axis = 64;

    let camera = Camera::new(
        Vec3::new(12.0, 2.0, 3.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        20.0,
        width as f32 / height as f32,
    );

    let mut scene = Scene::new();

    //load_scene(&mut scene);
    random_scene(&mut scene);

    scene.construct_bvh();

    let scene       = Arc::new(scene);
    let pixels_done = Arc::new(AtomicUsize::new(0));

    let core_count        = num_cpus::get();
    let total_pixel_count = width * height;

    let queue = {
        let range_count      = core_count * RANGES_PER_THREAD;
        let pixels_per_range = (total_pixel_count + range_count - 1) / range_count;

        let mut ranges = Vec::with_capacity(range_count);

        for i in 0..range_count {
            let start = i * pixels_per_range;

            let outside_screen = if i + 1 == range_count {
                (pixels_per_range * range_count).checked_sub(total_pixel_count).unwrap()
            } else {
                0
            };

            let size = pixels_per_range.checked_sub(outside_screen).unwrap();

            ranges.push(PixelRange {
                start,
                size,
            });
        }

        Arc::new(WorkQueue::new(ranges))
    };

    println!("Raytracing using {} threads...\n", core_count);

    let mut threads = Vec::with_capacity(core_count);
    let start_time  = Instant::now();

    for core in 0..core_count {
        let scene       = scene.clone();
        let camera      = camera.clone();
        let queue       = queue.clone();
        let pixels_done = pixels_done.clone();

        threads.push(thread::spawn(move || {
            pin::pin_to_core(core);

            let mut results = Vec::with_capacity(RANGES_PER_THREAD);

            while let Some(range) = queue.take() {
                let pixel_count = range.size;
                let start_pixel = range.start;

                let mut pixels = Vec::with_capacity(pixel_count);

                for i in 0..pixel_count {
                    let x = (i + start_pixel) % width;
                    let y = (i + start_pixel) / width;

                    let mut color_sum = Vec3::zero();

                    for sx in 0..samples_per_axis {
                        for sy in 0..samples_per_axis {
                            let x = x as f32 + (sx as f32 / (samples_per_axis - 1) as f32);
                            let y = y as f32 + (sy as f32 / (samples_per_axis - 1) as f32);

                            let u = x / width as f32;
                            let v = 1.0 - (y / height as f32);

                            let ray   = camera.ray(u, v);
                            let color = trace_ray(&ray, &scene);

                            color_sum += color;
                        }
                    }

                    if i > 0 {
                        if i % PROGRESS_STEP == 0 {
                            pixels_done.fetch_add(PROGRESS_STEP, Ordering::Relaxed);
                        } else if i == pixel_count - 1 {
                            pixels_done.fetch_add(pixel_count % PROGRESS_STEP,
                                                  Ordering::Relaxed);
                        }
                    }

                    let samples = samples_per_axis * samples_per_axis;

                    let color = color_sum / samples as f32;
                    let color = Vec3::new(color.x.sqrt(), color.y.sqrt(), color.z.sqrt());

                    pixels.push(color);
                }

                results.push((start_pixel, pixels));
            }

            results
        }));
    }

    loop {
        let pixels_done = pixels_done.load(Ordering::Relaxed);
        let progress    = pixels_done as f64 / total_pixel_count as f64;

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

        let progress_per_second = progress / elapsed;
        let left                = (1.0 - progress) / progress_per_second;

        print!("] {:.1}% | {:.3}s elapsed | {:.1}s left", progress * 100.0, elapsed, left);

        if pixels_done == total_pixel_count {
            println!();
            break;
        }

        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(100));
    }

    let mut pixels = vec![Vec3::zero(); total_pixel_count];

    for thread in threads {
        for (start_pixel, buffer) in thread.join().unwrap() {
            pixels[start_pixel..][..buffer.len()].copy_from_slice(&buffer);
        }
    }

    save_image("output.png", &pixels, width, height);
}

fn save_image(path: &str, pixels: &[Vec3], width: usize, height: usize) {
    let mut imgbuf = ImageBuffer::new(width as u32, height as u32);

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let x = x as usize;
        let y = y as usize;

        let color = pixels[x + y * width];
        let r = (color.x * 255.0) as u8;
        let g = (color.y * 255.0) as u8;
        let b = (color.z * 255.0) as u8;

        *pixel = image::Rgb([r, g, b]);
    }

    imgbuf.save(path).expect("Failed to save output image.");
}
