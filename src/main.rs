#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate rand;
extern crate rand_core;
extern crate rand_isaac;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate rltk;
extern crate serde_json;

mod core;
mod entity;
mod game;
mod test;
mod ui;
mod util;

use crate::core::game_env::GameEnv;
use rltk::{GameState, Rltk};
use std::env;

struct State {}
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        ctx.print(1, 1, "Hello Rust World");
    }
}

/// For game testing run with
/// (bash) `RUST_LOG=innit=trace RUST_BACKTRACE=1 cargo run`
/// (fish) `env RUST_LOG=innit=trace RUST_BACKTRACE=1 cargo run`
pub fn main() -> rltk::BError {
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

    let context = rltk::RltkBuilder::simple80x50()
        .with_title("Innit alpha v0.0.2")
        .with_fps_cap(30.0)
        .build()?;
    let gs = State {};
    rltk::main_loop(context, gs)
}
