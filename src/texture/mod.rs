mod solid;
mod picture;

use std::sync::Arc;

use crate::Vec3;

pub use solid::SolidTexture;
pub use picture::PictureTexture;

pub type SharedTexture = Arc<dyn Texture + Send + Sync>;

pub trait Texture {
    fn color(&self, u: f32, v: f32, p: Vec3) -> Vec3;
}

fn make_shared(texture: impl Texture + Send + Sync + 'static) -> SharedTexture {
    Arc::new(texture)
}
