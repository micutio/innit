//! The DNA contains all core information, excluding temporary info such as position etc. This
//! module allows to generate objects from DNA and modify them using mutation as well as crossing.
//! Decoding DNA delivers attributes and functions that fall into one of three gene types.
//!
//! ## Gene Types
//!
//! * sensor - gathering information of the environment
//! * processor - decision making
//! * actuator - interacting with other objects and the game world
//!
//! ## Shape of the DNA
//!
//! +------+--------------+---------------+-------------+
//! | 0x00 | gene type ID | genome length | trait genes |
//! +------+--------------+---------------+-------------+
//!
//! ### sensor
//!
//! #### Qualities
//!
//! | Trait   | ID   | Attributes |
//! | ------- | ---- | ---------- |
//! | sensor  | 0x01 | range      |
//!
//! ### processor
//!
//! #### Qualities
//!
//! | Trait         | ID   | Attributes |
//! | ------------- | ---- | ---------- |
//! | quick action  | 0x02 | count      |
//!
//! ### actuator
//!
//! #### Qualities
//!
//! | Trait   | ID   | Attributes |
//! | ------- | ---- | ---------- |
//! | move    | 0x03 | speed      |
//! | attack  | 0x04 | damage     |
//! | defend  | 0x05 | health     |
//! | rest    | 0x06 | HP regen   |
//!
//! A DNA Genome is implemented as a string of hexadecimal numbers. The start of a gene is marked
//! by the number zero. Genes can overlap, so that parsing the new gene resumes "in the middle" of
//! a previous gene. The genes should be small and encoding the presence of a quality. Attributes or
//! versatility is then controlled by the cumulative occurrence of a gene.
//! Basically: the more often a gene occurs, the stronger its trait will be.
// TODO: How to handle synergies/anti-synergies?
// TODO: How to calculate energy cost per action?
// TODO: Design a DNA parser and a mapping from symbol to trait struct.
// TODO: Can behavior be encoded in the genome too i.e., fight or flight?

use rand::Rng;
use std::cmp;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;

