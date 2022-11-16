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

extern crate bracket_lib;
extern crate casim;
extern crate lazy_static;
extern crate log;
extern crate pretty_env_logger;
extern crate rand;
extern crate rand_core;
extern crate rand_isaac;
extern crate serde;
extern crate serde_json;

use bracket_lib::prelude as rltk;

// For game testing run with `RUST_LOG=innit=trace RUST_BACKTRACE=1 cargo run`.
// Check [https://nnethercote.github.io/perf-book/title-page.html] for optimisation strategies.
// Check [https://bfnightly.bracketproductions.com/rustbook/webbuild.html] for building as WASM.

/// # Panics
/// If the setup fails.
/// # Errors
/// Errors are repackaged with `color_eyre`
pub fn main() -> color_eyre::Result<()> {
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
    innit::parse_cmdline_flags();

    // build engine and launch the game
    let context = match innit::create_rltk_terminal(innit::VERSION) {
        Ok(it) => it,
        Err(err) => panic!("{}", err),
    };

    if let Err(err) = rltk::main_loop(context, innit::game::Game::new()) {
        panic!("{}", err);
    } else {
        Ok(())
    }
}
