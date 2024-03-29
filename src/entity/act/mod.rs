//! Module Action provides the action interface, which is used to create any kind of action that
//! can be performed by the player or an NPC.
//! Any action is supposed to be assigned to one of the three trait families (sensing, prcessing,
//! actuating) of an object

mod complement_system;
mod hereditary;
mod inventory;

pub use self::complement_system::*;
pub use self::hereditary::*;
pub use self::inventory::*;

use crate::entity::object::Object;
use crate::game::Position;
use crate::game::{ObjectStore, State};

use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Interface for all actions.
/// They need to be `performable` and have a cost (even if it's 0).
#[cfg_attr(not(target_arch = "wasm32"), typetag::serde(tag = "type"))]
pub trait Action: ActionClone + Debug {
    fn perform(
        &self,
        state: &mut State,
        objects: &mut ObjectStore,
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
        "ActPass" => Ok(Box::new(Pass)),
        "ActMove" => Ok(Box::new(Move::new())),
        "ActRepairStructure" => Ok(Box::new(RepairStructure::new())),
        "ActAttack" => Ok(Box::new(Attack::new())),
        "ActEditGenome" => Ok(Box::new(EditGenome::new())),
        _ => Err(format!("cannot find action for {}", action_descriptor)),
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

/// Results from processing an objects action for that turn, in ascending rank.
#[derive(PartialEq, Eq, Debug)]
pub enum ObjectFeedback {
    NoAction,   // object did not act and is still pondering its turn
    NoFeedback, // action completed, but requires no visual feedback
    Render,
    UpdateHud,
    GenomeManipulator,
    GameOver, // "main" player died
}

/// Possible target groups are: objects, empty space, anything or self (None).
/// Non-targeted actions will always be applied to the performing object itself.
#[derive(Clone, Debug, PartialEq, Eq)]
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
            Self::North => Position::from_xy(0, -1),
            Self::South => Position::from_xy(0, 1),
            Self::East => Position::from_xy(1, 0),
            Self::West => Position::from_xy(-1, 0),
            Self::Center => Position::from_xy(0, 0),
        }
    }

    /// Returns the target direction from acting position p1 to targeted position p2.
    pub fn from_pos(p1: &Position, p2: &Position) -> Self {
        match p1.offset(p2) {
            (0, -1) => Self::North,
            (0, 1) => Self::South,
            (1, 0) => Self::East,
            (-1, 0) => Self::West,
            (0, 0) => Self::Center,
            _ => panic!("calling from_xy on non-adjacent target"),
        }
    }
}
