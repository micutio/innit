//! Module Ai
//!
//! Structures and methods for constructing the game ai.

// internal imports

use crate::entity::action::{Action, Target, TargetCategory};
use crate::entity::control::{Ai, Controller};
use crate::entity::object::Object;
use crate::{
    core::game_objects::GameObjects,
    entity::action::hereditary::{ActInjectRnaVirus, ActPass},
};
use crate::{core::game_state::GameState, entity::action::hereditary::ActProduceVirion};
use rand::seq::{IteratorRandom, SliceRandom};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// As the name suggests this AI passes its turn forever.
/// This might actually be replaced with [Object.control](crate::entity::object::Object) == None, which save some more CPU cycles.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiPassive;

#[typetag::serde]
impl Ai for AiPassive {
    fn act(
        &mut self,
        _state: &mut GameState,
        _objects: &mut GameObjects,
        _owner: &mut Object,
    ) -> Box<dyn Action> {
        Box::new(ActPass::default())
    }
}

/// Each turn chooses a random valid action with a random valid target
#[derive(Debug, Serialize, Deserialize, Clone)]
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
            return Box::new(ActPass::default());
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
            Box::new(ActPass::default())
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiVirus {}

impl AiVirus {
    pub fn new() -> Self {
        AiVirus {}
    }
}

#[typetag::serde]
impl Ai for AiVirus {
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
            assert!(!owner.dna.raw.is_empty());
            return Box::new(ActInjectRnaVirus::new(
                Target::from_pos(&owner.pos, &target.pos),
                owner.dna.raw.clone(),
            ));
        }
        Box::new(ActPass::default())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiForceVirusProduction {
    original_ai: Option<Controller>,
    turns_active: Option<i32>,
    current_turn: i32,
    rna: Option<Vec<u8>>,
}

impl AiForceVirusProduction {
    pub fn new_duration(
        original_ai: Option<Controller>,
        duration_turns: i32,
        rna: Option<Vec<u8>>,
    ) -> Self {
        AiForceVirusProduction {
            original_ai,
            turns_active: Some(duration_turns),
            current_turn: 0,
            rna,
        }
    }

    fn _new_forever(original_ai: Option<Controller>, rna: Option<Vec<u8>>) -> Self {
        AiForceVirusProduction {
            original_ai,
            turns_active: None,
            current_turn: 0,
            rna,
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
                if let Some(original_ai) = self.original_ai.take() {
                    owner.control.replace(original_ai);
                    return Box::new(ActPass::update());
                }
            } else {
                self.current_turn += 1;
            }
        }
        Box::new(ActProduceVirion::new(self.rna.clone()))
    }
}
