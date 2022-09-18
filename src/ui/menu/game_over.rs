use crate::game::{self, ObjectStore, State};
use crate::ui::menu::{self, Menu, MenuItem};

#[derive(Copy, Clone, Debug)]
pub enum GameOverMenuItem {
    Credits,
    ReturnToMain,
}

impl MenuItem for GameOverMenuItem {
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

pub fn new() -> Menu<GameOverMenuItem> {
    Menu::with_header(
        "Game Over",
        &[
            (GameOverMenuItem::Credits, "Credits".to_string()),
            (
                GameOverMenuItem::ReturnToMain,
                "Return to Main Menu".to_string(),
            ),
        ],
    )
}
