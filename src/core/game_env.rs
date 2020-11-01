use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct GameEnv {
    pub debug_mode: bool,
}

impl GameEnv {
    pub fn new() -> Self {
        GameEnv { debug_mode: false }
    }

    pub fn set_debug_mode(&mut self, debug_mode: bool) {
        self.debug_mode = debug_mode;
    }
}
