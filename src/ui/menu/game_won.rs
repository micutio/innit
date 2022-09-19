use crate::game::{self, ObjectStore, State};
use crate::ui::menu::{self, Item, Menu};

#[derive(Copy, Clone, Debug)]
pub enum MenuItem {
    Credits,
    ReturnToMain,
}

impl Item for MenuItem {
    fn process(
        _state: &mut State,
        _objects: &mut ObjectStore,
        _menu: &mut Menu<Self>,
        item: &Self,
    ) -> game::RunState {
        match item {
            Self::Credits => game::RunState::CreditsScreen(menu::credits::new()),
            Self::ReturnToMain => game::RunState::MainMenu(menu::main::new()),
        }
    }
}

pub fn new() -> Menu<MenuItem> {
    Menu::with_header(
        "SUCCESS - INFECTION REPELLED!",
        &[
            (MenuItem::Credits, "Credits".to_string()),
            (MenuItem::ReturnToMain, "Return to Main Menu".to_string()),
        ],
    )
}
