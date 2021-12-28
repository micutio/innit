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

mod entity;
mod game;
mod raws;
mod test;
mod ui;
mod util;
mod world_gen;

use std::env;

rltk::embedded_resource!(FONT_8X8_CHEEPICUS, "../resources/fonts/Cheepicus_8x8.png");
rltk::embedded_resource!(
    FONT_12X12_CHEEPICUS,
    "../resources/fonts/Cheepicus_12x12.png"
);
rltk::embedded_resource!(
    FONT_14X14_CHEEPICUS,
    "../resources/fonts/Cheepicus_14x14.png"
);
rltk::embedded_resource!(
    FONT_16X16_CHEEPICUS,
    "../resources/fonts/Cheepicus_16x16.png"
);
rltk::embedded_resource!(FONT_8X8_REX_PAINT, "../resources/fonts/rex_paint_8x8.png");
rltk::embedded_resource!(
    FONT_12X12_REX_PAINT,
    "../resources/fonts/rex_paint_12x12.png"
);
rltk::embedded_resource!(
    FONT_14X14_REX_PAINT,
    "../resources/fonts/rex_paint_14x14.png"
);
rltk::embedded_resource!(
    FONT_16X16_REX_PAINT,
    "../resources/fonts/rex_paint_16x16.png"
);

// For game testing run with `RUST_LOG=innit=trace RUST_BACKTRACE=1 cargo run`.
// Check [https://nnethercote.github.io/perf-book/title-page.html] for optimisation strategies.
// Check [https://bfnightly.bracketproductions.com/rustbook/webbuild.html] for building as WASM.

const VERSION: &str = "0.04";

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
    for i in 0..args.len() {
        if let Some(arg) = args.get(i) {
            if arg.eq("-d") || arg.eq("--debug") {
                game::env().set_debug_mode(true);
            }
            if arg.eq("-s") || arg.eq("--seed") {
                // try get next argument to retrieve the seed number
                if i + 1 == args.len() {
                    info!("no seed parameter provided, fall back to use '0' instead");
                }
                if let Some(next_arg) = args.get(i + 1) {
                    let seed = match next_arg.parse::<u64>() {
                        Ok(n) => n,
                        Err(_) => {
                            info!("no numerical seed parameter provided, fall back to use '0' instead");
                            0
                        }
                    };
                    game::env().set_seed(seed);
                }
            }
            if arg.eq("-t") || arg.eq("--turns") {
                // try get next argument to retrieve the seed number
                if i + 1 == args.len() {
                    info!("Option '-t | --turns' requires an integer parameter!");
                }
                if let Some(next_arg) = args.get(i + 1) {
                    let turn_limit = match next_arg.parse::<u128>() {
                        Ok(n) => n,
                        Err(_) => {
                            info!("no numerical seed parameter provided, fall back to use '0' instead");
                            0
                        }
                    };
                    game::env().set_turn_limit(turn_limit);
                }
            }
            if arg.eq("--spectate") {
                game::env().set_spectating(true);
            }
            if arg.eq("-np") || arg.eq("--no-particles") {
                game::env().set_disable_particles(true);
            }
        }
    }

    // build engine and launch the game

    rltk::link_resource!(FONT_8X8_CHEEPICUS, "resources/fonts/Cheepicus_8x8.png");
    rltk::link_resource!(FONT_12X12_CHEEPICUS, "resources/fonts/Cheepicus_12x12.png");
    rltk::link_resource!(FONT_14X14_CHEEPICUS, "resources/fonts/Cheepicus_14x14.png");
    rltk::link_resource!(FONT_16X16_CHEEPICUS, "resources/fonts/Cheepicus_16x16.png");
    rltk::link_resource!(FONT_8X8_REX_PAINT, "resources/fonts/rex_paint_8x8.png");
    rltk::link_resource!(FONT_12X12_REX_PAINT, "resources/fonts/rex_paint_12x12.png");
    rltk::link_resource!(FONT_14X14_REX_PAINT, "resources/fonts/rex_paint_14x14.png");
    rltk::link_resource!(FONT_16X16_REX_PAINT, "resources/fonts/rex_paint_16x16.png");
    let font_8x8_cheepicus = "fonts/Cheepicus_8x8.png";
    let font_12x12_cheepicus = "fonts/Cheepicus_12x12.png";
    let font_14x14_cheepicus = "fonts/Cheepicus_14x14.png";
    let font_16x16_cheepicus = "fonts/Cheepicus_16x16.png";
    let font_8x8_rex_paint = "fonts/rex_paint_8x8.png";
    let font_12x12_rex_paint = "fonts/rex_paint_12x12.png";
    let font_14x14_rex_paint = "fonts/rex_paint_14x14.png";
    let font_16x16_rex_paint = "fonts/rex_paint_16x16.png";
    let context = rltk::RltkBuilder::new()
        .with_dimensions(game::consts::SCREEN_WIDTH, game::consts::SCREEN_HEIGHT)
        .with_font(font_8x8_rex_paint, 8, 8)
        .with_font(font_12x12_rex_paint, 12, 12)
        .with_font(font_14x14_rex_paint, 14, 14)
        .with_font(font_16x16_rex_paint, 16, 16)
        .with_font(font_8x8_cheepicus, 8, 8)
        .with_font(font_12x12_cheepicus, 12, 12)
        .with_font(font_14x14_cheepicus, 14, 14)
        .with_font(font_16x16_cheepicus, 16, 16)
        .with_advanced_input(true)
        .with_fancy_console(
            game::consts::SCREEN_WIDTH,
            game::consts::SCREEN_HEIGHT,
            font_8x8_rex_paint,
        ) // world layer
        .with_fancy_console(
            game::consts::SCREEN_WIDTH,
            game::consts::SCREEN_HEIGHT,
            font_8x8_rex_paint,
        ) // hud layer
        .with_fancy_console(
            game::consts::SCREEN_WIDTH,
            game::consts::SCREEN_HEIGHT,
            font_8x8_rex_paint,
        ) // particles layer
        .with_title(format!("Innit alpha v{}", VERSION))
        .with_fps_cap(60.0)
        // .with_automatic_console_resize(true)
        .with_vsync(true)
        .build()?;

    rltk::main_loop(context, game::Game::new())
}
