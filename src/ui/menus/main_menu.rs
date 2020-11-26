use crate::game::{load_game, Game, RunState};
use crate::ui::menus::{Menu, MenuItem};

#[derive(Copy, Clone)]
pub enum MainMenuItem {
    NewGame,
    Resume,
    // Controls,
    // Options,
    Quit,
}

impl MenuItem for MainMenuItem {
    fn process(game: &mut Game, menu: &mut Menu<MainMenuItem>, item: &MainMenuItem) -> RunState {
        match item {
            MainMenuItem::NewGame => {
                // start new game
                let (state, objects) = Game::new_game(game.state.env);
                game.reset(state, objects);
                RunState::Ticking
                // game_loop(&mut state, frontend, &mut input, &mut objects);
            }
            MainMenuItem::Resume => {
                // load game from file
                match load_game() {
                    Ok((state, objects)) => {
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
        (MainMenuItem::NewGame, "New Game".to_string()),
        (MainMenuItem::Resume, "Resume Last Game".to_string()),
        (MainMenuItem::Quit, "Quit".to_string()),
    ])
}
