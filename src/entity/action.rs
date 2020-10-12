//! Module Action provides the action interface, which is used to create any kind of action that
//! can be performed by the player or an NPC.
//! Any action is supposed to be assigned to one of the three trait families (sensing, prcessing,
//! actuating) of an object

// TODO: Create actions for setting and using quick/primary/secondary actions.

use std::fmt::Debug;

use tcod::colors;

use crate::core::game_objects::GameObjects;
use crate::core::game_state::{GameState, MessageLog, ObjectProcResult};
use crate::entity::object::Object;
use crate::player::PLAYER;

/// Targets can only be adjacent to the object: north, south, east, west or the objects itself.
#[derive(Clone, Debug, PartialEq)]
pub enum TargetCategory {
    Any,
    BlockingObject,
    EmptyObject,
    None,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum Target {
    North,
    South,
    East,
    West,
    Center,
}

impl Target {
    fn to_xy(&self) -> (i32, i32) {
        match self {
            Target::North => (0, -1),
            Target::South => (0, 1),
            Target::East => (1, 0),
            Target::West => (-1, 0),
            Target::Center => (0, 0),
        }
    }

    pub fn from_xy(x1: i32, y1: i32, x2: i32, y2: i32) -> Target {
        match (x2 - x1, y2 - y1) {
            (0, -1) => Target::North,
            (0, 1) => Target::South,
            (1, 0) => Target::East,
            (-1, 0) => Target::West,
            (0, 0) => Target::Center,
            _ => panic!("calling from_xy on non-adjacent target"),
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
pub trait Action: ActionClone + Debug {
    fn perform(
        &self,
        game_state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult;

    fn get_target_category(&self) -> TargetCategory;

    fn set_target(&mut self, t: Target);

    fn set_level(&mut self, lvl: i32);
}

pub trait ActionClone {
    fn clone_action(&self) -> Box<dyn Action>;
}

impl<T> ActionClone for T
where
    T: Action + Clone + 'static,
{
    fn clone_action(&self) -> Box<dyn Action> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Action> {
    fn clone(&self) -> Self {
        self.clone_action()
    }
}

/// Dummy action for passing the turn.
#[derive(Debug, Serialize, Deserialize, Clone)]
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
            if player.distance_to(&owner) <= player.sensors.sensing_range as f32
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

    fn get_target_category(&self) -> TargetCategory {
        TargetCategory::None
    }

    fn set_target(&mut self, _: Target) {}

    fn set_level(&mut self, lvl: i32) {}
}

/// Attack another object.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttackAction {
    lvl: i32,
    target: Target,
}

impl AttackAction {
    pub fn new() -> Self {
        AttackAction {
            lvl: 0,
            target: Target::Center,
        }
    }
}

#[typetag::serde]
impl Action for AttackAction {
    fn perform(
        &self,
        _game_state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult {
        // get coords of self position plus direction
        // find any objects that are at that position and blocking
        // assert that there is only one available
        // return
        let target_pos: (i32, i32) = (
            owner.x + self.target.to_xy().0,
            owner.y + self.target.to_xy().1,
        );
        let valid_targets: Vec<&Object> = objects
            .get_vector()
            .iter()
            .flatten()
            .filter(|o| o.physics.is_blocking && o.pos().eq(&target_pos))
            .collect();

        assert!(valid_targets.len() <= 1);
        if let Some(target_obj) = valid_targets.first() {
            // TODO: Take damage
            ActionResult::Success {
                callback: ObjectProcResult::CheckEnterFOV,
            }
        } else {
            ActionResult::Failure
        }
    }

    fn get_target_category(&self) -> TargetCategory {
        TargetCategory::BlockingObject
    }

    fn set_target(&mut self, target: Target) {
        self.target = target;
    }

    fn set_level(&mut self, lvl: i32) {
        self.lvl = lvl;
    }
}

/// Move an object
// TODO: Maybe create enum target {self, other{object_id}} to use for any kind of targetable action.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoveAction {
    lvl: i32,
    direction: Target,
}

impl MoveAction {
    // TODO: use level
    pub fn new() -> Self {
        MoveAction {
            lvl: 0,
            direction: Target::Center,
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

    fn get_target_category(&self) -> TargetCategory {
        TargetCategory::EmptyObject
    }

    fn set_target(&mut self, target: Target) {
        self.direction = target;
    }

    fn set_level(&mut self, lvl: i32) {
        self.lvl = lvl;
    }
}
