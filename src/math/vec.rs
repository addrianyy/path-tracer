#![allow(dead_code)]

use std::ops::{Add, Sub, Mul, Div, Neg, AddAssign, SubAssign, MulAssign, DivAssign};

mod vectorized {
    use std::arch::x86_64::*;

    pub type ArchVector = __m128;

    pub fn new(x: f32, y: f32, z: f32) -> ArchVector {
        let array = [x, y, z, 0.0];

        unsafe {
            _mm_loadu_ps(array.as_ptr() as *const _ as _)
        }
    }

    pub fn extract(vector: ArchVector) -> (f32, f32, f32) {
        let mut array = [0.0, 0.0, 0.0, 0.0];

        unsafe {
            _mm_storeu_ps(array.as_mut_ptr() as *mut _ as _, vector);
        }

        (array[0], array[1], array[2])
    }

    pub fn sum(v: ArchVector) -> f32 {
        let (x, y, z) = extract(v);

        x + y + z
    }

    pub fn product(vector: ArchVector) -> f32 {
        let (x, y, z) = extract(vector);

        x * y * z
    }

    pub fn zero() -> ArchVector { unsafe { _mm_setzero_ps() } }
    pub fn fill(v: f32) -> ArchVector { unsafe { _mm_set1_ps(v) } }

    pub fn add(a: ArchVector, b: ArchVector) -> ArchVector { unsafe { _mm_add_ps(a, b) } }
    pub fn sub(a: ArchVector, b: ArchVector) -> ArchVector { unsafe { _mm_sub_ps(a, b) } }
    pub fn mul(a: ArchVector, b: ArchVector) -> ArchVector { unsafe { _mm_mul_ps(a, b) } }
    pub fn div(a: ArchVector, b: ArchVector) -> ArchVector { unsafe { _mm_div_ps(a, b) } }
    pub fn min(a: ArchVector, b: ArchVector) -> ArchVector { unsafe { _mm_min_ps(a, b) } }
    pub fn max(a: ArchVector, b: ArchVector) -> ArchVector { unsafe { _mm_max_ps(a, b) } }
    pub fn sqrt(a: ArchVector) -> ArchVector { unsafe { _mm_sqrt_ps(a) } }
}

use vectorized::ArchVector;

#[derive(Copy, Clone)]
pub struct Vec3 {
    vector: ArchVector,
}

impl Vec3 {
    fn vector(&self) -> ArchVector {
        self.vector
    }

    fn from_vector(vector: ArchVector) -> Self {
        Self {
            vector,
        }
    }

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self::from_vector(vectorized::new(x, y, z))
    }

    pub fn fill(v: f32) -> Self {
        Self::from_vector(vectorized::fill(v))
    }

    pub fn zero() -> Self {
        Self::from_vector(vectorized::zero())
    }

    pub fn length_sqr(self) -> f32 {
        let v = self.vector();

        vectorized::sum(vectorized::mul(v, v))
    }

    pub fn length(self) -> f32 {
        self.length_sqr().sqrt()
    }

    pub fn normalized(self) -> Self {
        self / self.length()
    }

    pub fn dot(a: Self, b: Self) -> f32 {
        vectorized::sum(vectorized::mul(a.vector(), b.vector()))
    }

    pub fn cross(a: Self, b: Self) -> Self {
        let (ax, ay, az) = a.extract();
        let (bx, by, bz) = b.extract();

        let x =   ay * bz - az * by;
        let y = -(ax * bz - az * bx);
        let z =   ax * by - ay * bx;

        Self::new(x, y, z)
    }

    pub fn min(a: Self, b: Self) -> Self {
        Self::from_vector(vectorized::min(a.vector(), b.vector()))
    }

    pub fn max(a: Self, b: Self) -> Self {
        Self::from_vector(vectorized::max(a.vector(), b.vector()))
    }

    pub fn sqrt(&self) -> Self {
        Self::from_vector(vectorized::sqrt(self.vector()))
    }

    pub fn extract(&self) -> (f32, f32, f32) {
        vectorized::extract(self.vector())
    }

    pub fn extract_array(&self) -> [f32; 3] {
        let (x, y, z) = self.extract();
        [x, y, z]
    }

    pub fn x(&self) -> f32 { self.extract().0 }
    pub fn y(&self) -> f32 { self.extract().1 }
    pub fn z(&self) -> f32 { self.extract().2 }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self {
        self * Vec3::fill(-1.0)
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::from_vector(vectorized::add(self.vector(), other.vector()))
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::from_vector(vectorized::sub(self.vector(), other.vector()))
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, v: f32) -> Self {
        self * Vec3::fill(v)
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, v: f32) -> Self {
        self / Vec3::fill(v)
    }
}

impl Mul for Vec3 {
    type Output = Self;

    fn mul(self, other: Vec3) -> Self {
        Self::from_vector(vectorized::mul(self.vector(), other.vector()))
    }
}

impl Div for Vec3 {
    type Output = Self;

    fn div(self, other: Vec3) -> Self {
        Self::from_vector(vectorized::div(self.vector(), other.vector()))
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, v: f32) {
        *self = *self * v;
    }
}

impl DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, v: f32) {
        *self = *self / v;
    }
}

impl MulAssign for Vec3 {
    fn mul_assign(&mut self, other: Vec3) {
        *self = *self * other;
    }
}

impl DivAssign for Vec3 {
    fn div_assign(&mut self, other: Vec3) {
        *self = *self / other;
    }
}
