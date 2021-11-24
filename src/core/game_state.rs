use crate::core::game_objects::GameObjects;
use crate::core::innit_env;
use crate::entity::action::*;
use crate::entity::genetics::GeneLibrary;
use crate::entity::object::Object;
use crate::entity::player::PLAYER;
use crate::util::game_rng::{GameRng, RngExtended};
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub enum MsgClass {
    Info,
    Action,
    Alert,
    Story,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Log {
    pub is_changed: bool,
    pub messages: Vec<(String, MsgClass)>,
}

impl Log {
    pub fn new() -> Self {
        Log {
            is_changed: false,
            messages: Vec::new(),
        }
    }
}

/// The message log can add text from any string collection.
pub trait MessageLog {
    fn add<T: Into<String>>(&mut self, message: T, class: MsgClass);
}

impl MessageLog for Log {
    /// Push a message into the log under two conditions:
    /// - either the log is empty
    /// - or the last message is not identical to the new message
    fn add<T: Into<String>>(&mut self, msg: T, class: MsgClass) {
        if self.messages.is_empty() {
            self.messages.push((msg.into(), class));
            self.is_changed = true;
            return;
        }

        if let Some(recent_msg) = self.messages.last() {
            let msg_str = msg.into();
            if !recent_msg.0.eq(&msg_str) {
                self.messages.push((msg_str, class));
                self.is_changed = true;
            }
        }
    }
}

/// Results from processing an objects action for that turn, in ascending rank.
#[derive(PartialEq, Debug)]
pub enum ObjectFeedback {
    NoAction,   // object did not act and is still pondering its turn
    NoFeedback, // action completed, but requires no visual feedback
    Render,
    UpdateHud,
    GenomeManipulator,
    GameOver, // "main" player died
}

/// The game state struct contains all information necessary to represent the current state of the
/// game, EXCEPT the object vector. Each field in this struct is serialised and written to the save
/// file and thus persistent data. No volatile data is allowed here.
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct GameState {
    pub rng: GameRng,
    pub log: Log,
    pub turn: u128,
    pub dungeon_level: u32,
    pub gene_library: GeneLibrary,
    pub obj_idx: usize,    // current object index
    pub player_idx: usize, // current player index
}

impl GameState {
    pub fn new(level: u32) -> Self {
        let rng_seed = if innit_env().is_using_fixed_seed {
            0
        } else {
            rand::thread_rng().next_u64()
        };

        GameState {
            // create the list of game messages and their colours, starts empty
            rng: GameRng::new_from_u64_seed(rng_seed),
            log: Log::new(),
            turn: 0,
            dungeon_level: level,
            gene_library: GeneLibrary::new(),
            obj_idx: 0,
            player_idx: PLAYER,
        }
    }

    pub fn is_players_turn(&self) -> bool {
        self.obj_idx == self.player_idx
    }

    pub fn player_energy_full(&self, objects: &GameObjects) -> bool {
        if let Some(player) = &objects[self.player_idx] {
            player.processors.energy == player.processors.energy_storage
        } else {
            false
        }
    }

    /// Process an object's turn i.e., let it perform as many actions as it has energy for.
    pub fn process_object(&mut self, objects: &mut GameObjects) -> ObjectFeedback {
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
                    return ObjectFeedback::NoAction;
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
            ObjectFeedback::Render
        }
    }

    fn take_turn(&mut self, objects: &mut GameObjects, actor: &mut Object) -> ObjectFeedback {
        if actor.control.is_none() {
            ObjectFeedback::NoFeedback
        } else if actor.processors.energy < actor.processors.energy_storage {
            // if not enough energy available try to replenish energy via metabolising
            actor.metabolize();
            if self.is_players_turn() {
                ObjectFeedback::Render
            } else {
                ObjectFeedback::NoFeedback
            }
        } else if let Some(next_action) = actor.extract_next_action(self, objects) {
            // use up energy before action
            if actor.physics.is_visible && next_action.get_identifier().ne("pass") {
                trace!("next action: {}", next_action.get_identifier());
            }
            if next_action.get_energy_cost() > actor.processors.energy_storage {
                self.log
                    .add("You don't have enough energy for that!", MsgClass::Info);
                ObjectFeedback::NoFeedback
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
        objects: &mut GameObjects,
        actor: &mut Object,
        action: Box<dyn Action>,
    ) -> ObjectFeedback {
        // first execute action, then process result and return
        match action.perform(self, objects, actor) {
            ActionResult::Success { callback } => match callback {
                ObjectFeedback::NoFeedback => ObjectFeedback::NoFeedback,
                _ => callback,
            },
            ActionResult::Failure => ObjectFeedback::NoFeedback, // TODO: How to handle fails?
            ActionResult::Consequence {
                callback,
                follow_up,
            } => {
                let consequence_feedback = self.process_action(objects, actor, follow_up);
                match (&callback, &consequence_feedback) {
                    (ObjectFeedback::NoFeedback, _) => consequence_feedback,
                    (ObjectFeedback::NoAction, _) => consequence_feedback,
                    (ObjectFeedback::GameOver, _) => callback,
                    (ObjectFeedback::Render, _) => callback,
                    (ObjectFeedback::UpdateHud, _) => callback,
                    (ObjectFeedback::GenomeManipulator, _) => callback,
                }
            }
        }
    }

    fn conclude_overload(&mut self, actor: &mut Object) {
        if actor.inventory.items.len() as i32 > actor.actuators.volume {
            actor.actuators.hp -= 1;
            if actor.is_player() {
                self.log
                    .add("You're overloaded! Taking damage...", MsgClass::Alert);
            }
        }
    }

    fn conclude_mutate(&mut self, actor: &mut Object) {
        if actor.tile.is_some() && !actor.physics.is_blocking {
            // no need to mutate empty tiles
            return;
        }

        if actor.dna.raw.is_empty() {
            println!("{} dna is empty!", actor.visual.name);
            return;
        }

        if self.rng.flip_with_prob(1.0 - actor.gene_stability) {
            // mutate the object's genome by randomly flipping a bit
            let random_position = self.rng.gen_range(0..actor.dna.raw.len());
            let old_gene = actor.dna.raw[random_position];
            // ^ = bitwise exclusive or
            let new_gene = old_gene ^ self.rng.random_bit();
            // Replace the modified gene in the dna. The change will become effectual once the
            // cell procreates or "reincarnates".
            actor.dna.raw[random_position] = new_gene;
            debug!(
                "{} flipping gene 0b{:08b} to 0b{:08b}",
                actor.visual.name, old_gene, new_gene
            );

            // TODO: Show mutation effect as diff between old and new genome!
            if actor.is_player() {
                self.log.add(
                    format!(
                        "Gene {} mutated from 0b{:08b} to 0b{:08b}",
                        random_position, old_gene, new_gene
                    ),
                    MsgClass::Alert,
                );
            } else if actor.physics.is_visible {
                self.log.add(
                    format!("Mutation occurred in {}!", actor.visual.name),
                    MsgClass::Info,
                );
            }
        }
    }

    fn conclude_ageing(
        &mut self,
        objects: &mut GameObjects,
        actor: &mut Object,
        process_result: &mut ObjectFeedback,
    ) {
        if actor.actuators.hp == 0 {
            actor.die(self, objects);
        } else {
            actor.processors.life_elapsed += 1;
            // the hud should be updated to show the new lifetime of the player unless already
            // something render-worthy happened
            if actor.is_player() {
                if let ObjectFeedback::NoFeedback = process_result {
                    *process_result = ObjectFeedback::UpdateHud
                };
            }
        }
    }

    fn conclude_recycle_obj(
        &mut self,
        objects: &mut GameObjects,
        actor: Object,
        process_result: &mut ObjectFeedback,
    ) {
        if !actor.alive && actor.physics.is_visible {
            self.log
                .add(format!("{} died!", actor.visual.name), MsgClass::Alert);
            debug!("{} died!", actor.visual.name);

            // if the dead object is a player then keep it in the world,
            // otherwise remove it.
            // NOTE: Maybe keep dead material around for scavenging.
            if actor.is_player() {
                objects[self.obj_idx].replace(actor);
            } else {
                objects.get_vector_mut().remove(self.obj_idx);
            }
            // if the "main" player is dead, the game is over
            if self.obj_idx == PLAYER {
                *process_result = ObjectFeedback::GameOver;
            }
        } else {
            objects[self.obj_idx].replace(actor);
        }
    }

    fn conclude_advance_turn(&mut self, obj_count: usize) {
        self.obj_idx = (self.obj_idx + 1) % obj_count;
        if self.obj_idx == 0 {
            self.turn += 1;
        }
    }
}
