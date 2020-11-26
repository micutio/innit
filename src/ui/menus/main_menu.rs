use crate::game::{load_game, Game, RunState};
use crate::ui::menu::{Menu, MenuItem};
use rltk::Rltk;

#[derive(Copy, Clone)]
pub enum MainMenuItem {
    NewGame,
    Resume,
    // Controls,
    // Options,
    Quit,
}

impl MenuItem for MainMenuItem {
    fn process(
        game: &mut Game,
        ctx: &mut Rltk,
        menu: &mut Menu<MainMenuItem>,
        item: &MainMenuItem,
    ) -> RunState {
        match item {
            MainMenuItem::NewGame => {
                // start new game
                let (mut state, mut objects) = Game::new_game(game.state.env, ctx);
                game.reset(state, objects);
                RunState::Ticking
                // game_loop(&mut state, frontend, &mut input, &mut objects);
            }
            MainMenuItem::Resume => {
                // load game from file
                match load_game() {
                    Ok((mut state, mut objects)) => {
                        game.reset(state, objects);
                        RunState::Ticking
                    }
                    Err(_e) => {
                        // TODO: Show alert to user... or not?
                        // msg_box(frontend, &mut None, "", "\nNo saved game to load\n", 24);
                        RunState::MainMenu(menu.clone())
                    }
                }
            }
            MainMenuItem::Quit => {
                std::process::exit(0);
            }
        }
    }
}

pub fn main_menu() -> Menu<MainMenuItem> {
    Menu::new(vec![
        (MainMenuItem::NewGame, "New Game"),
        (MainMenuItem::Resume, "Resume Last Game"),
        (MainMenuItem::Quit, "Quit"),
    ])
}
