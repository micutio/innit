use crate::game::{self, ObjectStore, State};
use crate::ui::menu::{Item, Menu};

#[derive(Copy, Clone, Debug)]
pub enum MenuItem {
    NewGame,
    Resume,
    // Controls,
    // Options,
    Quit,
}

impl Item for MenuItem {
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

pub fn new() -> Menu<MenuItem> {
    Menu::new(&[
        (MenuItem::NewGame, "New Game".to_string()),
        (MenuItem::Resume, "Resume Last Game".to_string()),
        (MenuItem::Quit, "Quit".to_string()),
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
