use rand::{Rng, RngCore};

use crate::core::game_env::GameEnv;
use crate::core::game_objects::GameObjects;
use crate::core::position::Position;
use crate::entity::action::*;
use crate::entity::control::Controller;
use crate::entity::genetics::GeneLibrary;
use crate::entity::object::Object;
use crate::entity::player::PLAYER;
use crate::ui::game_frontend::AnimationType;
use crate::util::game_rng::{GameRng, RngExtended};

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub enum MsgClass {
    Info,
    Action,
    Alert,
    Story,
}

/// Messages are expressed as colored text.
pub type Messages = Vec<(String, MsgClass)>;

/// The message log can add text from any string collection.
pub trait MessageLog {
    fn add<T: Into<String>>(&mut self, message: T, class: MsgClass);
}

impl MessageLog for Vec<(String, MsgClass)> {
    fn add<T: Into<String>>(&mut self, message: T, class: MsgClass) {
        self.push((message.into(), class));
    }
}

/// Results from processing an objects action for that turn, in ascending rank.
#[derive(PartialEq, Debug)]
pub enum ObjectProcResult {
    NoAction,   // object did not act and is still pondering its turn
    NoFeedback, // action completed, but requires no visual feedback
    CheckEnterPlayerFOV {
        origin: Position,
    }, // check whether the object has entered the player FOV
    Message {
        msg: String,
        class: MsgClass,
        origin: Position,
    }, // display a message, if position of origin is visible to the player
    Animate {
        anim_type: AnimationType,
        origin: Position,
    }, // play given animation to visualise action
    UpdatePlayerFOV, // action completed, requires updating FOV
    ReRender,   // trigger full re-render of the game world
}

/// The game state struct contains all information necessary to represent the current state of the
/// game, EXCEPT the object vector.
#[derive(Serialize, Deserialize)]
pub struct GameState {
    pub env: GameEnv,
    pub rng: GameRng,
    pub log: Messages,
    pub turn: u128,
    pub dungeon_level: u32,
    pub gene_library: GeneLibrary,
    pub current_obj_index: usize,
    pub current_player_index: usize,
}

impl GameState {
    pub fn new(env: GameEnv, level: u32) -> Self {
        GameState {
            // create the list of game messages and their colours, starts empty
            env,
            rng: GameRng::new_from_u64_seed(rand::thread_rng().next_u64()),
            log: vec![],
            turn: 0,
            dungeon_level: level,
            gene_library: GeneLibrary::new(),
            current_obj_index: 0,
            current_player_index: PLAYER,
        }
    }

    pub fn is_players_turn(&self) -> bool {
        self.current_obj_index == self.current_player_index
    }