use crate::entity::action::*;
use crate::ui::game_input::PlayAction;
use crate::util::game_rng::GameRng;
use crate::util::generate_gray_code;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub enum SuperTrait {
    Sense,
    Process,
    Actuate,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
pub enum ActionId {
    Sense,
    Quick,
    Primary,
    Secondary,
    Move,
    Attack,
    Defend,
    Rest,
}

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct ActionPrototype {
    pub super_trait: SuperTrait,
    pub trait_id:    ActionId,
    pub name:        String,
    pub parameter:   i32,
}

/// Construct a new player action from a given key code.
/// Get player's action item that corresponds with the player input and construct a new action
/// from the parameters in both
// NOTE: In the future we'll have to consider mouse clicks as well.
pub fn get_player_action(input: PlayAction, prototype: &ActionPrototype) -> Box<dyn Action> {
    use self::ActionId::*;
    use ui::game_input::PlayActionParameter::*;
    match input {
        PlayAction {
            trait_id: Move,
            param: Orientation(dir),
        } => Box::new(MoveAction::new(dir, prototype.parameter)),
        // TODO: Check if we can actually move!
        // (PlayInput(Move(ActionId::Move, Cardinal(Direction))), Some(action_prototype)) =>
        // Box::new(MoveAction::new(Direction, action_prototype.parameter)),
        _ => Box::new(PassAction),
    }
}

// In the following we describe each of the three integral components that give an object its body,
// mind and behavior.
// Each of them contains a list of actions related to their domain.
// TODO: Should attributes be fix on o-trait level or full-on generic as list of attribute objects?
// TODO: How to best model synergies and anti-synergies across o-traits?

/// This may or may not be body parts. Actuators like organells can also benefit the attributes.
/// Sensors contain:
/// - attributes
///   - range of effective sensing
///   - accuracy of sensing [future feature]
/// - functions:
///   - sense environment
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Sensor {
    actions: Vec<ActionPrototype>,
}

impl Sensor {
    pub fn new() -> Self {
        Sensor {
            actions: Vec::new(),
        }
    }
}

/// Processors contain:
/// - attributes:
///   - capacity, a quantization/modifier of how energy-costly and complex the functions are
/// - functions:
///   - setting of primary/secondary actions [player]
///   - decision making algorithm [player/ai]
///   - ai control [ai]
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Processor {
    actions: Vec<ActionPrototype>,
}

impl Processor {
    pub fn new() -> Self {
        Processor {
            actions: Vec::new(),
        }
    }
}

/// Actuators can actually be concrete body parts e.g., organelles, spikes
/// Actuators contain:
/// - attributes:
///   - speed, a modifier of the energy cost of the functions
/// - functions:
///   - move
///   - attack
///   - defend
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Actuator {
    actions: Vec<ActionPrototype>,
}

impl Actuator {
    pub fn new() -> Self {
        Actuator {
            actions: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeneRecord {
    name:        String,
    super_trait: SuperTrait,
    action:      ActionId,
    /* attributes: Vec<?>,
     * synergies: Vec<?>,
     * anti-synergies: Vec<?>, */
}

/// The gene library lets the user define genes.
/// Input should look like this:
///   - trait name
///   - super trait
///   - attributes
///   - action
///   - synergies
///   - anti-synergies
///
/// Actions can be chosen from a pool of predefined methods.
// TODO: How to encode non-action attributes e.g, cell membrane thickness?
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Default)]
pub struct GeneLibrary {
    /// Traits are now supposed to be generic, so enums are no longer the way to go
    // TODO: Re-use enum ActionId to identify actions instead. They are basically already doing it.
    gray_to_trait: HashMap<u8, String>,
    /// This one should be straight forward. Lets the custom traits make use of supertrait specific
    /// attributes.
    trait_to_super: HashMap<u8, SuperTrait>,
    /// As mentioned above, re-use TraitIDs to allow mappings to actions.
    trait_to_action: HashMap<u8, ActionId>,
    /// Vector of gray code with index corresponding to its binary representation
    gray_code: Vec<u8>,
    /// Count the number of traits we have, sort of as a running id.
    trait_count: usize,
}

impl GeneLibrary {
    pub fn new() -> Self {
        GeneLibrary {
            gray_to_trait:   HashMap::new(),
            trait_to_super:  HashMap::new(),
            trait_to_action: HashMap::new(),
            gray_code:       generate_gray_code(4),
            trait_count:     0,
        }
    }

    fn add_gene(&mut self, gene: GeneRecord) {
        debug!("[dna] adding new gene to the library: {:?}", gene);
        let trait_code = self.gray_code[self.trait_count];
        self.gray_to_trait.insert(trait_code, gene.name);
        self.trait_to_super.insert(trait_code, gene.super_trait);
        self.trait_to_action.insert(trait_code, gene.action);
        self.trait_count += 1;
    }

    fn read_genes_from_file() -> Result<Vec<GeneRecord>, Box<dyn Error>> {
        let mut json_genes = String::new();
        let mut file = File::open("data/genes")?;
        file.read_to_string(&mut json_genes)?;
        let result = serde_json::from_str::<Vec<GeneRecord>>(&json_genes)?;
        Ok(result)
    }

    pub fn init_genes(&mut self) {
        match GeneLibrary::read_genes_from_file() {
            Ok(genes) => {
                for gene in genes {
                    debug!("adding gene {:?} to the library", gene);
                    self.add_gene(gene);
                }
            }
            Err(..) => {
                error!("[dna] Enable to read gene file!");
            }
        }
    }

    // TODO: Add parameters to control distribution of sense, process and actuate!
    // TODO: Use above parameters for NPC definitions, readable from datafiles!
    pub fn new_dna(&self, game_rng: &mut GameRng, avg_genome_len: usize) -> Vec<u8> {
        let mut dna = Vec::new();
        // randomly grab a trait and add trait id, length and random attribute value
        for _ in 0..10 {
            // push 0x00 first as the genome start symbol
            dna.push(0 as u8);
            // pick random trait number from list
            let trait_num = game_rng.gen_range(0, self.trait_count);
            // add trait id
            dna.push(self.gray_code[trait_num]);
            // add length // TODO: encode length in ActionId
            dna.push(1);
            // add random attribute value
            dna.push(game_rng.gen_range(0, 16) as u8);
        }
        debug!("new dna generated: {:?}", dna);
        dna
    }

    pub fn decode_dna(&self, dna: &[u8]) -> (Sensor, Processor, Actuator) {
        let mut start_ptr: usize = 0;
        let mut end_ptr: usize = dna.len();
        let mut sensor = Sensor::new();
        let mut processor = Processor::new();
        let mut actuator = Actuator::new();

        self.decode_gene(
            dna,
            start_ptr,
            end_ptr,
            &mut sensor,
            &mut processor,
            &mut actuator,
        );

        (sensor, processor, actuator)
    }

    fn decode_gene(
        &self,
        dna: &[u8],
        mut start_ptr: usize,
        mut end_ptr: usize,
        s: &mut Sensor,
        p: &mut Processor,
        a: &mut Actuator,
    ) {
        // pointing at 0x00 now
        start_ptr += 1;
        // read trait id
        match dna.get(start_ptr) {
            Some(val) => match self.trait_to_super.get(val) {
                Some(SuperTrait::Sense) => {
                    // do something with sense
                }
                Some(SuperTrait::Process) => {
                    // do something with process
                }
                Some(SuperTrait::Actuate) => {
                    // do something with actuate
                }
                None => return,
            },
            None => return,
        }
        // read length
        end_ptr = cmp::min(end_ptr, dna[start_ptr] as usize);
        //
    }
}
