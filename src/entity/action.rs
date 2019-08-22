//! Module Action provides the action interface, which is used to create any kind of action that
//! can be performed by the player or an NPC.

use std::fmt::Debug;

use tcod::colors;

use crate::core::game_objects::GameObjects;
use crate::core::game_state::{GameState, MessageLog, ObjectProcResult, TORCH_RADIUS};
use crate::entity::object::Object;
use crate::ui::player::PLAYER;

/// Result of performing an action.
/// It can succeed, fail and cause direct consequences.
pub enum ActionResult {
    /// Sucessfully finished action
    Success { callback: ObjectProcResult },
    /// Failed to perform an action, ideally without any side effect.
    Failure,
    /// Another action happens automatically after this one.
    Consequence { action: Option<Box<dyn Action>> },
}

struct ActionObject<T: Action + Copy> {
    action: T,
}

impl<T> ActionObject<T>
where
    T: Action + Copy,
{
    fn instantiate(&self) -> ActionObject<T> {
        ActionObject {
            action: self.action,
        }
    }
}

/// Prototype for all actions.
/// They need to be `performable` and have a cost (even if it's 0).
#[typetag::serde(tag = "type")]
pub trait Action: Debug {
    fn perform(
        &self,
        game_state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
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
        game_state: &mut GameState,
        game_objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult {
        // do nothing
        // duh
        // TODO: make sure all game log messages are only displayed if the cause is visible to the
        // player
        if let Some(player) = &game_objects[PLAYER] {
            if player.distance_to(&owner) <= TORCH_RADIUS as f32 && owner.tile.is_none() {
                // don't record all tiles passing constantly
                game_state.log.add(
                    format!("{} passes their turn", owner.visual.name),
                    colors::WHITE,
                );
            }
        }
        ActionResult::Success {
            callback: ObjectProcResult::NoFeedback,
        }
    }

    fn get_energy_cost(&self) -> i32 {
        0 // being lazy is easy
    }
}

/// Attack another object.
#[derive(Debug, Serialize, Deserialize)]
pub struct AttackAction {
    base_power:  i32,
    target_id:   Option<usize>,
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
        game_state: &mut GameState,
        objects: &mut GameObjects,
        _owner: &mut Object,
    ) -> ActionResult {
        match self.target_id {
            Some(target_id) => {
                // TODO: Replace with defend action.
                if let Some(ref mut target) = objects[target_id] {
                    // target.take_damage(self.base_power, game_state);
                    return ActionResult::Success {
                        callback: ObjectProcResult::CheckEnterFOV,
                    };
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

/// Move an object
/// TODO: Maybe create enum target {self, other{object_id}} to use for any kind of targetable
/// action.
#[derive(Debug, Serialize, Deserialize)]
pub struct MoveAction {
    direction:   Direction,
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
        _game_state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult {
        use self::Direction::*;
        let (dx, dy) = match self.direction {
            North => (0, -1),
            South => (0, 1),
            East => (1, 0),
            West => (-1, 0),
        };

        let (x, y) = owner.pos();
        if !&objects.is_blocked(x + dx, y + dy) {
            info!(
                "move {} from ({},{}) to ({},{})",
                owner.visual.name,
                x,
                y,
                x + dx,
                y + dy
            );
            owner.set_pos(x + dx, y + dy);
            ActionResult::Success {
                callback: ObjectProcResult::CheckEnterFOV,
            }
        } else {
            ActionResult::Failure
        }
    }

    fn get_energy_cost(&self) -> i32 {
        self.energy_cost
    }
}
