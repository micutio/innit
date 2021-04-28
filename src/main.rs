#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate pretty_env_logger;
extern crate rand;
extern crate rand_core;
extern crate rand_isaac;
extern crate rltk;
extern crate serde;
extern crate serde_json;

mod core;
mod entity;
mod game;
mod test;
mod ui;
mod util;

use crate::game::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::{core::game_env::GameEnv, game::Game};
use std::env;

// For game testing run with `RUST_LOG=innit=trace RUST_BACKTRACE=1 cargo run`.
// Check [https://nnethercote.github.io/perf-book/title-page.html] for optimisation strategies.

pub fn main() -> rltk::BError {
    println!(
        r#"
        _____             _ _   
        \_   \_ __  _ __ (_) |_ 
         / /\/ '_ \| '_ \| | __|
      /\/ /_ | | | | | | | | |_ 
      \____/ |_| |_|_| |_|_|\__|  

      2019 - 2021 Michael Wagner
    "#
    );

    // init logger
    pretty_env_logger::init();
    let mut env: GameEnv = GameEnv::new();

    // parse program arguments
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

    // TODO: Create game environment from presets and command line flags.

    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple(SCREEN_WIDTH, SCREEN_HEIGHT)
        .unwrap()
        .with_advanced_input(true)
        .with_font("fonts/rex_paint_10x10.png", 10, 10)
        .with_sparse_console(SCREEN_WIDTH, SCREEN_HEIGHT, "fonts/rex_paint_10x10.png")
        // .with_fancy_console(SCREEN_WIDTH, SCREEN_HEIGHT, "fonts/rex_paint_10x10.png") // menu
        // .with_font("fonts/rex_paint_14x14.png", 14, 14)
        // .with_fancy_console(SCREEN_WIDTH, SCREEN_HEIGHT, "fonts/rex_paint_14x14.png") // menu
        .with_title("Innit alpha v0.0.4")
        .with_vsync(false)
        .with_fps_cap(60.0)
        .with_automatic_console_resize(false)
        .build()?;

    context.set_active_font(1, false);
    rltk::main_loop(context, Game::new(env))
}
