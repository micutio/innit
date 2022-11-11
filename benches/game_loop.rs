use bracket_lib::prelude as rltk;
use criterion::{criterion_group, criterion_main, Criterion};
use innit::create_rltk_terminal;
use innit::game;

pub fn game_loop_benchmark(c: &mut Criterion) {
    // init logger
    pretty_env_logger::init();

    // setup game rules
    game::env().set_seed(0);
    game::env().set_spectating(true);
    game::env().set_turn_limit(500);

    c.bench_function("gol", |b| {
        b.iter(|| {
            let context = match create_rltk_terminal(innit::VERSION) {
                Ok(it) => it,
                Err(err) => panic!("{}", err),
            };
            if let Err(err) = rltk::main_loop(context, game::Game::new()) {
                panic!("{}", err);
            }
        })
    });
}

criterion_group!(benches, game_loop_benchmark);
criterion_main!(benches);
