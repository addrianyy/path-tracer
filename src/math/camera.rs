use super::{Vec3, Ray};

#[derive(Clone)]
pub struct Camera {
    lower_left_corner: Vec3,
    origin:            Vec3,
    horizontal:        Vec3,
    vertical:          Vec3,
    width:             usize,
    height:            usize,
}

impl Camera {
    pub fn new(eyes: Vec3, target: Vec3, up: Vec3, fov: f32, width: usize, height: usize) -> Self {
        let aspect_ratio = width as f32 / height as f32;

        let half_height = (fov.to_radians() / 2.0).tan();
        let half_width  = half_height * aspect_ratio;

        let w = (eyes - target).normalized();
        let u = Vec3::cross(up, w).normalized();
        let v = Vec3::cross(w, u);

        let lower_left_corner = eyes - u * half_width - v * half_height - w;

        Self {
            width,
            height,
            lower_left_corner,
            origin:     eyes,
            horizontal: u * 2.0 * half_width,
            vertical:   v * 2.0 * half_height,
        }
    }

    #[inline(always)]
    pub fn ray(&self, u: f32, v: f32) -> Ray {
        let direction = (self.lower_left_corner + self.horizontal * u + self.vertical * v) -
            self.origin;

        // `direction` is almost normalized so it should be fine. Calling normal `new`
        // will significantly degrade the performance.
        Ray::new_normalized(self.origin, direction)
    }

    #[inline(always)]
    pub fn width(&self) -> usize { self.width }

    #[inline(always)]
    pub fn height(&self) -> usize { self.height }
}
