#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate rand;
extern crate rand_core;
extern crate rand_isaac;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tcod;

mod core;
mod entity;
mod game;
mod test;
mod ui;
mod util;

use std::env;

use crate::core::game_env::GameEnv;
use crate::ui::game_frontend::{main_menu, GameFrontend};

pub fn launch_game(env: GameEnv) {
    main_menu(env, &mut GameFrontend::new());
}

/// For game testing run with
/// (bash) `RUST_LOG=innit=trace RUST_BACKTRACE=1 cargo run`
/// (fish) `env RUST_LOG=innit=trace RUST_BACKTRACE=1 cargo run`
pub fn main() {
    pretty_env_logger::init();
    let mut env: GameEnv = GameEnv::new();

    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);

    for arg in args {
        if arg.eq("-d") || arg.eq("--debug") {
            env.set_debug_mode(true);
        }
        if arg.eq("-s") || arg.eq("--seeding") {
            env.set_rng_seeding(true);
        }
    }

    // TODO: Create game environment from presets and command line flags!

    launch_game(env);
}
