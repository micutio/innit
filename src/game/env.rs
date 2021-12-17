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
    /// optional fixed rng seed
    pub seed: Option<u64>,
    /// optional turn limit
    pub turn_limit: Option<u128>,
    /// if trie: do not create a player object
    pub is_spectating: bool,
}

impl GameEnv {
    pub fn new() -> Self {
        GameEnv {
            is_debug_mode: false,
            seed: None,
            turn_limit: None,
            is_spectating: false,
        }
    }

    pub fn set_debug_mode(&mut self, debug_mode: bool) {
        self.is_debug_mode = debug_mode;
    }

    pub fn set_seed(&mut self, seed_param: u64) {
        self.seed = Some(seed_param);
    }

    pub fn set_turn_limit(&mut self, limit: u128) {
        self.turn_limit = Some(limit);
    }

    pub fn set_spectating(&mut self, spectate_only: bool) {
        self.is_spectating = spectate_only;
    }
}
