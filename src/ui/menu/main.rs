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
        _menu: &mut Menu<Self>,
        item: &Self,
    ) -> game::RunState {
        match item {
            Self::NewGame => game::RunState::NewGame,
            Self::Resume => game::RunState::LoadGame,
            Self::Quit => quit(),
        }
    }
}

pub fn new() -> Menu<MainMenuItem> {
    Menu::new(&[
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