    /// Process an object's turn i.e., let it perform as many actions as it has energy for.
    pub fn process_object(&mut self, objects: &mut GameObjects) -> ObjectProcResult {
        // unpack object to process its next action
        if let Some(mut active_object) = objects.extract(self.current_obj_index) {
            if let Some(Controller::Player(_)) = active_object.control {
                // update player index just in case we have multiple player controlled objects
                self.current_player_index = self.current_obj_index;
                // abort the turn if the player has not decided on the next action and also cannot metabolize anymore
                if !active_object.has_next_action()
                    && active_object.processors.energy == active_object.processors.energy_storage
                {
                    return ObjectProcResult::NoAction;
                }
            }

            // Object takes the turn, which has three phases:
            // 1. turn preparation
            // 2. turn action
            // 3. turn conclusion
            trace!(
                "{} | {}'s turn now @energy {}/{}",
                self.current_obj_index,
                active_object.visual.name,
                active_object.processors.energy,
                active_object.processors.energy_storage
            );

            // TURN PREPARATION ///////////////////////////////////////////////////////////////////
            // Innit doesn't have any action preparations as of yet.

            // TURN ACTION ////////////////////////////////////////////////////////////////////////
            // If not enough energy available try to metabolise.
            let process_result =
                if active_object.processors.energy < active_object.processors.energy_storage {
                    // replenish energy
                    active_object.metabolize();
                    ObjectProcResult::NoFeedback
                } else if let Some(next_action) = active_object.extract_next_action(self, objects) {
                    // use up energy before action
                    active_object.processors.energy -= next_action.get_energy_cost();
                    self.process_action(objects, &mut active_object, next_action)
                } else {
                    panic!("How can an object 'has_next_action' but NOT have an action?");
                    // ObjectProcResult::NoFeedback
                };

            // TURN CONCLUSION ////////////////////////////////////////////////////////////////////
            // Apply recurring effects so that the player can factor this into the next action.

            // TODO: Damage from overloading

            // Random mutation
            if active_object.dna.raw.is_empty() {
                println!("{} dna is empty!", active_object.visual.name);
            }
            if !active_object.dna.raw.is_empty()
                && self.rng.flip_with_prob(1.0 - active_object.gene_stability)
            {
                // mutate the object's genome by randomly flipping a bit
                let random_gene = self.rng.gen_range(0, active_object.dna.raw.len());
                let old_gene = active_object.dna.raw[random_gene];
                debug!(
                    "{} flipping gene: 0b{:08b}",
                    active_object.visual.name, active_object.dna.raw[random_gene]
                );
                active_object.dna.raw[random_gene] ^= self.rng.random_bit();
                debug!(
                    "{}            to: 0b{:08b}",
                    active_object.visual.name, active_object.dna.raw[random_gene]
                );

                // apply new genome to object
                let (sensors, processors, actuators, dna) = self
                    .gene_library
                    .decode_dna(active_object.dna.dna_type, active_object.dna.raw.as_slice());
                active_object.change_genome(sensors, processors, actuators, dna);

                // TODO: Show mutation effect as diff between old and new genome!
                if self.current_obj_index == self.current_player_index {
                    self.log.add(
                        format!("A mutation occurred in your genome {}", old_gene),
                        MsgClass::Alert,
                    );
                } else if let Some(player) = &objects[self.current_player_index] {
                    debug!(
                        "sensing range: {}, dist: {}",
                        player.sensors.sensing_range as f32,
                        player.pos.distance(&active_object.pos)
                    );
                    if player.pos.distance(&active_object.pos)
                        <= player.sensors.sensing_range as f32
                    {
                        // don't record all tiles passing constantly
                        self.log.add(
                            format!("{} mutated!", active_object.visual.name),
                            MsgClass::Info,
                        );
                    }
                }
            }

            // return object back to objects vector
            objects[self.current_obj_index].replace(active_object);

            // finally increase object index and turn counter
            self.current_obj_index = (self.current_obj_index + 1) % objects.get_num_objects();
            if self.current_obj_index == PLAYER {
                self.turn += 1;
            }

            // return the result of our action
            process_result
        } else {
            panic!("no object at index {}", self.current_obj_index);
        }
    }

    /// Process an action of the given object.
    fn process_action(
        &mut self,
        objects: &mut GameObjects,
        actor: &mut Object,
        action: Box<dyn Action>,
    ) -> ObjectProcResult {
        // first execute action, then process result and return
        match action.perform(self, objects, actor) {
            ActionResult::Success { callback } => callback,
            ActionResult::Failure => ObjectProcResult::NoAction, // how to handle fails?
            ActionResult::Consequence { action } => {
                // if we have a side effect, process it first and then the `main` action
                self.process_action(objects, actor, action.unwrap())
                // TODO: Think of a way to handle both results of action and consequence.
                // TODO: Extract into function, recursively bubble up results and return the highest
                //       priority or just a list of all of them
            }
        }
    }
}

// NOTE: All functions below are hot candidates for a rewrite because they might not fit into the
//       new command pattern system.

pub struct Transition {
    pub level: u32,
    pub value: u32,
}

/// Return a value that depends on dungeon level.
/// The table specifies what value occurs after each level, default is 0.
pub fn from_dungeon_level(table: &[Transition], level: u32) -> u32 {
    table
        .iter()
        .rev()
        .find(|transition| level >= transition.level)
        .map_or(0, |transition| transition.value)
}
