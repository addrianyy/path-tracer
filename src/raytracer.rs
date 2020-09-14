use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use crate::{Vec3, Ray};
use crate::rng::Rng;
use crate::scene::Scene;
use crate::math::Camera;

pub type Pixel = [u8; 3];

pub struct Statistics {
    pub pixels_done: AtomicUsize,
    start_time:      Instant,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            pixels_done: AtomicUsize::new(0),
            start_time:  Instant::now(),
        }
    }

    pub fn elapsed(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }
}

pub struct Raytracer {
    scene:   Scene,
    camera:  Camera,
    samples: usize,
}

impl Raytracer {
    pub fn new(camera: Camera, mut scene: Scene, samples: usize) -> Self {
        scene.construct_bvh();

        Self {
            camera,
            scene,
            samples,
        }
    }

    #[inline(always)]
    fn trace_ray(&self, mut ray: Ray, rng: &mut Rng) -> Vec3 {
        const MAX_TRACES: usize = 5;

        let mut attenuation = Vec3::fill(1.0);

        for _ in 0..MAX_TRACES {
            if let Some(record) = self.scene.trace(&ray) {
                let scattered = record.material.scatter(&ray, &record, rng);

                if let Some((att_multiplier, new_ray)) = scattered {
                    attenuation *= att_multiplier;
                    ray          = new_ray;
                } else {
                    return Vec3::fill(0.0);
                }
            } else {
                break;
            }
        }

        let t     = 0.5 * (ray.direction.extract().1 + 1.0);
        let color = Vec3::new(1.0, 1.0, 1.0) * (1.0 - t) + Vec3::new(0.5, 0.7, 1.0) * t;

        color * attenuation
    }

    #[inline(always)]
    fn trace_pixel(&self, x: usize, y: usize, rng: &mut Rng) -> Vec3 {
        let mut color_sum = Vec3::zero();

        for sx in 0..self.samples {
            let x = x as f32 + (sx as f32 / (self.samples - 1) as f32);
            let u = x / self.width() as f32;

            for sy in 0..self.samples {
                let y = y as f32 + (sy as f32 / (self.samples - 1) as f32);
                let v = 1.0 - (y / self.height() as f32);

                let ray   = self.camera.ray(u, v);
                let color = self.trace_ray(ray, rng);

                color_sum += color;
            }
        }

        let samples = self.samples * self.samples;
        let color   = (color_sum / samples as f32).sqrt();

        color
    }

    pub fn render_fragment(&self, start_pixel: usize, pixels: &mut [Pixel],
                           stats: &Statistics, rng: &mut Rng) {
        const PROGRESS_STEP: usize = 8192;

        let pixel_count = pixels.len();

        for (i, pixel) in pixels.iter_mut().enumerate() {
            let x = (i + start_pixel) % self.width();
            let y = (i + start_pixel) / self.width();

            let color = self.trace_pixel(x, y, rng);

            if i > 0 {
                if i % PROGRESS_STEP == 0 {
                    stats.pixels_done.fetch_add(PROGRESS_STEP, Ordering::Relaxed);
                } else if i == pixel_count - 1 {
                    stats.pixels_done.fetch_add(pixel_count % PROGRESS_STEP,
                                                Ordering::Relaxed);
                }
            }

            let (r, g, b) = (color * 255.0).extract();

            *pixel = [r as u8, g as u8, b as u8];
        }
    }

    #[inline(always)]
    pub fn width(&self) -> usize { self.camera.width() }

    #[inline(always)]
    pub fn height(&self) -> usize { self.camera.height() }

    #[inline(always)]
    pub fn pixel_count(&self) -> usize { self.width() * self.height() }
}
