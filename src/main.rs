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
mod raws;
mod test;
mod ui;
mod util;

use crate::game::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::raws::object_template::ObjectTemplate;
use crate::raws::spawn::Spawn;
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

    let spawn_str: String = serde_json::to_string(&Spawn::example()).unwrap();
    println!("{}", spawn_str);

    let obj_str: String = serde_json::to_string(&ObjectTemplate::example()).unwrap();
    println!("{}", obj_str);

    raws::load_raws();

    // build engine and launch the game
    use rltk::RltkBuilder;
    // let font = "fonts/rex_paint_10x10.png";
    let font = "fonts/rex_paint_8x8.png";
    let mut context = RltkBuilder::simple(SCREEN_WIDTH, SCREEN_HEIGHT)
        .unwrap()
        .with_advanced_input(true)
        .with_font(font, 8, 8)
        .with_sparse_console(SCREEN_WIDTH, SCREEN_HEIGHT, font) // hud layer
        .with_sparse_console(SCREEN_WIDTH, SCREEN_HEIGHT, font) // particles
        .with_title("Innit alpha v0.0.4")
        .with_vsync(false)
        .with_fps_cap(60.0)
        .with_automatic_console_resize(false)
        .build()?;

    context.set_active_font(1, false);
    rltk::main_loop(context, Game::new())
}
