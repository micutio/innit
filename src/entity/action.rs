//! Module Action provides the action interface, which is used to create any kind of action that
//! can be performed by the player or an NPC.
//! Any action is supposed to be assigned to one of the three trait families (sensing, prcessing,
//! actuating) of an object

// TODO: Create actions for setting and using quick/primary/secondary actions.

use std::fmt::Debug;

use tcod::colors;

use crate::core::game_objects::GameObjects;
use crate::core::game_state::{GameState, MessageLog, ObjectProcResult};
use crate::entity::action::Target::BlockingObject;
use crate::entity::object::Object;
use crate::player::PLAYER;

/// Targets can only be adjacent to the object: north, south, east, west or the objects itself.
pub enum Target {
    EmptyObject,
    BlockingObject,
    None,
    Any,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum Direction {
    North,
    South,
    East,
    West,
    Center,
}

impl Direction {
    fn to_xy(&self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
            Direction::Center => (0, 0),
        }
    }
}

/// Result of performing an action.
/// It can succeed, fail and cause direct consequences.
pub enum ActionResult {
    /// Successfully finished action
    Success { callback: ObjectProcResult },
    /// Failed to perform an action, ideally without any side effect.
    Failure,
    /// Another action happens automatically after this one.
    Consequence { action: Option<Box<dyn Action>> },
}

/// Prototype for all actions.
/// They need to be `performable` and have a cost (even if it's 0).
/// TODO: Add target here and remove action prototypes!
#[typetag::serde(tag = "type")]
pub trait Action: Debug {
    fn perform(
        &self,
        game_state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult;

    fn target(&self) -> Target;
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
        if let Some(player) = &game_objects[PLAYER] {
            if player.distance_to(&owner) <= player.sensors.sense_range as f32
                && owner.tile.is_none()
            {
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

    fn target(&self) -> Target {
        Target::None
    }
}

/// Attack another object.
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

#[typetag::serde]
impl Action for AttackAction {
    fn perform(
        &self,
        _game_state: &mut GameState,
        objects: &mut GameObjects,
        _owner: &mut Object,
    ) -> ActionResult {
        match self.target_id {
            Some(target_id) => {
                // TODO: Replace with defend action.
                if let Some(ref mut _target) = objects[target_id] {
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

    fn target(&self) -> Target {
        Target::BlockingObject
    }
}

/// Move an object
// TODO: Maybe create enum target {self, other{object_id}} to use for any kind of targetable action.
#[derive(Debug, Serialize, Deserialize)]
pub struct MoveAction {
    direction: Direction,
}

impl MoveAction {
    // TODO: use level
    pub fn new(direction: Direction, level: i32) -> Self {
        MoveAction { direction }
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
        let (dx, dy) = self.direction.to_xy();
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

    fn target(&self) -> Target {
        Target::EmptyObject
    }
}
