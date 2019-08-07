use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::mem;

/// A seedable random number generator that can be serialized for consistent random number
/// generation. For more info on Rust RNGs, refer to https://rust-random.github.io/book/guide-rngs.html
/// For infor on serializable RNG, refer to
///     https://github.com/rsaarelm/calx/blob/45a8d78edda35f2acd59bf9d2bf96fbb98b214ed/calx-alg/src/rng.rs#L33-L84
#[derive(Clone, Debug)]
pub struct GameRng<T> {
    inner: T,
}

impl<T: Rng + 'static> GameRng<T> {
    pub fn new(inner: T) -> GameRng<T> {
        GameRng { inner }
    }
}

impl<T: SeedableRng + Rng + 'static> SeedableRng for GameRng<T> {
    // For implementing seed refer to: https://rust-random.github.io/rand/rand_core/trait.SeedableRng.html
    type Seed = <T as SeedableRng>::Seed;

    // fn from_seed(seed: <T as rand::SeedableRng>::Seed) -> GameRng<T> {
    fn from_seed(seed: Self::Seed) -> GameRng<T> {
        GameRng::new(SeedableRng::from_seed(seed))
    }
}

impl<T: Rng + 'static> Serialize for GameRng<T> {
    /// Serialize the rng as a binary blob.
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut vec = Vec::new();
        unsafe {
            let view = self as *const _ as *const u8;
            for i in 0..(mem::size_of::<T>()) {
                // vec.push(*view.offset(i as isize));
                vec.push(*view.add(i));
            }
        }
        vec.serialize(s)
    }
}

impl<'a, T: Rng + 'static> Deserialize<'a> for GameRng<T> {
    /// Deserialize the rng from a binary blob.
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        let bin_blob: Vec<u8> = serde::Deserialize::deserialize(d)?;
        unsafe {
            if bin_blob.len() == mem::size_of::<T>() {
                Ok(GameRng::new(mem::transmute_copy(&bin_blob[0])))
            } else {
                Err(serde::de::Error::invalid_length(
                    bin_blob.len(),
                    &"Bad inner RNG length",
                ))
            }
        }
    }
}
