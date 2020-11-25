use crate::game::{load_game, Game, RunState};
use crate::ui::menu::{Menu, MenuInstance};
use rltk::Rltk;

#[derive(Copy, Clone)]
pub enum MainMenuOption {
    NewGame,
    Resume,
    // Controls,
    // Options,
    Quit,
}

impl MainMenuOption {
    pub fn process(
        game: &mut Game,
        ctx: &mut Rltk,
        menu: Menu<MainMenuOption>,
        item: &MainMenuOption,
    ) -> RunState {
        match item {
            MainMenuOption::NewGame => {
                // start new game
                let (mut state, mut objects) = Game::new_game(game.state.env, ctx);
                game.reset(state, objects);
                RunState::Ticking
                // game_loop(&mut state, frontend, &mut input, &mut objects);
            }
            MainMenuOption::Resume => {
                // load game from file
                match load_game() {
                    Ok((mut state, mut objects)) => {
                        game.reset(state, objects);
                        RunState::Ticking
                    }
                    Err(_e) => {
                        // TODO: Show alert to user... or not?
                        // msg_box(frontend, &mut None, "", "\nNo saved game to load\n", 24);
                        RunState::Menu(MenuInstance::MainMenu(menu))
                    }
                }
            }
            MainMenuOption::Quit => {
                std::process::exit(0);
            }
        }
    }
}

pub fn main_menu() -> Menu<MainMenuOption> {
    Menu::new(vec![
        (MainMenuOption::NewGame, "New Game"),
        (MainMenuOption::Resume, "Resume Last Game"),
        (MainMenuOption::Quit, "Quit"),
    ])
}
