//! Module Ai
//!
//! Structures and methods for constructing the game ai.
// external imports
// use rand::Rng;
// use tcod::colors;

// internal imports
// use entity::object::ObjectVec;
// use game_state::{GameState, MessageLog};
// use ui::game_frontend::FovMap;
use crate::core::game_objects::GameObjects;
use crate::entity::action::{Action, PassAction};
use crate::entity::object::Object;
use crate::util::game_rng::GameRng;

use std::fmt::Debug;

#[typetag::serde(tag = "type")]
pub trait Ai: Debug {
    fn act(
        &self,
        object: &mut Object,
        game_objects: &mut GameObjects,
        game_rng: &mut GameRng,
    ) -> Box<dyn Action>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PassiveAi;

impl PassiveAi {
    pub fn new() -> Self {
        PassiveAi {}
    }
}

#[typetag::serde]
impl Ai for PassiveAi {
    fn act(
        &self,
        object: &mut Object,
        game_objects: &mut GameObjects,
        game_rng: &mut GameRng,
    ) -> Box<dyn Action> {
        Box::new(PassAction)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RandomAi;

impl RandomAi {
    pub fn new() -> Self {
        RandomAi {}
    }
}

#[typetag::serde]
impl Ai for RandomAi {
    fn act(
        &self,
        object: &mut Object,
        game_objects: &mut GameObjects,
        game_rng: &mut GameRng,
    ) -> Box<dyn Action> {
        // If the object doesn't have any action, return a pass.
        if object.actuators.actions.is_empty()
            && object.processors.actions.is_empty()
            && object.sensors.actions.is_empty()
        {
            return Box::new(PassAction);
        }
        // Get a list of possible targets, blocking and non-blocking, and search only for actions
        // that can be used with these targets.
        let adjacent_targets: Vec<&Object> = game_objects
            .get_vector()
            .iter()
            .filter(|obj| match obj {
                Some(o) => ((o.x - object.x) - (o.y - object.y)).abs() == 1,
                None => false,
            })
            .filter_map(|o| o.as_ref())
            .collect();

        if adjacent_targets.is_empty() {
            // try finding an action with no or self target
            return Box::new(PassAction);
        }

        // iterate over all available actions and pick one at random with randomised target
        let mut result: Option<Box<dyn Action>> = None;
        while result.is_none() {}
        // dummy return for now
        Box::new(PassAction)
    }
}
