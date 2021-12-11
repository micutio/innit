use crate::game::objects::ObjectStore;
use crate::game::State;
use crate::game::RunState;
use crate::ui::menu::main_menu::main_menu;
use crate::ui::menu::{Menu, MenuItem};

#[derive(Copy, Clone, Debug)]
pub enum GameOverMenuItem {
    Credits,
    ReturnToMain,
}

impl MenuItem for GameOverMenuItem {
    fn process(
        _state: &mut State,
        _objects: &mut ObjectStore,
        _menu: &mut Menu<GameOverMenuItem>,
        item: &GameOverMenuItem,
    ) -> RunState {
        match item {
            GameOverMenuItem::Credits => unimplemented!(),
            GameOverMenuItem::ReturnToMain => RunState::MainMenu(main_menu()),
        }
    }
}

pub fn game_over_menu() -> Menu<GameOverMenuItem> {
    Menu::new(vec![
        (GameOverMenuItem::Credits, "Credits".to_string()),
        (
            GameOverMenuItem::ReturnToMain,
            "Return to Main Menu".to_string(),
        ),
    ])
}
