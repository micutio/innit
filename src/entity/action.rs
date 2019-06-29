/// Module Action
///
/// This module provides the action interface, which is used to create any kind
/// of action that can be performed by the player or an NPC.
use std::rc::Rc;

use entity::object::Object;
use game_state::GameState;

pub enum ActionResult {
    /// Sucessfully finished action
    Success,
    /// Failed to perform an action, ideally without any side effect.
    Failure,
    /// Another action happens automatically after this one.
    Consequence {
        action: Option<Rc<Action>>,
    },
    // Another action happens as the same time as this one.
    SideEffect {
        action: Option<Rc<Action>>,
    },
}

pub trait Action {
    fn perform(&self, objects: &mut [Object], game_state: &mut GameState) -> ActionResult;
}

// Example action
#[derive(Debug, Serialize, Deserialize)]
pub struct AttackAction {
    base_power: i32,
    target_id: Option<usize>,
}

impl AttackAction {
    pub fn new(base_power: i32) -> Self {
        AttackAction {
            base_power,
            target_id: None,
        }
    }

    pub fn acquire_target(&mut self, target_id: usize) {
        self.target_id = Some(target_id);
    }
}

impl Action for AttackAction {
    fn perform(&self, objects: &mut [Object], game_state: &mut GameState) -> ActionResult {
        match self.target_id {
            Some(target_id) => {
                // TODO: Replace with defend action.
                objects[target_id].take_damage(self.base_power, game_state);
                ActionResult::Success
            }
            None => ActionResult::Failure,
        }
    }
}
