use super::{Texture, SharedTexture};
use crate::Vec3;

pub struct SolidTexture {
    color: Vec3,
}

impl SolidTexture {
    pub fn new(color: Vec3) -> SharedTexture {
        super::make_shared(Self {
            color,
        })
    }
}

impl Texture for SolidTexture {
    fn color(&self, _u: f32, _v: f32, _p: Vec3) -> Vec3 {
        self.color
    }
}
