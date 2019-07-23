/// Module Main
///
/// This module contains all structures and methods pertaining to the user interface.

#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tcod;

mod core;
mod entity;
mod game;
mod ui;
mod util;

// internal modules
use ui::game_frontend::{main_menu, GameFrontend};

fn launch_game() {
    let mut game_frontend: GameFrontend = GameFrontend::new();
    main_menu(&mut game_frontend);
}

fn main() {
    pretty_env_logger::init();
    // RUST_LOG=innit=trace cargo run
    launch_game();
}
