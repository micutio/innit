/// Module Main
///
/// This module contains all structures and methods pertaining to the user interface.
extern crate rand;
extern crate serde;
extern crate tcod;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod entity;
mod game_state;
mod ui;
mod util;
mod world;

// internal modules
use ui::game_frontend::{main_menu, GameFrontend};

fn launch_game() {
    let mut game_frontend: GameFrontend = GameFrontend::new();
    main_menu(&mut game_frontend);
}

fn main() {
    launch_game();
}
