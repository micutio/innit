use crate::entity::act::{self, Action};
use crate::entity::{genetics, Object};
use crate::game::msg::MessageLog;
use crate::game::{self, consts, msg, ObjectStore};
use crate::util::rng;

use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};

/// The game state struct contains all information necessary to represent the current state of the
/// game, EXCEPT the object vector. Each field in this struct is serialised and written to the save
/// file and thus persistent data. No volatile data is allowed here.
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct State {
    pub rng: rng::GameRng,
    pub log: msg::Log,
    pub turn: u128,
    pub dungeon_level: u32,
    pub gene_library: genetics::GeneLibrary,
    pub obj_idx: usize,    // current object index
    pub player_idx: usize, // current player index
}

impl State {
    pub fn new(level: u32) -> Self {
        let rng_seed = if let Some(seed_param) = game::env().seed {
            seed_param
        } else {
            rand::thread_rng().next_u64()
        };

        info!("using rng seed: {}", rng_seed);

        State {
            // create the list of game messages and their colours, starts empty
            rng: rng::GameRng::new_from_u64_seed(rng_seed),
            log: msg::Log::new(),
            turn: 0,
            dungeon_level: level,
            gene_library: genetics::GeneLibrary::new(),
            obj_idx: 0,
            player_idx: consts::PLAYER,
        }
    }

    pub fn is_players_turn(&self) -> bool {
        self.obj_idx == self.player_idx
    }

    pub fn player_energy_full(&self, objects: &ObjectStore) -> bool {
        if let Some(player) = &objects[self.player_idx] {
            player.processors.energy == player.processors.energy_storage
        } else {
            false
        }
    }

    /// Process an object's turn i.e., let it perform as many actions as it has energy for.
    pub fn process_object(&mut self, objects: &mut ObjectStore) -> act::ObjectFeedback {
        // unpack object to process its next action
        if let Some(mut actor) = objects.extract_by_index(self.obj_idx) {
            // Object takes the turn, which has three phases:
            // 1. turn preparation
            // 2. turn action
            // 3. turn conclusion
            // if actorect.physics.is_visible {
            trace!(
                "{} | {}'s turn now @energy {}/{}",
                self.obj_idx,
                actor.visual.name,
                actor.processors.energy,
                actor.processors.energy_storage
            );
            // }

            // TURN PREPARATION ///////////////////////////////////////////////////////////////////
            // check whether the object is ready to take the turn, i.e.: has an action queued up
            if actor.is_player() {
                // update player index just in case we have multiple player controlled objects
                self.player_idx = self.obj_idx;
                // abort the turn if the player has not decided on the next action and also cannot
                // metabolize anymore
                let can_rest = actor.processors.energy == actor.processors.energy_storage;
                if !actor.has_next_action() && can_rest {
                    objects.replace(self.obj_idx, actor);
                    return act::ObjectFeedback::NoAction;
                }
            }

            // TURN ACTION ////////////////////////////////////////////////////////////////////////
            let mut process_result = self.take_turn(objects, &mut actor);

            // TURN CONCLUSION ////////////////////////////////////////////////////////////////////

            // check whether object is overloaded
            self.conclude_overload(&mut actor);

            // have a chance at random mutation
            self.conclude_mutate(&mut actor);

            // check whether object is still alive
            self.conclude_ageing(objects, &mut actor, &mut process_result);

            // return object back to objects vector, if still alive
            self.conclude_recycle_obj(objects, actor, &mut process_result);

            // finally increase object index and turn counter
            self.conclude_advance_turn(objects.get_obj_count());

            // return the result of our action
            process_result
        } else {
            // panic!("no object at index {}", self.obj_idx);
            // objects.get_vector_mut().remove(self.obj_idx);

            // increase object index and turn counter
            self.conclude_advance_turn(objects.get_obj_count());
            act::ObjectFeedback::Render
        }
    }

    fn take_turn(&mut self, objects: &mut ObjectStore, actor: &mut Object) -> act::ObjectFeedback {
        if actor.control.is_none() {
            return act::ObjectFeedback::NoFeedback;
        }

        if actor.processors.energy < actor.processors.energy_storage {
            // if not enough energy available try to replenish energy via metabolising
            actor.metabolize();
            if self.is_players_turn() {
                return act::ObjectFeedback::Render;
            } else {
                return act::ObjectFeedback::NoFeedback;
            }
        }

        if let Some(next_action) = actor.extract_next_action(self, objects) {
            // use up energy before action
            if actor.physics.is_visible && next_action.get_identifier().ne("pass") {
                trace!("next action: {}", next_action.get_identifier());
            }
            if next_action.get_energy_cost() > actor.processors.energy_storage {
                self.log.add(
                    "You don't have enough energy for that!",
                    msg::MsgClass::Info,
                );
                act::ObjectFeedback::NoFeedback
            } else {
                actor.processors.energy -= next_action.get_energy_cost();
                self.process_action(objects, actor, next_action)
            }
        } else {
            // TODO: Turn this into a proper error message and graceful shutdown.
            panic!("How can an object 'has_next_action' but NOT have an action?");
        }
    }

