use crate::core::game_objects::GameObjects;
use crate::core::game_state::GameState;
use crate::game::RunState;
use crate::ui::menus::{Menu, MenuItem};

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
            MainMenuItem::Quit => std::process::exit(0),
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
