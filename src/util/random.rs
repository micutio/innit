use rand::seq::SliceRandom;
use rand::{Rng, RngCore, SeedableRng};
use rand_core::{impls, Error};
use rand_isaac::isaac64::Isaac64Rng;
use serde::{Deserialize, Serialize};
use std::mem;

// Type of RNG to be used in-game.
pub type GameRng = SerializableRng<Isaac64Rng>;

/// A seedable random number generator that can be serialized for consistent random number
/// generation. For more info on Rust RNGs, refer to
/// <https://rust-random.github.io/book/guide-rngs.html>
/// For an example implementation of serializable RNG, refer to
/// <https://github.com/rsaarelm/calx/blob/45a8d78edda35f2acd59bf9d2bf96fbb98b214ed/calx-alg/src/rng.rs#L33-L84>
#[derive(Clone, Debug)]
pub struct SerializableRng<T> {
    inner: T,
}

impl<T: Rng + 'static> SerializableRng<T> {
    pub const fn new(inner: T) -> SerializableRng<T> {
        SerializableRng { inner }
    }
}

impl<T: SeedableRng + 'static> SerializableRng<T> {
    pub fn new_from_u64_seed(seed: u64) -> SerializableRng<T> {
        SerializableRng {
            inner: SeedableRng::seed_from_u64(seed),
        }
    }
}

impl<T: SeedableRng + Rng + 'static> SeedableRng for SerializableRng<T> {
    // For implementing seed refer to: https://rust-random.github.io/rand/rand_core/trait.SeedableRng.html
    type Seed = <T as SeedableRng>::Seed;

    // fn from_seed(seed: <T as rand::SeedableRng>::Seed) -> SerializableRng<T> {
    fn from_seed(seed: Self::Seed) -> SerializableRng<T> {
        SerializableRng::new(SeedableRng::from_seed(seed))
    }
}

impl<T: Rng + 'static> Serialize for SerializableRng<T> {
    /// Serialize the rng as a binary blob.
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut vec = Vec::new();
        unsafe {
            #[allow(clippy::ptr_as_ptr)]
            let view = self as *const _ as *const u8;
            for i in 0..(mem::size_of::<T>()) {
                // vec.push(*view.offset(i as isize));
                vec.push(*view.add(i));
            }
        }
        vec.serialize(s)
    }
}

impl<'a, T: Rng + 'static> Deserialize<'a> for SerializableRng<T> {
    /// Deserialize the rng from a binary blob.
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        let bin_blob: Vec<u8> = serde::Deserialize::deserialize(d)?;
        unsafe {
            if bin_blob.len() == mem::size_of::<T>() {
                Ok(SerializableRng::new(mem::transmute_copy(&bin_blob[0])))
            } else {
                Err(serde::de::Error::invalid_length(
                    bin_blob.len(),
                    &"Bad inner RNG length",
                ))
            }
        }
    }
}

impl<T: Rng> RngCore for SerializableRng<T> {
    fn next_u32(&mut self) -> u32 {
        self.inner.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.inner.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        impls::fill_bytes_via_next(self, dest);
    }

    #[allow(clippy::unit_arg)]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        Ok(self.fill_bytes(dest))
    }
}

/// Game specific methods for random number generators.
pub trait RngExtended {
    /// Return true or false with 50/50 chance of being true
    fn coinflip(&mut self) -> bool;

    fn flip_with_prob(&mut self, probability: f64) -> bool;

    fn random_bit(&mut self) -> u8;
}

impl<T: Rng> RngExtended for SerializableRng<T> {
    fn coinflip(&mut self) -> bool {
        self.gen_bool(0.5)
    }

    fn flip_with_prob(&mut self, probability: f64) -> bool {
        self.gen_bool(probability)
    }

    fn random_bit(&mut self) -> u8 {
        *vec![1, 2, 4, 8, 16, 32, 64, 128].choose(self).unwrap()
    }
}
