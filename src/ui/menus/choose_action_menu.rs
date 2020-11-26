use crate::game::{Game, RunState};
use crate::ui::menu::{Menu, MenuItem};
use rltk::Rltk;

#[derive(Clone)]
pub enum ActionCategory {
    Primary,
    Secondary,
    Quick1,
    Quick2,
}

#[derive(Clone)]
pub struct ActionItem {
    id: String,
    category: ActionCategory,
}

impl ActionItem {
    pub fn new(id: String, category: ActionCategory) -> Self {
        ActionItem { id, category }
    }
}

impl MenuItem for ActionItem {
    fn process(
        game: &mut Game,
        _ctx: &mut Rltk,
        _menu: &mut Menu<ActionItem>,
        item: &ActionItem,
    ) -> RunState {
        if let Some(ref mut object) = game.objects[game.state.current_player_index] {
            if let Some(a) = object
                .get_all_actions()
                .iter()
                .find(|a| a.get_identifier().eq(&item.id))
            {
                match item.category {
                    ActionCategory::Primary => object.set_primary_action(a.clone_action()),
                    ActionCategory::Secondary => object.set_secondary_action(a.clone_action()),
                    ActionCategory::Quick1 => object.set_quick1_action(a.clone_action()),
                    ActionCategory::Quick2 => object.set_quick2_action(a.clone_action()),
                }
            }
        }

        RunState::Ticking
    }
}

pub fn choose_action_menu(
    available_actions: Vec<String>,
    category: ActionCategory,
) -> Menu<ActionItem> {
    let items: Vec<(ActionItem, &str)> = available_actions
        .iter()
        .cloned()
        .map(|s| (ActionItem::new(s, category), s.as_str()))
        .collect();
    Menu::new(items)
}
