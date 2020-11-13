//! Module Ai
//!
//! Structures and methods for constructing the game ai.

// internal imports

use rand::seq::{IteratorRandom, SliceRandom};
use std::fmt::Debug;

use crate::core::game_objects::GameObjects;
use crate::core::game_state::GameState;
use crate::entity::action::{
    ActInjectVirus, ActPass, ActProduceVirus, Action, Target, TargetCategory,
};
use crate::entity::control::{Ai, Controller};
use crate::entity::object::Object;

#[derive(Debug, Serialize, Deserialize)]
pub struct AiPassive;

impl AiPassive {
    pub fn new() -> Self {
        AiPassive {}
    }
}

#[typetag::serde]
impl Ai for AiPassive {
    fn act(
        &mut self,
        _state: &mut GameState,
        _objects: &mut GameObjects,
        _owner: &mut Object,
    ) -> Box<dyn Action> {
        Box::new(ActPass)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiRandom;

impl AiRandom {
    pub fn new() -> Self {
        AiRandom {}
    }
}

#[typetag::serde]
impl Ai for AiRandom {
    fn act(
        &mut self,
        state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> Box<dyn Action> {
        // If the object doesn't have any action, return a pass.
        if owner.actuators.actions.is_empty()
            && owner.processors.actions.is_empty()
            && owner.sensors.actions.is_empty()
        {
            return Box::new(ActPass);
        }

        // Get a list of possible targets, blocking and non-blocking, and search only for actions
        // that can be used with these targets.
        let adjacent_targets: Vec<&Object> = objects
            .get_vector()
            .iter()
            .flatten()
            .filter(|obj| {
                owner.pos.is_adjacent(&obj.pos)
                    && (obj.physics.is_blocking || !objects.is_pos_occupied(&obj.pos))
            })
            // .filter_map(|o| o.as_ref())
            .collect();

        println!("adjacent target count: {:?}", &adjacent_targets.len());

        let mut valid_targets = vec![
            TargetCategory::None,
            TargetCategory::Any,
            TargetCategory::EmptyObject,
            TargetCategory::BlockingObject,
        ];

        // options:
        // a) targets empty => only None a.k.a self
        if adjacent_targets.is_empty() {
            valid_targets.retain(|t| *t == TargetCategory::None);
        }

        // b) no empty targets available
        if !adjacent_targets.iter().any(|t| !t.physics.is_blocking) {
            valid_targets.retain(|t| *t != TargetCategory::EmptyObject)
        }

        // d) no blocking targets available => remove blocking from selection
        if !adjacent_targets.iter().any(|t| t.physics.is_blocking) {
            valid_targets.retain(|t| *t != TargetCategory::BlockingObject);
        }

        // dbg!("valid targets: {:?}", &valid_targets);

        // find an action that matches one of the available target categories
        let possible_actions: Vec<&Box<dyn Action>> = owner
            .actuators
            .actions
            .iter()
            .chain(owner.processors.actions.iter())
            .chain(owner.sensors.actions.iter())
            .filter(|a| valid_targets.contains(&(*a).get_target_category()))
            .collect();

        if let Some(a) = possible_actions.choose(&mut state.rng) {
            let mut boxed_action = a.clone_action();
            match boxed_action.get_target_category() {
                TargetCategory::None => boxed_action.set_target(Target::Center),
                TargetCategory::BlockingObject => {
                    if let Some(target_obj) = adjacent_targets
                        .iter()
                        .filter(|at| at.physics.is_blocking)
                        .choose(&mut state.rng)
                    {
                        boxed_action.set_target(Target::from_pos(&owner.pos, &target_obj.pos))
                    }
                }
                TargetCategory::EmptyObject => {
                    if let Some(target_obj) = adjacent_targets
                        .iter()
                        .filter(|at| !at.physics.is_blocking)
                        .choose(&mut state.rng)
                    {
                        boxed_action.set_target(Target::from_pos(&owner.pos, &target_obj.pos))
                    }
                }
                TargetCategory::Any => {
                    if let Some(target_obj) = adjacent_targets.choose(&mut state.rng) {
                        boxed_action.set_target(Target::from_pos(&owner.pos, &target_obj.pos))
                    }
                }
            }
            boxed_action
        } else {
            Box::new(ActPass)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiRnaVirus {}

#[typetag::serde]
impl Ai for AiRnaVirus {
    fn act(
        &mut self,
        state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> Box<dyn Action> {
        // if there is an adjacent cell, attempt to infect it
        if let Some(target) = objects
            .get_vector()
            .iter()
            .flatten()
            .filter(|obj| {
                owner.pos.is_adjacent(&obj.pos)
                    && (obj.physics.is_blocking)
                    && obj
                        .processors
                        .receptors
                        .iter()
                        .any(|e| owner.processors.receptors.contains(e))
            })
            .choose(&mut state.rng)
        {
            return Box::new(ActInjectVirus::new(Target::from_pos(
                &owner.pos,
                &target.pos,
            )));
        }
        Box::new(ActPass)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiForceVirusProduction {
    original_ai: Option<Controller>,
    turns_active: Option<i32>,
    current_turn: i32,
}

impl AiForceVirusProduction {
    pub fn new_duration(original_ai: Option<Controller>, duration_turns: i32) -> Self {
        AiForceVirusProduction {
            original_ai,
            turns_active: Some(duration_turns),
            current_turn: 0,
        }
    }

    fn _new_forever(original_ai: Option<Controller>) -> Self {
        AiForceVirusProduction {
            original_ai,
            turns_active: None,
            current_turn: 0,
        }
    }
}

#[typetag::serde]
impl Ai for AiForceVirusProduction {
    fn act(
        &mut self,
        _state: &mut GameState,
        _objects: &mut GameObjects,
        owner: &mut Object,
    ) -> Box<dyn Action> {
        if let Some(t) = self.turns_active {
            if self.current_turn == t {
                owner.control = self.original_ai.take();
            } else {
                self.current_turn += 1;
            }
        }

        Box::new(ActProduceVirus::new())
    }
}
