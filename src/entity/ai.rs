//! Module Ai
//!
//! Structures and methods for constructing the game ai.

// internal imports

use crate::entity::act::{self, Action};
use crate::entity::control::{self, Ai};
use crate::entity::{genetics, Object};
use crate::game::{self, ObjectStore, State};
use crate::util::random::RngExtended;

use rand::seq::{IteratorRandom, SliceRandom};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// As the name suggests this AI passes its turn forever.
/// This might actually be replaced with [Object.control](crate::entity::object::Object) == None,
/// which saves some more CPU cycles.
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Passive;

#[cfg_attr(not(target_arch = "wasm32"), typetag::serde)]
impl Ai for Passive {
    fn act(
        &mut self,
        _state: &mut State,
        _objects: &mut ObjectStore,
        _owner: &mut Object,
    ) -> Box<dyn Action> {
        Box::new(act::Pass)
    }
}

/// Each turn chooses a random valid action with a random valid target
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct RandomAction;

impl RandomAction {
    pub const fn new() -> Self {
        Self {}
    }
}

#[cfg_attr(not(target_arch = "wasm32"), typetag::serde)]
impl Ai for RandomAction {
    fn act(
        &mut self,
        state: &mut State,
        objects: &mut ObjectStore,
        owner: &mut Object,
    ) -> Box<dyn Action> {
        // If the object doesn't have any action, return a pass.
        if owner.actuators.actions.is_empty()
            && owner.processors.actions.is_empty()
            && owner.sensors.actions.is_empty()
        {
            return Box::new(act::Pass);
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
            act::TargetCategory::None,
            act::TargetCategory::Any,
            act::TargetCategory::EmptyObject,
            act::TargetCategory::BlockingObject,
        ];

        // options:
        // a) targets empty => only None a.k.a self
        if adjacent_targets.is_empty() {
            valid_targets.retain(|t| *t == act::TargetCategory::None);
        }

        // b) no empty targets available
        if !adjacent_targets.iter().any(|t| !t.physics.is_blocking) {
            valid_targets.retain(|t| *t != act::TargetCategory::EmptyObject);
        }

        // d) no blocking targets available => remove blocking from selection
        if !adjacent_targets.iter().any(|t| t.physics.is_blocking) {
            valid_targets.retain(|t| *t != act::TargetCategory::BlockingObject);
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
                act::TargetCategory::None => boxed_action.set_target(act::Target::Center),
                act::TargetCategory::BlockingObject => {
                    if let Some(target_obj) = adjacent_targets
                        .iter()
                        .filter(|at| at.physics.is_blocking)
                        .choose(&mut state.rng)
                    {
                        boxed_action.set_target(act::Target::from_pos(&owner.pos, &target_obj.pos));
                    }
                }
                act::TargetCategory::EmptyObject => {
                    if let Some(target_obj) = adjacent_targets
                        .iter()
                        .filter(|at| !at.physics.is_blocking)
                        .choose(&mut state.rng)
                    {
                        boxed_action.set_target(act::Target::from_pos(&owner.pos, &target_obj.pos));
                    }
                }
                act::TargetCategory::Any => {
                    if let Some(target_obj) = adjacent_targets.choose(&mut state.rng) {
                        boxed_action.set_target(act::Target::from_pos(&owner.pos, &target_obj.pos));
                    }
                }
            }
            boxed_action
        } else {
            Box::new(act::Pass)
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RandomWalk;

#[cfg_attr(not(target_arch = "wasm32"), typetag::serde)]
impl Ai for RandomWalk {
    fn act(
        &mut self,
        state: &mut State,
        objects: &mut ObjectStore,
        owner: &mut Object,
    ) -> Box<dyn Action> {
        // try and find some empty adjacent cells that can be walked to
        if let Some(t) = objects
            .get_vector()
            .iter()
            .flatten()
            .filter(|obj| {
                owner.pos.is_adjacent(&obj.pos)
                    && (obj.physics.is_blocking || !objects.is_pos_occupied(&obj.pos))
            })
            // .filter_map(|o| o.as_ref())
            .collect::<Vec<&Object>>()
            .choose(&mut state.rng)
        {
            let mut action = Box::new(act::Move::new());
            action.set_target(act::Target::from_pos(&owner.pos, &t.pos));
            action
        } else {
            Box::new(act::Pass)
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Virus;

impl Virus {
    pub const fn new() -> Self {
        Self {}
    }
}

#[cfg_attr(not(target_arch = "wasm32"), typetag::serde)]
impl Ai for Virus {
    fn act(
        &mut self,
        state: &mut State,
        objects: &mut ObjectStore,
        owner: &mut Object,
    ) -> Box<dyn Action> {
        // Check whether we're surrounded by wall tiles:
        // If that's the case, die to avoid ending up with unreachable virions deep inside the wall
        // tile tissue.
        if !objects
            .get_neighborhood_tiles(owner.pos)
            .flatten()
            .any(|t| !t.physics.is_blocking)
        {
            owner.die(state, objects);
            return Box::new(act::Pass);
        }

        // if there is an adjacent cell, attempt to infect it
        // println!("neighborhood START");
        if let Some(target) = objects
            .get_neighborhood_tiles(owner.pos)
            .flatten()
            .chain(
                objects
                    .get_non_tiles()
                    .iter()
                    .flatten()
                    .filter(|obj| owner.pos.is_adjacent(&obj.pos)),
            )
            .filter(|obj| {
                let is_blocking = obj.physics.is_blocking;
                let is_not_virus = obj.dna.dna_type != genetics::DnaType::Rna;
                let is_receptor_match = obj
                    .processors
                    .receptors
                    .iter()
                    .any(|e1| owner.processors.receptors.iter().any(|e2| e1.typ == e2.typ));
                is_blocking && is_not_virus && is_receptor_match
            })
            .choose(&mut state.rng)
        {
            let target_dir = act::Target::from_pos(&owner.pos, &target.pos);
            return Box::new(act::InjectRnaVirus::new(target_dir, owner.dna.raw.clone()));
        }
        // println!("neighborhood END");

        // if there is no target to infect, try a random walk instead
        if state.rng.flip_with_prob(0.1) {
            // println!("RANDOM WALK START");
            let blocking_entity_pos: Vec<game::Position> = objects
                .get_non_tiles()
                .iter()
                .flatten()
                .filter(|obj| obj.physics.is_blocking && obj.pos.is_adjacent(&owner.pos))
                .map(|obj| obj.pos)
                // .inspect(|p| println!("blocking_entity pos: {},{}", p.x, p.y))
                .collect();
            let target_position = objects
                .get_neighborhood_tiles(owner.pos)
                .flatten()
                .filter(|obj| !obj.physics.is_blocking && !blocking_entity_pos.contains(&obj.pos))
                .map(|obj| obj.pos)
                // .inspect(|p| println!("available target: {},{}", p.x, p.y))
                .choose(&mut state.rng);
            // print!("RANDOM WALK END");

            if let Some(target_pos) = target_position {
                // println!(" WITH TARGET: {},{}", target_pos.x, target_pos.y);
                let mut action = Box::new(act::Move::new());
                action.set_target(act::Target::from_pos(&owner.pos, &target_pos));
                return action;
            }
        }

        // if nothing else sticks, just pass
        Box::new(act::Pass)
    }
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct ForcedVirusProduction {
    original_ai: Option<control::Controller>,
    turns_active: Option<i32>,
    current_turn: i32,
    rna: Option<Vec<u8>>,
}

impl ForcedVirusProduction {
    pub const fn new_duration(
        original_ai: Option<control::Controller>,
        duration_turns: i32,
        rna: Option<Vec<u8>>,
    ) -> Self {
        Self {
            original_ai,
            turns_active: Some(duration_turns),
            current_turn: 0,
            rna,
        }
    }

    const fn _new_forever(original_ai: Option<control::Controller>, rna: Option<Vec<u8>>) -> Self {
        Self {
            original_ai,
            turns_active: None,
            current_turn: 0,
            rna,
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), typetag::serde)]
impl Ai for ForcedVirusProduction {
    fn act(
        &mut self,
        _state: &mut State,
        _objects: &mut ObjectStore,
        owner: &mut Object,
    ) -> Box<dyn Action> {
        if let Some(t) = self.turns_active {
            if self.current_turn == t {
                if let Some(original_ai) = self.original_ai.take() {
                    owner.control.replace(original_ai);
                    return Box::new(act::Pass);
                }
            } else {
                self.current_turn += 1;
            }
        }
        Box::new(act::ProduceVirion::new(self.rna.clone()))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WallTile;

#[cfg_attr(not(target_arch = "wasm32"), typetag::serde)]
impl Ai for WallTile {
    fn act(
        &mut self,
        state: &mut State,
        objects: &mut ObjectStore,
        owner: &mut Object,
    ) -> Box<dyn Action> {
        // If the object doesn't have any action, return a pass.
        if owner.actuators.actions.is_empty()
            && owner.processors.actions.is_empty()
            && owner.sensors.actions.is_empty()
        {
            return Box::new(act::Pass);
        }

        if owner.processors.life_elapsed >= owner.processors.life_expectancy {
            if let Some(killswitch) = owner
                .processors
                .actions
                .iter()
                .find(|a| a.get_identifier().eq("killswitch"))
            {
                let mut killswitch_action = killswitch.clone_action();
                killswitch_action.set_target(act::Target::Center);
                return killswitch_action;
            }
        } else if let Some(fission_action) = owner
            .actuators
            .actions
            .iter()
            .find(|a| a.get_identifier().eq("bin. fission"))
        {
            // If the tile can perform fission, check whether a neighboring cell is available
            // and also contains a high enough concentration of growth gradient.

            let target_cell = objects
                .get_neighborhood_tiles(owner.pos)
                .flatten()
                .filter(|obj| {
                    obj.tile.as_ref().map_or(false, |_tile| {
                        !obj.physics.is_blocking || !objects.is_pos_occupied(&obj.pos)
                    })
                })
                .choose(&mut state.rng);
            if let Some(target) = target_cell {
                if let Some(target_tile) = &target.tile {
                    if state.rng.flip_with_prob(target_tile.morphogen / 2.0) {
                        let mut fission = fission_action.clone_action();
                        fission.set_target(act::Target::from_pos(&owner.pos, &target.pos));
                        return fission;
                    }
                }
            }
        }
        Box::new(act::Pass)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FloorTile;

#[cfg_attr(not(target_arch = "wasm32"), typetag::serde)]
impl Ai for FloorTile {
    fn act(
        &mut self,
        _state: &mut State,
        objects: &mut ObjectStore,
        owner: &mut Object,
    ) -> Box<dyn Action> {
        if objects.is_pos_occupied(&owner.pos) {
            Box::new(act::UpdateComplementProteins)
        } else {
            Box::new(act::Pass)
        }
    }
}
