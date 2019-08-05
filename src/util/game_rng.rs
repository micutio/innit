//! A game-global random number generator
//! For more info on Rust RNGs, refer to https://rust-random.github.io/book/guide-rngs.html
//! For infor on serializable RNG, refer to
//!     https://github.com/rsaarelm/calx/blob/45a8d78edda35f2acd59bf9d2bf96fbb98b214ed/calx-alg/src/rng.rs#L33-L84

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

// #[derive(Serialize, Deserialize)]
pub struct GameRng(Pcg32);

impl GameRng {
    pub fn seed_game_rng(seed: [u8; 16]) -> Self {
        GameRng(Pcg32::from_seed(seed))
    }
}

