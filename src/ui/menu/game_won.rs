use crate::game::{self, ObjectStore, State};
use crate::ui::menu::{self, Menu, MenuItem};

#[derive(Copy, Clone, Debug)]
pub enum GameWonMenuItem {
    Credits,
    ReturnToMain,
}

impl MenuItem for GameWonMenuItem {
    fn process(
        _state: &mut State,
        _objects: &mut ObjectStore,
        _menu: &mut Menu<GameWonMenuItem>,
        item: &GameWonMenuItem,
    ) -> game::RunState {
        match item {
            GameWonMenuItem::Credits => unimplemented!(),
            GameWonMenuItem::ReturnToMain => game::RunState::MainMenu(menu::main::new()),
        }
    }
}

pub fn new() -> Menu<GameWonMenuItem> {
    Menu::new(vec![
        (GameWonMenuItem::Credits, "Credits".to_string()),
        (
            GameWonMenuItem::ReturnToMain,
            "Return to Main Menu".to_string(),
        ),
    ])
}
