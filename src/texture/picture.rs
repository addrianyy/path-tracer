use super::{Texture, SharedTexture};
use crate::Vec3;

pub struct PictureTexture {
    image: image::RgbImage,
}

impl PictureTexture {
    pub fn new(path: &str) -> SharedTexture {
        super::make_shared(Self {
            image: image::open(path).unwrap().into_rgb(),
        })
    }
}

impl Texture for PictureTexture {
    fn color(&self, u: f32, v: f32, _p: Vec3) -> Vec3 {
        let width  = self.image.width()  as i32;
        let height = self.image.height() as i32;

        let i = (u * width as f32) as i32;
        let j = (((1.0 - v) * height as f32) - 0.001) as i32;

        let clamp = |value: i32, min, max| {
            value.min(max).max(min)
        };

        let i = clamp(i, 0, width  - 1);
        let j = clamp(j, 0, height - 1);

        let pixel = self.image.get_pixel(i as u32, j as u32);

        let r = pixel.0[0] as f32 / 255.0;
        let g = pixel.0[1] as f32 / 255.0;
        let b = pixel.0[2] as f32 / 255.0;

        Vec3::new(r, g, b)
    }
}
