use crate::game::{self, ObjectStore, State};
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
        _state: &mut State,
        _objects: &mut ObjectStore,
        _menu: &mut Menu<MainMenuItem>,
        item: &MainMenuItem,
    ) -> game::RunState {
        match item {
            MainMenuItem::NewGame => game::RunState::NewGame,
            MainMenuItem::Resume => game::RunState::LoadGame,
            MainMenuItem::Quit => quit(),
        }
    }
}

pub fn new() -> Menu<MainMenuItem> {
    Menu::new(vec![
        (MainMenuItem::NewGame, "New Game".to_string()),
        (MainMenuItem::Resume, "Resume Last Game".to_string()),
        (MainMenuItem::Quit, "Quit".to_string()),
    ])
}

#[cfg(not(target_arch = "wasm32"))]
fn quit() -> game::RunState {
    std::process::exit(0)
}

#[cfg(target_arch = "wasm32")]
fn quit() -> game::RunState {
    game::RunState::MainMenu(new())
}
