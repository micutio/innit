use serde::{Deserialize, Serialize};
use std::sync::{Mutex, MutexGuard};

lazy_static! {
    static ref GAME_ENV: Mutex<GameEnv> = Mutex::new(GameEnv::new());
}

pub fn env<'a>() -> MutexGuard<'a, GameEnv> {
    GAME_ENV.lock().unwrap()
}

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub struct GameEnv {
    /// if true: run innit in debug mode
    pub is_debug_mode: bool,
    /// if true: use random seed for reproducible random number generation
    pub is_using_fixed_seed: bool,
    /// if trie: do not create a player object
    pub is_spectating: bool,
}

impl GameEnv {
    pub fn new() -> Self {
        GameEnv {
            is_debug_mode: false,
            is_using_fixed_seed: false,
            is_spectating: false,
        }
    }

    pub fn set_debug_mode(&mut self, debug_mode: bool) {
        self.is_debug_mode = debug_mode;
    }

    pub fn set_rng_seeding(&mut self, use_fixed_seed: bool) {
        self.is_using_fixed_seed = use_fixed_seed;
    }

    pub fn set_spectating(&mut self, spectate_only: bool) {
        self.is_spectating = spectate_only;
    }
}