    /// Process an action of the given object.
    fn process_action(
        &mut self,
        objects: &mut ObjectStore,
        actor: &mut Object,
        action: Box<dyn Action>,
    ) -> act::ObjectFeedback {
        // first execute action, then process result and return
        match action.perform(self, objects, actor) {
            act::ActionResult::Success { callback } => match callback {
                act::ObjectFeedback::NoFeedback => act::ObjectFeedback::NoFeedback,
                _ => callback,
            },
            act::ActionResult::Failure => act::ObjectFeedback::NoFeedback, // TODO: How to handle fails?
            act::ActionResult::Consequence {
                callback,
                follow_up,
            } => {
                let consequence_feedback = self.process_action(objects, actor, follow_up);
                match (&callback, &consequence_feedback) {
                    (act::ObjectFeedback::NoFeedback, _) => consequence_feedback,
                    (act::ObjectFeedback::NoAction, _) => consequence_feedback,
                    (act::ObjectFeedback::GameOver, _) => callback,
                    (act::ObjectFeedback::Render, _) => callback,
                    (act::ObjectFeedback::UpdateHud, _) => callback,
                    (act::ObjectFeedback::GenomeManipulator, _) => callback,
                }
            }
        }
    }

    fn conclude_overload(&mut self, actor: &mut Object) {
        if actor.inventory.items.len() as i32 > actor.actuators.volume {
            // println!(
            //     "{} at ({},{}) is overloaded to the point of damage",
            //     actor.visual.name, actor.pos.x, actor.pos.y
            // );
            actor.actuators.hp -= 1;
            if actor.is_player() {
                self.log
                    .add("You're overloaded! Taking damage...", msg::MsgClass::Alert);
            }
        }
    }

    fn conclude_mutate(&mut self, actor: &mut Object) {
        if actor.tile.is_some() && !actor.physics.is_blocking {
            // no need to mutate empty tiles
            return;
        }

        if !actor.alive {
            return;
        }

        if actor.dna.raw.is_empty() {
            debug!("{} dna is empty!", actor.visual.name);
            return;
        }

        if rng::RngExtended::flip_with_prob(&mut self.rng, 1.0 - actor.gene_stability) {
            // mutate the object's genome by randomly flipping a bit
            let gene_width = 3;
            let trait_count = actor.dna.raw.len() / gene_width as usize;
            let trait_start = self.rng.gen_range(0..trait_count) * gene_width;
            let trait_end = trait_start + gene_width as usize;
            let gene_pos = self.rng.gen_range(0..gene_width as usize);
            let position = trait_start + gene_pos;
            let old_gene = actor.dna.raw[position];
            let old_trait = Vec::from_iter(actor.dna.raw[trait_start..trait_end].iter().cloned());
            // ^ = bitwise exclusive or
            let new_gene = old_gene ^ rng::RngExtended::random_bit(&mut self.rng);
            // Replace the modified gene in the dna. The change will become effectual once the
            // cell procreates or "reincarnates".
            actor.dna.raw[position] = new_gene;
            let new_trait = Vec::from_iter(actor.dna.raw[trait_start..trait_end].iter().cloned());
            trace!(
                "{} flipping gene {:08b} to {:08b}",
                actor.visual.name,
                old_gene,
                new_gene
            );

            if actor.is_player() {
                let gene_no = (trait_start / gene_width as usize) + 1; // start from 1 instead of 0
                let old_trait_dna = &self
                    .gene_library
                    .dna_to_traits(actor.dna.dna_type, &old_trait)
                    .3;
                let new_trait_dna = &self
                    .gene_library
                    .dna_to_traits(actor.dna.dna_type, &new_trait)
                    .3;
                let is_old_trait_junk = old_trait_dna.simplified.is_empty();
                let old_trait_name = if is_old_trait_junk {
                    "junk?"
                } else {
                    &old_trait_dna.simplified[0].trait_name
                };
                let is_new_trait_junk = new_trait_dna.simplified.is_empty();
                let new_trait_name = if is_new_trait_junk {
                    "junk?"
                } else {
                    &new_trait_dna.simplified[0].trait_name
                };
                self.log.add(
                    format!(
                        "Gene {} mutated from {} to {}",
                        gene_no, old_trait_name, new_trait_name
                    ),
                    msg::MsgClass::Alert,
                );
            } else if actor.physics.is_visible {
                self.log.add(
                    format!("Mutation occurred in {}!", actor.visual.name),
                    msg::MsgClass::Info,
                );
            }
        }
    }

    fn conclude_ageing(
        &mut self,
        objects: &mut ObjectStore,
        actor: &mut Object,
        process_result: &mut act::ObjectFeedback,
    ) {
        if actor.actuators.hp <= 0 {
            actor.die(self, objects);
        } else {
            actor.processors.life_elapsed += 1;
            // the hud should be updated to show the new lifetime of the player unless already
            // something render-worthy happened
            if actor.is_player() {
                if let act::ObjectFeedback::NoFeedback = process_result {
                    *process_result = act::ObjectFeedback::UpdateHud
                };
            }
        }
    }

    fn conclude_recycle_obj(
        &mut self,
        objects: &mut ObjectStore,
        actor: Object,
        process_result: &mut act::ObjectFeedback,
    ) {
        if !actor.alive {
            if actor.physics.is_visible {
                self.log
                    .add(format!("{} died!", actor.visual.name), msg::MsgClass::Alert);
                debug!("{} died!", actor.visual.name);
            }

            // if the dead object is a player then keep it in the world,
            // otherwise remove it.
            // NOTE: Maybe keep dead material around for scavenging.
            if actor.is_player() {
                objects[self.obj_idx].replace(actor);
            } else {
                objects.get_vector_mut().remove(self.obj_idx);
            }
            // if the "main" player is dead, the game is over
            if self.obj_idx == consts::PLAYER {
                *process_result = act::ObjectFeedback::GameOver;
            }
        } else {
            objects[self.obj_idx].replace(actor);
        }
    }

    fn conclude_advance_turn(&mut self, obj_count: usize) {
        let next_obj_idx = (self.obj_idx + 1) % obj_count;
        if next_obj_idx < self.obj_idx {
            self.turn += 1;
        }
        self.obj_idx = next_obj_idx;
    }
}
