//! Module Main
//!
//! This module contains all structures and methods pertaining to the user interface.

extern crate rand;
extern crate serde;
extern crate tcod;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod ai;
mod fighter;
mod game_state;
mod gui;
mod item;
mod object;
mod util;
mod world;

// internal modules
use gui::launch_game;

fn main() {
    launch_game();
}
