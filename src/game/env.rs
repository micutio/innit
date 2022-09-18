use serde::{Deserialize, Serialize};
use std::sync::{Mutex, MutexGuard};

lazy_static! {
    static ref GAME_ENV: Mutex<Environment> = Mutex::new(Environment::new());
}

pub fn env<'a>() -> MutexGuard<'a, Environment> {
    GAME_ENV.lock().unwrap()
}

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
#[allow(clippy::use_self)]
pub enum GameOption {
    Enabled,
    #[default]
    Disabled,
}

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub struct Environment {
    /// if true: run innit in debug mode
    pub debug_mode: GameOption,
    /// if true: do not create a player object
    pub spectating: GameOption,
    pub particles: GameOption,
    pub gfx: GameOption,
    pub tile_size: i32,
    /// optional fixed rng seed
    pub seed: Option<u64>,
    /// optional turn limit
    pub turn_limit: Option<u128>,
    /// visualisation of the commplement system:
    /// - 0, 1, 2 -> display respective pathway
    /// - 3 -> don't show
    pub complement_system_display: usize,
}

impl Environment {
    pub const fn new() -> Self {
        Self {
            debug_mode: GameOption::Disabled,
            spectating: GameOption::Disabled,
            particles: GameOption::Enabled,
            gfx: GameOption::Disabled,
            tile_size: 16,
            seed: None,
            turn_limit: None,
            complement_system_display: 3,
        }
    }

    pub fn set_debug_mode(&mut self, is_enabled: bool) {
        self.debug_mode = if is_enabled {
            GameOption::Enabled
        } else {
            GameOption::Disabled
        };
    }

    pub fn set_spectating(&mut self, is_enabled: bool) {
        self.spectating = if is_enabled {
            GameOption::Enabled
        } else {
            GameOption::Disabled
        };
    }

    pub fn set_particles(&mut self, is_enabled: bool) {
        self.particles = if is_enabled {
            GameOption::Enabled
        } else {
            GameOption::Disabled
        };
    }

    pub fn set_disable_gfx(&mut self, is_enabled: bool) {
        self.gfx = if is_enabled {
            GameOption::Enabled
        } else {
            GameOption::Disabled
        };
    }

    pub fn set_tile_size(&mut self, tile_size: i32) {
        self.tile_size = tile_size;
    }

    pub fn set_seed(&mut self, seed_param: u64) {
        self.seed = Some(seed_param);
    }

    pub fn set_turn_limit(&mut self, limit: u128) {
        self.turn_limit = Some(limit);
    }

    pub fn set_complement_display(&mut self, display_mode: usize) {
        self.complement_system_display = display_mode;
    }
}
