use rand::{Rng, SeedableRng};

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

    // fn reseed(&mut self, seed: S) {
    //     self.inner.reseed(seed);
    // }

    // For implementing seed refer to: https://rust-random.github.io/rand/rand_core/trait.SeedableRng.html
    // type Seed: Sized + Default + asMut<[u8]>;


    fn from_seed(seed: <T as rand::SeedableRng>::Seed) -> GameRng<T> {
        GameRng::new(SeedableRng::from_seed(seed))
    }
}
