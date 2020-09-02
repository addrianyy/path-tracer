#![allow(dead_code)]

use std::ops::{Add, Sub, Mul, Div, Neg, AddAssign, SubAssign, MulAssign, DivAssign};

mod vectorized {
    use std::arch::x86_64::*;

    #[repr(C, align(16))]
    struct AlignedSseArray([f32; 4]);

    pub type ArchVector = __m128;

    pub fn new(x: f32, y: f32, z: f32) -> ArchVector {
        let array = AlignedSseArray([x, y, z, 0.0]);

        unsafe {
            _mm_load_ps(array.0.as_ptr())
        }
    }

    pub fn fill(v: f32) -> ArchVector { unsafe { _mm_set1_ps(v) } }
    pub fn zero()       -> ArchVector { unsafe { _mm_setzero_ps() } }

    pub fn extract(vector: ArchVector) -> (f32, f32, f32) {
        let mut array = AlignedSseArray([0.0, 0.0, 0.0, 0.0]);

        unsafe {
            _mm_store_ps(array.0.as_mut_ptr(), vector);
        }

        (array.0[0], array.0[1], array.0[2])
    }

    pub fn sum(vector: ArchVector) -> f32 {
        let (x, y, z) = extract(vector);

        x + y + z
    }

    pub fn product(vector: ArchVector) -> f32 {
        let (x, y, z) = extract(vector);

        x * y * z
    }

    /*
    pub fn cross_product(a: ArchVector, b: ArchVector) -> ArchVector {
        unsafe {
            let a_yzx = _mm_shuffle_ps(a, a, _MM_SHUFFLE(3, 0, 2, 1));
            let b_yzx = _mm_shuffle_ps(b, b, _MM_SHUFFLE(3, 0, 2, 1));
            let c     = _mm_sub_ps(_mm_mul_ps(a, b_yzx), _mm_mul_ps(a_yzx, b));

            _mm_shuffle_ps(c, c, _MM_SHUFFLE(3, 0, 2, 1))
        }
    }
    */

    pub fn normalize(v: ArchVector) -> ArchVector {
        unsafe {
            _mm_mul_ps(v, _mm_rsqrt_ps(fill(sum(mul(v, v)))))
        }
    }

    pub fn add(a: ArchVector, b: ArchVector) -> ArchVector { unsafe { _mm_add_ps(a, b) } }
    pub fn sub(a: ArchVector, b: ArchVector) -> ArchVector { unsafe { _mm_sub_ps(a, b) } }
    pub fn mul(a: ArchVector, b: ArchVector) -> ArchVector { unsafe { _mm_mul_ps(a, b) } }
    pub fn div(a: ArchVector, b: ArchVector) -> ArchVector { unsafe { _mm_div_ps(a, b) } }
    pub fn min(a: ArchVector, b: ArchVector) -> ArchVector { unsafe { _mm_min_ps(a, b) } }
    pub fn max(a: ArchVector, b: ArchVector) -> ArchVector { unsafe { _mm_max_ps(a, b) } }
    pub fn sqrt(a: ArchVector)               -> ArchVector { unsafe { _mm_sqrt_ps(a) } }

    #[allow(non_snake_case)]
    const fn _MM_SHUFFLE(z: u32, y: u32, x: u32, w: u32) -> i32 {
        ((z << 6) | (y << 4) | (x << 2) | w) as i32
    }
}

use vectorized::ArchVector;

#[derive(Copy, Clone)]
pub struct Vec3 {
    vector: ArchVector,
}

impl Vec3 {
    #[inline(always)]
    fn vector(&self) -> ArchVector {
        self.vector
    }

    #[inline(always)]
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
         Self::from_vector(vectorized::normalize(self.vector()))
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
