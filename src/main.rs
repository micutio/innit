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
mod world;

use gui::{main_menu, Tcod, LIMIT_FPS, PANEL_HEIGHT, SCREEN_HEIGHT, SCREEN_WIDTH};
use world::{WORLD_HEIGHT, WORLD_WIDTH};

use tcod::console::*;
use tcod::map::Map as FovMap;

fn main() {
    let root = Root::initializer()
        .font("assets/terminal16x16_gs_ro.png", FontLayout::AsciiInRow)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("roguelike")
        .init();

    tcod::system::set_fps(LIMIT_FPS);

    let mut tcod = Tcod {
        root: root,
        con: Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT),
        panel: Offscreen::new(SCREEN_WIDTH, PANEL_HEIGHT),
        fov: FovMap::new(WORLD_WIDTH, WORLD_HEIGHT),
        mouse: Default::default(),
    };

    main_menu(&mut tcod);
}
