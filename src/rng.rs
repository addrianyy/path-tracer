pub struct Rng(u64);

impl Rng {
    pub fn new() -> Self {
        let seed = if is_x86_feature_detected!("rdseed") {
            loop {
                let mut seed = 0;

                let result = unsafe {
                    std::arch::x86_64::_rdseed64_step(&mut seed)
                };

                if result == 1 {
                    break seed;
                }
            }
        } else {
            unsafe {
                // Hopefully "random" enough.
                std::arch::x86_64::_rdtsc()
            }
        };

        Self::with_seed(seed)
    }

    pub fn with_seed(seed: u64) -> Self {
        let mut rng = Self(seed);

        for _ in 0..10 {
            let _ = rng.raw_rand();
        }

        rng
    }

    #[inline(always)]
    pub fn rand<T: Random>(&mut self) -> T {
        T::rand(self)
    }

    #[inline(always)]
    pub fn rand_range<T: Random + PartialOrd>(&mut self, min: T, max: T) -> T {
        assert!(min < max, "Specified range is invalid.");

        T::rand_range(self, min, max)
    }

    #[inline(always)]
    fn raw_rand(&mut self) -> u64 {
        let mut x = self.0;

        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;

        self.0 = x;

        x
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Random: Sized {
    fn rand(rng: &mut Rng) -> Self;
    fn rand_range(rng: &mut Rng, min: Self, max: Self) -> Self;
}

impl Random for f32 {
    #[inline(always)]
    fn rand(rng: &mut Rng) -> Self {
        let v = rng.raw_rand() as u32;

        (v >> 8) as f32 / (1u32 << 24) as f32
    }

    #[inline(always)]
    fn rand_range(rng: &mut Rng, min: Self, max: Self) -> Self {
        rng.rand::<Self>() * (max - min) + min
    }
}

impl Random for f64 {
    #[inline(always)]
    fn rand(rng: &mut Rng) -> Self {
        let v = rng.raw_rand();

        (v >> 11) as f64 / (1u64 << 53) as f64
    }

    #[inline(always)]
    fn rand_range(rng: &mut Rng, min: Self, max: Self) -> Self {
        rng.rand::<Self>() * (max - min) + min
    }
}

macro_rules! implement_simple_rand {
    ($type: ty) => {
        impl Random for $type {
            #[inline(always)]
            fn rand(rng: &mut Rng) -> Self {
                rng.raw_rand() as Self
            }

            #[inline(always)]
            fn rand_range(_rng: &mut Rng, _min: Self, _max: Self) -> Self {
                unimplemented!()
            }
        }
    };

    ($($type: ty),*) => {
        $( implement_simple_rand! { $type } )*
    };
}

implement_simple_rand! { u8, u16, u32, u64 }
implement_simple_rand! { i8, i16, i32, i64 }
