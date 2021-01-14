use crate::core::game_objects::GameObjects;
use crate::core::game_state::GameState;
use crate::game::RunState;
use crate::ui::menus::{Menu, MenuItem};

#[derive(Copy, Clone, Debug)]
pub enum GameOverMenuItem {
    Credits,
    ReturnToMain,
}

impl MenuItem for GameOverMenuItem {
    fn process(
        _state: &mut GameState,
        _objects: &mut GameObjects,
        _menu: &mut Menu<GameOverMenuItem>,
        item: &GameOverMenuItem,
    ) -> RunState {
        match item {
            GameOverMenuItem::Credits => unimplemented!(),
            GameOverMenuItem::ReturnToMain => unimplemented!(),
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
