use std::fmt::Debug;
/// Module Action
///
/// This module provides the action interface, which is used to create any kind
/// of action that can be performed by the player or an NPC.
// external imports
use tcod::colors;

// internal imports
use entity::object::{Object, ObjectVec};
use game_state::{GameState, MessageLog};
use world::is_blocked;

/// Result of performing an action.
/// It can succeed, fail and cause direct consequences.
/// TODO: (!) Include UI feedback e.g., animation, FOV update and rendering!
pub enum ActionResult {
    /// Sucessfully finished action
    Success,
    /// Failed to perform an action, ideally without any side effect.
    Failure,
    /// Another action happens automatically after this one.
    Consequence { action: Option<Box<dyn Action>> },
}

/// Prototype for all actions.
/// They need to be `performable` and have a cost (even if it's 0).
#[typetag::serde(tag = "type")]
pub trait Action: Debug {
    fn perform(
        &self,
        owner: &mut Object,
        objects: &mut ObjectVec,
        game_state: &mut GameState,
    ) -> ActionResult;

    fn get_energy_cost(&self) -> i32;
}

/// Dummy action for passing the turn.
#[derive(Debug, Serialize, Deserialize)]
pub struct PassAction;

#[typetag::serde]
impl Action for PassAction {
    fn perform(
        &self,
        owner: &mut Object,
        _objects: &mut ObjectVec,
        game_state: &mut GameState,
    ) -> ActionResult {
        // do nothing
        // duh
        game_state
            .log
            .add(format!("{} passes their turn", owner.name), colors::WHITE);
        ActionResult::Success
    }

    fn get_energy_cost(&self) -> i32 {
        0 // being lazy is easy
    }
}

/// Attack another object.
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
    fn perform(
        &self,
        _owner: &mut Object,
        objects: &mut ObjectVec,
        game_state: &mut GameState,
    ) -> ActionResult {
        match self.target_id {
            Some(target_id) => {
                // TODO: Replace with defend action.
                // unwrap should be safe to use here because the object not available
                // in `objects` is the owner of this action.
                if let Some(ref mut target) = objects[target_id] {
                    target.take_damage(self.base_power, game_state);
                    return ActionResult::Success;
                }
                ActionResult::Failure
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
    North,
    South,
    East,
    West,
}

/// Move an object
/// TODO: Maybe create enum target {self, other{object_id}} to use for any kind of targetable action.
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
    fn perform(
        &self,
        owner: &mut Object,
        objects: &mut ObjectVec,
        game_state: &mut GameState,
    ) -> ActionResult {
        use self::Direction::*;
        let (dx, dy) = match self.direction {
            North => (0, -1),
            South => (0, 1),
            East => (1, 0),
            West => (-1, 0),
        };

        let (x, y) = owner.pos();
        if !is_blocked(&game_state.world, &objects, x + dx, y + dy) {
            println!(
                "move {} from {},{} to {},{}",
                owner.name,
                x,
                y,
                x + dx,
                y + dy
            );
            owner.set_pos(x + dy, y + dy);
            // TODO: Check whether we walked into the player's field of view.
            ActionResult::Success
        } else {
            ActionResult::Failure
        }
    }

    fn get_energy_cost(&self) -> i32 {
        self.energy_cost
    }
}
