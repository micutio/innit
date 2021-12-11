//! Module Action provides the action interface, which is used to create any kind of action that
//! can be performed by the player or an NPC.
//! Any action is supposed to be assigned to one of the three trait families (sensing, prcessing,
//! actuating) of an object

pub(crate) mod hereditary;
pub(crate) mod inventory;

use crate::game::game_objects::GameObjects;
use crate::game::game_state::{GameState, ObjectFeedback};
use crate::game::position::Position;
use crate::entity::action::hereditary::*;
use crate::entity::object::Object;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Possible target groups are: objects, empty space, anything or self (None).
/// Non-targeted actions will always be applied to the performing object itself.
#[derive(Clone, Debug, PartialEq)]
pub enum TargetCategory {
    Any,
    BlockingObject,
    EmptyObject,
    None,
}

/// Targets can only be adjacent to the object: north, south, east, west or the objects itself.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum Target {
    North,
    South,
    East,
    West,
    Center,
}

impl Target {
    fn to_pos(&self) -> Position {
        match self {
            Target::North => Position::new(0, -1),
            Target::South => Position::new(0, 1),
            Target::East => Position::new(1, 0),
            Target::West => Position::new(-1, 0),
            Target::Center => Position::new(0, 0),
        }
    }

    /// Returns the target direction from acting position p1 to targeted position p2.
    pub fn from_pos(p1: &Position, p2: &Position) -> Target {
        match p1.offset(p2) {
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
    Success { callback: ObjectFeedback },
    /// Failed to perform an action, ideally without any side effect.
    Failure,
    /// Another action happens automatically after this one.
    Consequence {
        callback: ObjectFeedback,
        follow_up: Box<dyn Action>,
    },
}

/// Interface for all actions.
/// They need to be `performable` and have a cost (even if it's 0).
#[cfg_attr(not(target_arch = "wasm32"), typetag::serde(tag = "type"))]
pub trait Action: ActionClone + Debug {
    fn perform(
        &self,
        state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult;

    fn set_target(&mut self, t: Target);

    fn set_level(&mut self, lvl: i32);

    fn get_target_category(&self) -> TargetCategory;

    fn get_level(&self) -> i32;

    fn get_identifier(&self) -> String;

    fn get_energy_cost(&self) -> i32;

    fn to_text(&self) -> String;
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

pub fn action_from_string(action_descriptor: &str) -> Result<Box<dyn Action>, String> {
    match action_descriptor {
        "ActPass" => Ok(Box::new(ActPass::default())),
        "ActMove" => Ok(Box::new(ActMove::new())),
        "ActRepairStructure" => Ok(Box::new(ActRepairStructure::new())),
        "ActAttack" => Ok(Box::new(ActAttack::new())),
        "ActEditGenome" => Ok(Box::new(ActEditGenome::new())),
        _ => Err(format!("cannot find action for {}", action_descriptor)),
    }
}
