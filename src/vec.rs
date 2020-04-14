#![allow(dead_code)]

use std::ops::Index;
use std::ops::{Add, Sub, Mul, Div, Neg, AddAssign, SubAssign, MulAssign, DivAssign};

#[derive(Default, Debug, Copy, Clone)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn fill(v: f32) -> Self {
        Self { x: v, y: v, z: v }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }

    pub fn length_sqr(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(self) -> f32 {
        self.length_sqr().sqrt()
    }

    pub fn normalized(self) -> Self {
        self / self.length()
    }

    pub fn dot(a: Self, b: Self) -> f32 {
        a.x * b.x + a.y * b.y + a.z * b.z
    }

    pub fn cross(a: Self, b: Self) -> Self {
        let x =   a.y * b.z - a.z * b.y;
        let y = -(a.x * b.z - a.z * b.x);
        let z =   a.x * b.y - a.y * b.x;

        Self::new(x, y, z)
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, v: f32) -> Self {
        Self::new(self.x * v, self.y * v, self.z * v)
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, v: f32) -> Self {
        Self::new(self.x / v, self.y / v, self.z / v)
    }
}

impl Mul for Vec3 {
    type Output = Self;

    fn mul(self, other: Vec3) -> Self {
        Self::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }
}

impl Div for Vec3 {
    type Output = Self;

    fn div(self, other: Vec3) -> Self {
        Self::new(self.x / other.x, self.y / other.y, self.z / other.z)
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

impl Index<usize> for Vec3 {
    type Output = f32;

    fn index(&self, index: usize) -> &f32 {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Invalid Vec3 index {}", index),
        }
    }
}
