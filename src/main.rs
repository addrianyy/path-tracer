#![allow(dead_code)]

mod parallel_renderer;
mod raytracer;
mod traceable;
mod material;
mod texture;
mod scene;
mod math;
mod rng;

pub use math::{Vec3, Ray};

use scene::Scene;
use math::Camera;
use parallel_renderer::ParallelRenderer;
use raytracer::{Raytracer, Statistics, Pixel};

use std::sync::atomic::Ordering;
use std::time::Duration;
use std::io::{self, Write};
use std::sync::Arc;
use std::thread;

use image::RgbImage;

fn flatten_image(mut buffer: Vec<[u8; 3]>) -> Vec<u8> {
    let buf = buffer.as_mut_ptr();
    let len = buffer.len();
    let cap = buffer.capacity();

    assert!(std::mem::align_of::<u8>() == std::mem::align_of::<[u8; 3]>());

    std::mem::forget(buffer);

    unsafe {
        Vec::from_raw_parts(buf as *mut u8, len * 3, cap * 3)
    }
}

fn reporter(stats: &Statistics, pixel_count: usize) {
    loop {
        let pixels_done = stats.pixels_done.load(Ordering::Relaxed);
        let progress    = pixels_done as f64 / pixel_count as f64;

        let elapsed = stats.elapsed();

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
}

fn main() {
    let width   = 3840 - 200;
    let height  = 2160 - 200;
    let samples = 16;

    let camera = Camera::new(
        Vec3::new(12.0, 2.0, 3.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        20.0,
        width,
        height,
    );

    let mut scene = Scene::new();

    scene::generators::simple_scene(&mut scene);
    //scene::generators::random_scene(&mut scene);

    let raytracer   = Raytracer::new(camera, scene, samples);
    let pixel_count = raytracer.pixel_count();

    let mut renderer = ParallelRenderer::new();
    let mut buffer   = vec![Pixel::default(); pixel_count];

    let stats = Arc::new(Statistics::new());

    let reporter = {
        let stats = stats.clone();

        thread::spawn(move || reporter(&stats, pixel_count))
    };

    let context = (raytracer, stats);

    renderer.render(&context, &mut buffer, move |context, rng, start_pixel, pixels| {
        let (raytracer, stats) = context;

        raytracer.render_fragment(start_pixel, pixels, &stats, rng);
    });

    reporter.join().unwrap();

    RgbImage::from_raw(width as u32, height as u32, flatten_image(buffer))
        .expect("Failed to create image buffer for the PNG.")
        .save("output.png")
        .expect("Failed to save output image.");
}
