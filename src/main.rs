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
mod ui;
mod util;

use ui::game_frontend::{main_menu, GameFrontend};

pub fn launch_game() {
    let mut game_frontend: GameFrontend = GameFrontend::new();
    main_menu(&mut game_frontend);
}

// For game testing run with `RUST_LOG=innit=trace RUST_BACKTRACE=1 cargo run`
pub fn main() {
    pretty_env_logger::init();
    launch_game();
}
