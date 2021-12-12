use crate::game::{self, ObjectStore, State};
use crate::ui::menu::{Menu, MenuItem};

#[derive(Clone, Copy, Debug)]
pub enum ActionCategory {
    Primary,
    Secondary,
    Quick1,
    Quick2,
}

#[derive(Clone, Debug)]
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
        state: &mut State,
        objects: &mut ObjectStore,
        _menu: &mut Menu<ActionItem>,
        item: &ActionItem,
    ) -> game::RunState {
        if let Some(ref mut object) = objects[state.player_idx] {
            let action_opt = object.match_action(&item.id);

            if let Some(action) = action_opt {
                match item.category {
                    ActionCategory::Primary => object.set_primary_action(action.clone_action()),
                    ActionCategory::Secondary => object.set_secondary_action(action.clone_action()),
                    ActionCategory::Quick1 => object.set_quick1_action(action.clone_action()),
                    ActionCategory::Quick2 => object.set_quick2_action(action.clone_action()),
                }
            }
        }

        game::RunState::Ticking
    }
}

pub fn new(available_actions: Vec<String>, category: ActionCategory) -> Menu<ActionItem> {
    let items: Vec<(ActionItem, String)> = available_actions
        .iter()
        .cloned()
        .map(|s| (ActionItem::new(s.clone(), category), s))
        .collect();
    Menu::new(items)
}
