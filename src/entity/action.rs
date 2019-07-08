/// Module Action
///
/// This module provides the action interface, which is used to create any kind
/// of action that can be performed by the player or an NPC.
/// TODO: Add AI actions to control their turn taking.
use std::rc::Rc;
use std::fmt::Debug;

use entity::object::{ObjectVec, Object};
use game_state::GameState;

pub enum ActionResult {
    /// Sucessfully finished action
    Success,
    /// Failed to perform an action, ideally without any side effect.
    Failure,
    /// Another action happens automatically after this one.
    Consequence {
        action: Option<Rc<dyn Action>>,
    },
    // Another action happens as the same time as this one.
    SideEffect {
        action: Option<Rc<dyn Action>>,
    },
}

#[typetag::serde(tag = "type")]
pub trait Action: Debug {
    fn perform(&self, owner: &mut Object, objects: &mut ObjectVec, game_state: &mut GameState) -> ActionResult;

    fn get_energy_cost(&self) -> i32;
}

// Example action
#[derive(Debug, Serialize, Deserialize)]
pub struct AttackAction {
    base_power: i32,
    target_id: Option<usize>,
    energy_cost: i32,
}

impl AttackAction {
    pub fn new(base_power: i32, energy_cost: i32) -> Self {
        AttackAction {
            base_power,
            target_id: None,
            energy_cost,
        }
    }

    pub fn acquire_target(&mut self, target_id: usize) {
        self.target_id = Some(target_id);
    }
}

#[typetag::serde]
impl Action for AttackAction {
    fn perform(&self, owner: &mut Object, objects: &mut ObjectVec, game_state: &mut GameState) -> ActionResult {
        match self.target_id {
            Some(target_id) => {
                // TODO: Replace with defend action.
                // unwrap should be safe to use here because the object not available
                // in `objects` is the owner of this action.
                objects[target_id].unwrap().take_damage(self.base_power, game_state);
                ActionResult::Success
            }
            None => ActionResult::Failure,
        }
    }

    fn get_energy_cost(&self) -> i32 {
        self.energy_cost
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Direction {
    North, South, East, West,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoveAction {
    direction: Direction,
    energy_cost: i32,
}

impl MoveAction {
    pub fn new(direction: Direction, energy_cost: i32) -> Self {
        MoveAction {
            direction,
            energy_cost,
        }
    }
}

#[typetag::serde]
impl Action for MoveAction {
    fn perform(&self, owner: &mut Object, objects: &mut ObjectVec, game_state: &mut GameState) {
        let (dx, dy) = match self.direction {
            North => (0, -1),
            South => (0, 1),
            East => (1, 0),
            West => (-1, 0),
        };

        let (x, y) = owner.pos();
        if !is_blocked(game_state.world, objects, x + dx, y + dy) {
            owner.set_pos(x + dy, y + dy);
        }
    }

    fn get_energy_cost(&self) -> i32 {
        self.energy_cost
    }
}