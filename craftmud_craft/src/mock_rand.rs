#[derive(Debug, Clone)]
pub struct IterRng<I: Iterator<Item=u64>> {
    iter: I,
}

impl<I: Iterator<Item=u64>> IterRng<I> {
    /// Create a `IterRng`, yielding a deterministic sequence given by the
    /// iteraor. After exhausting the iter, returns zeros.
    pub fn new(iter: I) -> Self {
        Self {
            iter,
        }
    }
}

impl IterRng<std::vec::IntoIter<u64>> {
    pub fn new_from_vec(vec: Vec<u64>) -> Self {
        Self {
            iter: vec.into_iter(),
        }
    }
}

impl<I: Iterator<Item=u64>> rand_core::RngCore for IterRng<I> {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as _
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.iter.next().unwrap_or(0u64)
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        rand_core::impls::fill_bytes_via_next(self, dest);
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

pub fn gen_uniform_sample_u32(low: u32, high: u32, target: u32) -> u64 {
    let range = (high - low + 1) as u64;
    let target_hi = ((target - low) as u64) << 32;
    let gen = target_hi / range;
    gen + 1
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::{
        Rng,
        // distributions::{Distribution, Uniform}
    };

    #[test]
    fn x() {
        const LOW: u32 = 1;
        const HIGH: u32 = 100;
        const TARGET: u32 = 50;

        let mut iter_rng = IterRng::new(
            std::iter::repeat(gen_uniform_sample_u32(LOW, HIGH, TARGET) as u64)
        );

        let uniform = UniformU32::new_inclusive(LOW, HIGH);
        assert_eq!(Some(TARGET), uniform.sample(&mut iter_rng));
    }

    struct UniformU32 {
        low: u32,
        range: u32,
        ints_to_reject: u32,
    }

    impl UniformU32 {

        fn new_inclusive(low: u32, high: u32) -> Self {
            let unsigned_max = u32::MAX;

            let range = high.wrapping_sub(low).wrapping_add(1);
            let ints_to_reject = if range > 0 {
                (unsigned_max - range + 1) % range
            } else {
                0
            };

            Self {
                low,
                range,
                ints_to_reject,
            }
        }

        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<u32> {
            fn wmul(lhs: u32, rhs: u32) -> (u32, u32) {
                let tmp = (lhs as u64) * (rhs as u64);
                ((tmp >> 32) as u32, tmp as u32)
            }

            let range = self.range;
            if range > 0 {
                let unsigned_max = u32::MAX;
                let zone = unsigned_max - self.ints_to_reject;
                //  loop {
                    let v = rng.gen();
                    let (hi, lo) = wmul(v, range);

                    // println!("lo: {:#8X}", lo);
                    // println!("zone: {:#8X}", zone);

                    if lo <= zone {
                        Some(self.low.wrapping_add(hi))
                    } else {
                        None
                    }
                // }
            } else {
                // Sample from the entire integer range.
                rng.gen()
            }
        }
    }
}