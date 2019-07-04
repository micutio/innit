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
use ui::game_frontend::{initialize_io, main_menu, GameIO};

fn launch_game() {
    let mut game_io: GameIO = initialize_io();
    main_menu(&mut game_io);
}

fn main() {
    launch_game();
}
