use crate::game::game_objects::GameObjects;
use crate::game::game_state::GameState;
use crate::game::RunState;
use crate::ui::menu::{Menu, MenuItem};

#[derive(Copy, Clone, Debug)]
pub enum MainMenuItem {
    NewGame,
    Resume,
    // Controls,
    // Options,
    Quit,
}

impl MenuItem for MainMenuItem {
    fn process(
        _state: &mut GameState,
        _objects: &mut GameObjects,
        _menu: &mut Menu<MainMenuItem>,
        item: &MainMenuItem,
    ) -> RunState {
        match item {
            MainMenuItem::NewGame => RunState::NewGame,
            MainMenuItem::Resume => RunState::LoadGame,
            MainMenuItem::Quit => quit(),
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

#[cfg(not(target_arch = "wasm32"))]
fn quit() -> RunState {
    std::process::exit(0)
}

#[cfg(target_arch = "wasm32")]
fn quit() -> RunState {
    RunState::MainMenu()
}
