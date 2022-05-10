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
    pub tile_size: i32,
    /// if true: run innit in debug mode
    pub is_debug_mode: bool,
    /// optional fixed rng seed
    pub seed: Option<u64>,
    /// optional turn limit
    pub turn_limit: Option<u128>,
    /// if true: do not create a player object
    pub is_spectating: bool,
    pub is_particles_disabled: bool,
    pub is_gfx_disabled: bool,
    /// visualisation of the commplement system:
    /// - 0, 1, 2 -> display respective pathway
    /// - 3 -> don't show
    pub complement_system_display: usize,
}

impl GameEnv {
    pub fn new() -> Self {
        GameEnv {
            tile_size: 16,
            is_debug_mode: false,
            seed: None,
            turn_limit: None,
            is_spectating: false,
            is_particles_disabled: false,
            is_gfx_disabled: true,
            complement_system_display: 3,
        }
    }

    pub fn set_tile_size(&mut self, tile_size: i32) {
        self.tile_size = tile_size;
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

    pub fn set_disable_particles(&mut self, disable_particles: bool) {
        self.is_particles_disabled = disable_particles;
    }

    pub fn set_disable_gfx(&mut self, disable_gfx: bool) {
        self.is_gfx_disabled = disable_gfx;
    }

    pub fn set_complement_display(&mut self, display_mode: usize) {
        self.complement_system_display = display_mode;
    }
}
