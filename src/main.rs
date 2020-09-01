#![allow(dead_code)]

mod traceable;
mod material;
mod texture;
mod math;

mod rng;
mod bvh;
mod scene;
mod camera;
mod processors;

pub use math::{Vec3, Ray};

use rng::Rng;
use scene::Scene;
use camera::Camera;
use traceable::Sphere;
use material::{Metal, Lambertian, Dielectric};
use texture::PictureTexture;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use std::io::{self, Write};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

use minifb::{Window, WindowOptions};
use image::RgbImage;

const RANGES_PER_THREAD: usize = 64;
const CHANNELS:          usize = 3;
const PROGRESS_STEP:     usize = 8192;

#[derive(Copy, Clone)]
struct PixelRange {
    start: usize,
    size:  usize,
}

struct PixelQueue {
    queue: Vec<PixelRange>,
    index: AtomicUsize,
}

impl PixelQueue {
    fn new(queue: Vec<PixelRange>) -> Self {
        Self {
            queue,
            index: AtomicUsize::new(0),
        }
    }

    fn pop(&self) -> Option<PixelRange> {
        let index = self.index.fetch_add(1, Ordering::SeqCst);

        if index < self.queue.len() {
            Some(self.queue[index])
        } else {
            None
        }
    }
}

struct State {
    scene:       Scene,
    camera:      Camera,
    queue:       PixelQueue,
    pixels_done: AtomicUsize,
}

impl State {
    fn new(scene: Scene, camera: Camera, pixel_count: usize, threads: usize) -> Self {
        let queue = {
            let range_count      = threads * RANGES_PER_THREAD;
            let pixels_per_range = (pixel_count + range_count - 1) / range_count;

            let mut ranges = Vec::with_capacity(range_count);

            for i in 0..range_count {
                let start = i * pixels_per_range;

                let outside_screen = if i + 1 == range_count {
                    (pixels_per_range * range_count).checked_sub(pixel_count).unwrap()
                } else {
                    0
                };

                let size = pixels_per_range.checked_sub(outside_screen).unwrap();

                ranges.push(PixelRange {
                    start,
                    size,
                });
            }

            PixelQueue::new(ranges)
        };

        Self {
            scene,
            camera,
            queue,
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

    let t     = 0.5 * (ray.direction.y + 1.0);
    let color = Vec3::new(1.0, 1.0, 1.0) * (1.0 - t) + Vec3::new(0.5, 0.7, 1.0) * t;

    color * current_attenuation
}

struct Pixels {
    pixels:     Vec<u8>,
    fb_pixels:  Option<Vec<u32>>,
    width:      usize,
    height:     usize,
}

impl Pixels {
    pub fn new(width: usize, height: usize, window: bool) -> Self {
        let pixel_count = width * height;

        let fb_pixels = if window {
            Some(vec![0; pixel_count])
        } else {
            None
        };

        Self {
            pixels: vec![0; pixel_count * CHANNELS],
            fb_pixels,
            width,
            height,
        }
    }

    pub fn add_fragment(&mut self, (start_pixel, buffer): (usize, Vec<u8>)) {
        self.pixels[start_pixel * CHANNELS..][..buffer.len()].copy_from_slice(&buffer);

        if let Some(fb_pixels) = self.fb_pixels.as_mut() {
            for (offset, chunk) in buffer.chunks(3).enumerate() {
                fb_pixels[start_pixel + offset] =
                    u32::from_be_bytes([0, chunk[0], chunk[1], chunk[2]]);
            }
        }
    }

    pub fn update_window_framebuffer(&mut self, window: &mut Window) {
        let framebuffer = &self.fb_pixels.as_ref().unwrap();

        window.update_with_buffer(framebuffer, self.width, self.height)
            .expect("Failed to update window framebuffer.");
    }

    pub fn disable_window(&mut self) {
        self.fb_pixels = None;
    }
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

    let preview_window = std::env::args().nth(1) == Some("preview".to_string());

    let mut scene = Scene::new();

    load_scene(&mut scene);
    //random_scene(&mut scene);

    scene.construct_bvh();

    let processors        = processors::logical();
    let core_count        = processors.len();
    let total_pixel_count = width * height;

    let (sender, receiver) = mpsc::channel();
    let state              = Arc::new(State::new(scene, camera, total_pixel_count, core_count));

    println!("Raytracing using {} threads...", core_count);

    let mut threads = Vec::with_capacity(core_count);
    let start_time  = Instant::now();

    let mut window = if preview_window {
        Some(Window::new("path-tracer", width, height, WindowOptions::default()).unwrap())
    } else {
        None
    };

    for processor in processors {
        let state  = state.clone();
        let sender = sender.clone();

        threads.push(thread::spawn(move || {
            processors::pin_to_processor(&processor);

            let mut rng = Rng::new();

            while let Some(range) = state.queue.pop() {
                let pixel_count = range.size;
                let start_pixel = range.start;

                let mut pixels = Vec::with_capacity(pixel_count * CHANNELS);

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

                            let ray   = state.camera.ray(u, v);
                            let color = trace_ray(&ray, &state.scene, &mut rng);

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

                    let samples = samples_per_axis * samples_per_axis;
                    let color   = color_sum / samples as f32;

                    let r = (color.x.sqrt() * 255.0) as u8;
                    let g = (color.y.sqrt() * 255.0) as u8;
                    let b = (color.z.sqrt() * 255.0) as u8;

                    pixels.push(r);
                    pixels.push(g);
                    pixels.push(b);
                }

                sender.send((start_pixel, pixels)).unwrap();
            }
        }));
    }

    drop(sender);

    let mut pixels = Pixels::new(width, height, window.is_some());

    loop {
        while let Ok(fragment) = receiver.try_recv() {
            pixels.add_fragment(fragment);
        }

        let pixels_done = state.pixels_done.load(Ordering::Relaxed);
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

        if let Some(wnd) = window.as_mut() {
            pixels.update_window_framebuffer(wnd);

            if !wnd.is_open() {
                window = None;
                pixels.disable_window();
            }
        }

        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(100));
    }

    for fragment in receiver {
        pixels.add_fragment(fragment);
    }

    RgbImage::from_raw(width as u32, height as u32, std::mem::take(&mut pixels.pixels))
        .expect("Failed to create image buffer for the PNG.")
        .save("output.png")
        .expect("Failed to save output image.");

    for thread in threads {
        thread.join().unwrap();
    }

    if let Some(wnd) = window.as_mut() {
        pixels.update_window_framebuffer(wnd);

        while wnd.is_open() {
            wnd.update();
        }
    }
}
