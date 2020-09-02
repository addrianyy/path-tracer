use core::arch::x86_64::*;

fn random_seed() -> u64 {
    assert!(is_x86_feature_detected!("rdseed"), "rdseed is not supported by the current CPU.");

    loop {
        let mut seed = 0;
        
        if unsafe { _rdseed64_step(&mut seed) } == 1 {
            break seed;
        }
    }
}

pub struct Rng256 {
    state:   __m256i,
    shift13: __m128i,
    shift7:  __m128i,
    shift17: __m128i,
}

impl Rng256 {
    pub fn new() -> Self {
        Self::with_seeds(
            [random_seed(), random_seed(), random_seed(), random_seed()]
        )
    }

    pub fn with_seeds(seeds: [u64; 4]) -> Self {
        assert!(is_x86_feature_detected!("avx2"), "AVX2 is not supported by the current CPU.");

        let mut rng = unsafe {
            Self {
                state:   _mm256_loadu_si256(&seeds as *const _ as *const _),
                shift13: _mm_set1_epi64x(13),
                shift7:  _mm_set1_epi64x(7),
                shift17: _mm_set1_epi64x(17),
            }
        };

        for _ in 0..10 {
            let _ = rng.rand();
        }

        rng
    }

    pub fn rand(&mut self) -> [u32; 8] {
        let mut x = self.state;

        unsafe {
            x = _mm256_xor_si256(x, _mm256_sll_epi64(x, self.shift13));
            x = _mm256_xor_si256(x, _mm256_srl_epi64(x, self.shift7));
            x = _mm256_xor_si256(x, _mm256_sll_epi64(x, self.shift17));
        }

        self.state = x;

        let mut result = [0u32; 8];

        unsafe {
            _mm256_storeu_si256(result.as_mut_ptr() as *mut _, x);
        }

        result
    }
}

pub struct Rng {
    generator: Rng256,
    buffer: [u32; 8],
    index:  usize,
}

impl Rng {
    pub fn new() -> Self {
        let mut generator = Rng256::new();

        Rng {
            buffer: generator.rand(),
            index:  0,
            generator,
        }
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

    fn raw_rand(&mut self) -> u32 {
        let value = self.buffer[self.index];

        self.index += 1;

        if self.index >= self.buffer.len() {
            self.index  = 0;
            self.buffer = self.generator.rand();
        }

        value
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
