use crate::core::game_objects::GameObjects;
use crate::core::innit_env;
use crate::entity::action::*;
use crate::entity::genetics::GeneLibrary;
use crate::entity::object::Object;
use crate::entity::player::PLAYER;
use crate::util::game_rng::GameRng;
use rand::RngCore;
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
#[derive(Serialize, Deserialize)]
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
        if let Some(mut active_object) = objects.extract_by_index(self.obj_idx) {
            // Object takes the turn, which has three phases:
            // 1. turn preparation
            // 2. turn action
            // 3. turn conclusion
            // if active_object.physics.is_visible {
            trace!(
                "{} | {}'s turn now @energy {}/{}",
                self.obj_idx,
                active_object.visual.name,
                active_object.processors.energy,
                active_object.processors.energy_storage
            );
            // }

            if active_object.is_player() {
                // update player index just in case we have multiple player controlled objects
                self.player_idx = self.obj_idx;
                // abort the turn if the player has not decided on the next action and also cannot metabolize anymore
                if !active_object.has_next_action()
                    && active_object.processors.energy == active_object.processors.energy_storage
                {
                    objects.replace(self.obj_idx, active_object);
                    return ObjectFeedback::NoAction;
                }
            }

            // TURN PREPARATION ///////////////////////////////////////////////////////////////////
            // Innit doesn't have any action preparations as of yet.

            // TURN ACTION ////////////////////////////////////////////////////////////////////////
            let mut process_result =
                // If not enough energy available try to metabolise.
                if active_object.control.is_none() {
                    ObjectFeedback::NoFeedback
                } else if active_object.processors.energy < active_object.processors.energy_storage {
                    // replenish energy
                    active_object.metabolize();
                    if self.is_players_turn() {
                        ObjectFeedback::Render
                    } else {
                        ObjectFeedback::NoFeedback
                    }
                } else if let Some(next_action) = active_object.extract_next_action(self, objects) {
                    // use up energy before action
                    if active_object.physics.is_visible && next_action.get_identifier().ne("pass") {
                        debug!("next action: {}", next_action.get_identifier());
                    }
                    if next_action.get_energy_cost() > active_object.processors.energy_storage {
                        self.log.add("You don't have enough energy for that!", MsgClass::Info);
                        ObjectFeedback::NoFeedback
                    } else {
                        active_object.processors.energy -= next_action.get_energy_cost();
                        self.process_action(objects, &mut active_object, next_action)
                    }
                } else {
                    panic!("How can an object 'has_next_action' but NOT have an action?");
                    // ObjectProcResult::NoFeedback
                };
            if !active_object.physics.is_visible && !active_object.physics.is_always_visible {
                // process_result.clear();
                process_result = ObjectFeedback::NoFeedback;
            }

            // TURN CONCLUSION ////////////////////////////////////////////////////////////////////
            // Apply recurring effects so that the player can factor this into the next action.

            if active_object.inventory.items.len() as i32 > active_object.actuators.volume {
                active_object.actuators.hp -= 1;
                if active_object.is_player() {
                    self.log
                        .add("You're overloaded! Taking damage...", MsgClass::Alert);
                }
            }

            // Random mutation
            // TODO: Perform random mutation when cells are procreating/multiplying, not just by chance every turn.
            // if active_object.dna.raw.is_empty() {
            //     println!("{} dna is empty!", active_object.visual.name);
            // }
            // if !active_object.dna.raw.is_empty()
            //     && self.rng.flip_with_prob(1.0 - active_object.gene_stability)
            // {
            //     // mutate the object's genome by randomly flipping a bit
            //     let random_gene = self.rng.gen_range(0, active_object.dna.raw.len());
            //     let old_gene = active_object.dna.raw[random_gene];
            //     debug!(
            //         "{} flipping gene: 0b{:08b}",
            //         active_object.visual.name, active_object.dna.raw[random_gene]
            //     );
            //     active_object.dna.raw[random_gene] ^= self.rng.random_bit();
            //     debug!(
            //         "{}            to: 0b{:08b}",
            //         active_object.visual.name, active_object.dna.raw[random_gene]
            //     );
            //
            //     // apply new genome to object
            //     let (sensors, processors, actuators, dna) = self
            //         .gene_library
            //         .decode_dna(active_object.dna.dna_type, active_object.dna.raw.as_slice());
            //     active_object.change_genome(sensors, processors, actuators, dna);
            //
            //     // TODO: Show mutation effect as diff between old and new genome!
            //     if self.current_obj_index == self.current_player_index {
            //         self.log.add(
            //             format!("A mutation occurred in your genome {}", old_gene),
            //             MsgClass::Alert,
            //         );
            //     } else if let Some(player) = &objects[self.current_player_index] {
            //         debug!(
            //             "sensing range: {}, dist: {}",
            //             player.sensors.sensing_range as f32,
            //             player.pos.distance(&active_object.pos)
            //         );
            //         if player.pos.distance(&active_object.pos)
            //             <= player.sensors.sensing_range as f32
            //         {
            //             // don't record all tiles passing constantly
            //             self.log.add(
            //                 format!("{} mutated!", active_object.visual.name),
            //                 MsgClass::Info,
            //             );
            //         }
            //     }
            // }

            // check whether object is still alive
            if active_object.actuators.hp == 0 {
                active_object.die(self, objects);
            }

            // return object back to objects vector, if still alive
            if !active_object.alive && active_object.physics.is_visible {
                self.log.add(
                    format!("{} died!", active_object.visual.name),
                    MsgClass::Alert,
                );
                debug!("{} died!", active_object.visual.name);

                // if the dead object is a player then keep it in the world,
                // otherwise remove it.
                // TODO: Think about keeping dead material around.
                if active_object.is_player() {
                    objects[self.obj_idx].replace(active_object);
                } else {
                    objects.get_vector_mut().remove(self.obj_idx);
                }
                // if the "main" player is dead, the game is over
                if self.obj_idx == PLAYER {
                    process_result = ObjectFeedback::GameOver;
                }
            } else {
                objects[self.obj_idx].replace(active_object);
            }

            // finally increase object index and turn counter
            self.obj_idx = (self.obj_idx + 1) % objects.get_obj_count();
            if self.obj_idx == 0 {
                self.turn += 1;
            }

            // return the result of our action
            process_result
        } else {
            // panic!("no object at index {}", self.obj_idx);
            // objects.get_vector_mut().remove(self.obj_idx);

            // increase object index and turn counter
            self.obj_idx = (self.obj_idx + 1) % objects.get_obj_count();
            if self.obj_idx == 0 {
                self.turn += 1;
            }
            ObjectFeedback::Render
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
            ActionResult::Failure => ObjectFeedback::NoAction, // how to handle fails?
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
}
