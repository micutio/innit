//! # Innit - An immune system roguelike
//!
//! Started following the [libtcod tutorial](https://tomassedovic.github.io/roguelike-tutorial),
//! later ported to and heavily influenced by the [RLTK tutorial](https://bfnightly.bracketproductions.com/rustbook/)
//!
//! Michael Wagner 2018
//!

#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(
    clippy::multiple_crate_versions,
    clippy::similar_names,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::use_self
)]

extern crate casim;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate bracket_lib;
extern crate pretty_env_logger;
extern crate rand;
extern crate rand_core;
extern crate rand_isaac;
extern crate serde;
extern crate serde_json;

mod entity;
mod game;
mod raws;
mod test;
mod ui;
mod util;
mod world_gen;

bracket_lib::prelude::add_wasm_support!();
use bracket_lib::prelude as rltk;
use bracket_lib::terminal::EMBED;
use color_eyre::eyre::Result;
use std::env;

rltk::embedded_resource!(FONT_16X16_YUN, "../resources/fonts/yun_16x16.png");
rltk::embedded_resource!(FONT_16X16_REX, "../resources/fonts/rex_16x16.png");
rltk::embedded_resource!(FONT_14X14_REX, "../resources/fonts/rex_14x14.png");
rltk::embedded_resource!(FONT_12X12_REX, "../resources/fonts/rex_12x12.png");
rltk::embedded_resource!(FONT_8X8_REX, "../resources/fonts/rex_8x8.png");

// For game testing run with `RUST_LOG=innit=trace RUST_BACKTRACE=1 cargo run`.
// Check [https://nnethercote.github.io/perf-book/title-page.html] for optimisation strategies.
// Check [https://bfnightly.bracketproductions.com/rustbook/webbuild.html] for building as WASM.

const VERSION: &str = "0.0.6";

/// # Panics
/// If the setup fails.
/// # Errors
/// Errors are repackaged with `color_eyre`
pub fn main() -> Result<()> {
    color_eyre::install()?;

    println!(
        r#"
        _____             _ _   
        \_   \_ __  _ __ (_) |_ 
         / /\/ '_ \| '_ \| | __|
      /\/ /_ | | | | | | | | |_ 
      \____/ |_| |_|_| |_|_|\__|  

      2019 - 2022 Michael Wagner
    "#
    );

    // init logger
    pretty_env_logger::init();

    // parse program arguments
    parse_cmdline_flags();

    // build engine and launch the game

    rltk::link_resource!(FONT_16X16_YUN, "resources/fonts/yun_16x16.png");
    rltk::link_resource!(FONT_16X16_REX, "resources/fonts/rex_16x16.png");
    rltk::link_resource!(FONT_14X14_REX, "resources/fonts/rex_14x14.png");
    rltk::link_resource!(FONT_12X12_REX, "resources/fonts/rex_12x12.png");
    rltk::link_resource!(FONT_8X8_REX, "resources/fonts/rex_8x8.png");
    let font_16x16_yun = "fonts/yun_16x16.png";
    let font_16x16_rex = "fonts/rex_16x16.png";
    let font_14x14_rex = "fonts/rex_14x14.png";
    let font_12x12_rex = "fonts/rex_12x12.png";
    let font_8x8_rex = "fonts/rex_8x8.png";

    let tile_size = game::env().tile_size;
    let context = match rltk::BTermBuilder::new()
        .with_dimensions(game::consts::SCREEN_WIDTH, game::consts::SCREEN_HEIGHT)
        .with_font(font_16x16_yun, 16, 16)
        .with_font(font_16x16_rex, 16, 16)
        .with_font(font_14x14_rex, 14, 14)
        .with_font(font_12x12_rex, 12, 12)
        .with_font(font_8x8_rex, 8, 8)
        .with_advanced_input(true)
        .with_fancy_console(
            // world layer
            game::consts::SCREEN_WIDTH,
            game::consts::SCREEN_HEIGHT,
            font_16x16_yun,
        )
        .with_fancy_console(
            // particles layer
            game::consts::SCREEN_WIDTH,
            game::consts::SCREEN_HEIGHT,
            font_16x16_yun,
        )
        .with_fancy_console(
            // shader layer
            game::consts::SCREEN_WIDTH,
            game::consts::SCREEN_HEIGHT,
            font_16x16_yun,
        )
        .with_fancy_console(
            game::consts::SCREEN_WIDTH,
            game::consts::SCREEN_HEIGHT,
            font_16x16_yun,
        ) // hud layer
        .with_title(format!("Innit alpha v{}", VERSION))
        .with_fps_cap(60.0)
        .with_tile_dimensions(tile_size, tile_size)
        // .with_vsync(true)
        .build()
    {
        Ok(it) => it,
        Err(err) => panic!("{}", err),
    };

    if let Err(err) = rltk::main_loop(context, game::Game::new()) {
        panic!("{}", err);
    } else {
        Ok(())
    }
}

fn parse_cmdline_flags() {
    let args: Vec<String> = env::args().collect();
    debug!("args: {:?}", args);
    for idx in 0..args.len() {
        if let Some(arg) = args.get(idx) {
            match arg.as_str() {
                "-d" | "--debug" => game::env().set_debug_mode(true),
                "-s" | "--seed" => parse_seed(&args, idx),
                "--tile-size" => parse_tile_size(&args, idx),
                "-t" | "--turns" => parse_turns(&args, idx),
                "--spectate" => game::env().set_spectating(true),
                "-np" | "--no-particles" => game::env().set_particles(false),
                "-g" | "--gfx" => game::env().set_disable_gfx(false),
                _ => {}
            }
        }
    }
}

fn parse_seed(args: &[String], idx: usize) {
    // try get next argument to retrieve the seed number
    if idx + 1 == args.len() {
        info!("no seed parameter provided, fall back to use '0' instead");
    }
    if let Some(next_arg) = args.get(idx + 1) {
        let seed = next_arg.parse().map_or_else(
            |_| {
                info!("no numerical seed parameter provided, fall back to use '0' instead");
                0
            },
            |v| v,
        );
        game::env().set_seed(seed);
    }
}

fn parse_tile_size(args: &[String], idx: usize) {
    // try get next argument to retrieve the tile size
    if idx + 1 == args.len() {
        info!("no tile size parameter provided, fall back to use '16' instead");
    }
    if let Some(next_arg) = args.get(idx + 1) {
        let t_size = next_arg.parse::<i32>().map_or_else(
            |_| {
                info!("no numerical tile size parameter provided, fall back to use '16' instead");
                16
            },
            |v| v,
        );
        game::env().set_tile_size(t_size);
    }
}

fn parse_turns(args: &[String], idx: usize) {
    // try get next argument to retrieve the seed number
    if idx + 1 == args.len() {
        info!("Option '-t | --turns' requires an integer parameter!");
    }
    if let Some(next_arg) = args.get(idx + 1) {
        let turn_limit = next_arg.parse::<u128>().map_or_else(
            |_| {
                info!("no numerical seed parameter provided, fall back to use '0' instead");
                0
            },
            |v| v,
        );
        game::env().set_turn_limit(turn_limit);
    }
}
