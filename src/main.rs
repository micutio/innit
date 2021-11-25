//! # Innit - An immune system roguelike
//!
//! Started following the [libtcod tutorial](https://tomassedovic.github.io/roguelike-tutorial),
//! later ported to and heavily influenced by the [RLTK tutorial](https://bfnightly.bracketproductions.com/rustbook/)
//!
//! Michael Wagner 2018

extern crate casim;
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

use crate::core::innit_env;
use crate::game::Game;
use crate::game::{SCREEN_HEIGHT, SCREEN_WIDTH};
use rltk::RltkBuilder;
use std::env;

rltk::embedded_resource!(CONSOLE_FONT, "../resources/fonts/16x16-sm.png");

// For game testing run with `RUST_LOG=innit=trace RUST_BACKTRACE=1 cargo run`.
// Check [https://nnethercote.github.io/perf-book/title-page.html] for optimisation strategies.
// Check [https://bfnightly.bracketproductions.com/rustbook/webbuild.html] for building as WASM.

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

    // parse program arguments
    let args: Vec<String> = env::args().collect();
    debug!("args: {:?}", args);
    for arg in args {
        if arg.eq("-d") || arg.eq("--debug") {
            innit_env().set_debug_mode(true);
        }
        if arg.eq("-s") || arg.eq("--seeding") {
            innit_env().set_rng_seeding(true);
        }
        if arg.eq("--spectate") {
            innit_env().set_spectating(true);
        }
    }

    // build engine and launch the game

    rltk::link_resource!(CONSOLE_FONT, "resources/fonts/16x16-sm.png");
    let font = "fonts/16x16-sm.png";
    let context = RltkBuilder::new()
        .with_dimensions(SCREEN_WIDTH, SCREEN_HEIGHT)
        .with_font(font, 16, 16)
        .with_advanced_input(true)
        .with_fancy_console(SCREEN_WIDTH, SCREEN_HEIGHT, font) // world layer
        .with_fancy_console(SCREEN_WIDTH, SCREEN_HEIGHT, font) // hud layer
        .with_sparse_console(SCREEN_WIDTH, SCREEN_HEIGHT, font) // particles layer
        .with_title("Innit alpha v0.0.4")
        .with_fps_cap(60.0)
        // .with_automatic_console_resize(true)
        .with_vsync(true)
        .build()?;

    // context.set_active_font(0, true);
    rltk::main_loop(context, Game::new())
}
