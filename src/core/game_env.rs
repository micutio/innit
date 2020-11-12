use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub struct GameEnv {
    /// if true: run innit in debug mode
    pub debug_mode: bool,
    /// if true: use random seed for reproducible random number generation
    pub use_fixed_seed: bool,
}

impl GameEnv {
    pub fn new() -> Self {
        GameEnv {
            debug_mode: false,
            use_fixed_seed: false,
        }
    }

    pub fn set_debug_mode(&mut self, debug_mode: bool) {
        self.debug_mode = debug_mode;
    }

    pub fn set_rng_seeding(&mut self, use_fixed_seed: bool) {
        self.use_fixed_seed = use_fixed_seed;
    }
}
