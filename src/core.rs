pub mod game_env;
pub mod game_objects;
pub mod game_state;
pub mod position;
pub mod world;

use std::sync::{Mutex, MutexGuard};

use crate::core::game_env::GameEnv;

lazy_static! {
    static ref GAME_ENV: Mutex<GameEnv> = Mutex::new(GameEnv::new());
}

pub fn set_debug_mode(is_debug: bool) {
    let mut env: MutexGuard<GameEnv> = GAME_ENV.lock().unwrap();
    env.set_debug_mode(is_debug);
}

pub fn innit_env<'a>() -> MutexGuard<'a, GameEnv> {
    GAME_ENV.lock().unwrap()
}
