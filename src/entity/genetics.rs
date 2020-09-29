//! The DNA contains all core information, excluding temporary info such as position etc. This
//! module allows to generate objects from DNA and modify them using mutation as well as crossing.
//! Decoding DNA delivers attributes and functions that fall into one of three gene types.
//!
//! Inspiration: https://creatures.fandom.com/wiki/ChiChi_Norn_Genome
//!
//! ## Gene Types
//!
//! * sensor - gathering information of the environment
//! * processor - decision making
//! * actuator - interacting with other objects and the game world
//!
//! ## Shape of the DNA
//!
//! +------+-----------+---------------+-------------+
//! | 0x00 | gene name | genome length | trait genes |
//! +------+-----------+---------------+-------------+
//!
//!
//! A DNA Genome is implemented as a string of hexadecimal numbers. The start of a gene is marked
//! by the number zero. Genes can overlap, so that parsing the new gene resumes "in the middle" of
//! a previous gene. The genes should be small and encoding the presence of a quality. Attributes or
//! versatility is then controlled by the cumulative occurrence of a gene.
//! Basically: the more often a gene occurs, the stronger its trait will be.
// TODO: How to handle synergies/anti-synergies?
// TODO: How to calculate energy cost per action?
// TODO: Can behavior be encoded in the genome too i.e., fight or flight?
// TODO: Should attributes be fix on trait level or full-on generic as list of attribute objects?
// TODO: How to best model synergies and anti-synergies across traits?

use rand::Rng;
use std::cmp;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;

use crate::entity::action::*;
use crate::entity::object::Object;
use crate::ui::game_input::PlayerAction;
use crate::util::game_rng::GameRng;
use crate::util::generate_gray_code;

/// All traits belong to one of three major categories, called trait families.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub enum TraitFamily {
    Sensing,
    Processing,
    Actuating,
}

// /// Traits can influence attributes and actions.
// #[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone, Copy)]
// #[serde(untagged)]
// pub enum Trait {
//     // #[serde(rename = "sttraitattribute")]
//     TAttribute(TraitAttribute),
//     // #[serde(rename = "sttraitaction")]
//     TAction(TraitAction),
// }

// Genetic traits are linked to actions and attributes.
// Actions are supposed to be linked to key inputs.
// Relationships:
//      - an attribute can be influenced by multiple traits
//      - an action can be influenced by multiple traits
//      - traits need to know how often they appear in the genome
//      - attributes and actions need to know this too!
//
// alternative:

pub struct GeneTrait {
    pub trait_name: String,
    pub trait_family: TraitFamily,
    pub attribute: TraitAttribute, // Vec<TraitAttribute>
    pub action: Box<dyn Action>,   // TraitAction
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum TraitAttribute {
    SensingRange,
    Hp,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum TraitAction {
    Sense,     // sensing
    Quick,     // quick action
    Primary,   // primary action
    Secondary, // secondary action
    Move,
    Attack,
    Defend,
    Rest,
}

// /// Action prototypes consist of a trait action as well as a count of this trait's occurrences in
// /// the genome.
// /// TODO: Instead of prototypes, immediately instantiate action and hand over target at action call!
// #[derive(PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
// pub struct ActionPrototype {
//     pub trait_id: TraitAction,
//     pub parameter: i32,
// }

/// Construct a new player action from a given key code.
/// Get player's action item that corresponds with the player input and
/// construct a new action from the parameters in both
// NOTE: In the future we'll have to consider mouse clicks as well.
pub fn build_player_action(player: &mut Object, input: PlayerAction) -> Box<dyn Action> {
    use crate::ui::game_input::PlayerAction::*;

    match input {
        DefaultAction(dir) => player.getDefaultAction(dir),
        _ => Box::new(PassAction),
    }
}

/// This may or may not be body parts. Actuators like organelles can also benefit the attributes.
/// Sensors contain:
/// - attributes
///   - range of effective sensing
///   - accuracy of sensing [future feature]
/// - functions:
///   - sense environment
#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct Sensors {
    pub actions: Vec<(Box<dyn Action>, i32)>,
    pub sense_range: i32,
}

impl Sensors {
    pub fn new() -> Self {
        Sensors {
            actions: Vec::new(),
            sense_range: 0,
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
#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct Processors {
    pub actions: Vec<(Box<dyn Action>, i32)>,
}

impl Processors {
    pub fn new() -> Self {
        Processors {
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
#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct Actuators {
    pub actions: Vec<(Box<dyn Action>, i32)>,
    pub hp: i32,
}

impl Actuators {
    pub fn new() -> Self {
        Actuators {
            actions: Vec::new(),
            hp: 0,
        }
    }
}

/// Gene Records hold all necessary information for a single gene.
/// Genes can either encode actions, attributes or both.
#[derive(Serialize, Deserialize, Debug)]
pub struct GeneRecord {
    name: String,
    super_trait: TraitFamily,
    action: TraitAction,
    /* attributes: Vec<?>,
     * synergies: Vec<?>,
     * anti-synergies: Vec<?>, */
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Dna {
    pub raw: Vec<u8>,
    pub simplified: Vec<TraitFamily>,
}

impl Dna {
    pub fn new() -> Dna {
        Dna {
            raw: Vec::new(),
            simplified: Vec::new(),
        }
    }
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
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Default)]
pub struct GeneLibrary {
    /// Traits are now supposed to be generic, so enums are no longer the way to go.
    /// Traits are encoded in gray code.
    gray_to_trait: HashMap<u8, Trait>,
    /// This one should be straight forward. Lets the custom traits make use of trait-family specific
    /// attributes.
    pub trait_by_family: HashMap<Trait, TraitFamily>,
    /// As mentioned above, re-use TraitIDs to allow mappings to actions.
    // trait_to_action: HashMap<u8, TraitAction>,
    /// Vector of gray code with index corresponding to its binary representation
    gray_code: Vec<u8>,
    /// Count the number of traits we have, sort of as a running id.
    trait_count: usize,
}

impl GeneLibrary {
    pub fn new() -> Self {
        let traits: Vec<Trait> = vec![
            Trait::TAttribute(TraitAttribute::SensingRange),
            Trait::TAction(TraitAction::Sense),
            Trait::TAction(TraitAction::Quick),
            Trait::TAction(TraitAction::Primary),
            Trait::TAction(TraitAction::Secondary),
            Trait::TAction(TraitAction::Move),
            Trait::TAction(TraitAction::Attack),
            Trait::TAction(TraitAction::Defend),
            Trait::TAttribute(TraitAttribute::Hp),
            Trait::TAction(TraitAction::Rest),
        ];
        // TODO: Introduce constant N for total number of traits to assert gray code vector length.
        let gray_code = generate_gray_code(4);
        let gray_to_trait: HashMap<u8, Trait> = traits
            .iter()
            .enumerate()
            .map(|(x, y)| (gray_code[x + 1], *y))
            .collect();

        // TODO: This is really unwieldy.
        // At least extract to it's own function or better yet, try to make it more generic.
        let mut trait_by_family: HashMap<Trait, TraitFamily> = HashMap::new();
        trait_by_family.insert(
            Trait::TAttribute(TraitAttribute::SensingRange),
            TraitFamily::Sensing,
        );
        trait_by_family.insert(Trait::TAction(TraitAction::Sense), TraitFamily::Sensing);
        trait_by_family.insert(Trait::TAction(TraitAction::Quick), TraitFamily::Processing);
        trait_by_family.insert(
            Trait::TAction(TraitAction::Primary),
            TraitFamily::Processing,
        );
        trait_by_family.insert(
            Trait::TAction(TraitAction::Secondary),
            TraitFamily::Processing,
        );
        trait_by_family.insert(Trait::TAction(TraitAction::Move), TraitFamily::Actuating);
        trait_by_family.insert(Trait::TAction(TraitAction::Attack), TraitFamily::Actuating);
        trait_by_family.insert(Trait::TAction(TraitAction::Defend), TraitFamily::Actuating);
        trait_by_family.insert(
            Trait::TAttribute(TraitAttribute::Hp),
            TraitFamily::Actuating,
        );
        trait_by_family.insert(Trait::TAction(TraitAction::Rest), TraitFamily::Actuating);

        // actual constructor
        GeneLibrary {
            gray_to_trait,
            trait_by_family,
            gray_code: generate_gray_code(4),
            trait_count: traits.len(),
        }
    }

    fn add_gene(&mut self, gene: GeneRecord) {
        debug!("[dna] adding new gene to the library: {:?}", gene);
        // TODO: Redo this. Is this even still necessary?
        // let trait_code = self.gray_code[self.trait_count];
        // self.gray_to_trait.insert(trait_code, gene.name);
        // self.trait_to_super.insert(trait_code, gene.super_trait);
        // self.trait_to_action.insert(trait_code, gene.action);
        // self.trait_count += 1;
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
                error!("[dna] Unable to read gene file!");
            }
        }
    }

    // TODO: Add parameters to control distribution of sense, process and actuate!
    // TODO: Use above parameters for NPC definitions, readable from datafiles!
    pub fn new_dna(&self, game_rng: &mut GameRng, avg_genome_len: usize) -> Vec<u8> {
        let mut dna = Vec::new();
        // randomly grab a trait and add trait id, length and random attribute value
        for _ in 0..avg_genome_len {
            // push 0x00 first as the genome start symbol
            dna.push(0 as u8);
            // pick random trait number from list
            let trait_num = game_rng.gen_range(0, self.trait_count);
            // add trait id
            dna.push(self.gray_code[trait_num]);
            // add length
            dna.push(1);
            // add random attribute value
            dna.push(game_rng.gen_range(0, 16) as u8);
        }
        // debug!("new dna generated: {:?}", dna);
        dna
    }

    pub fn decode_dna(&self, dna: &[u8]) -> (Sensors, Processors, Actuators, Dna) {
        let mut start_ptr: usize = 0;
        let mut end_ptr: usize = dna.len();
        let mut trait_builder: TraitBuilder = TraitBuilder::new();

        while start_ptr < dna.len() - 2 {
            let (s_ptr, e_ptr) = self.decode_gene(dna, start_ptr, end_ptr, &mut trait_builder);
            start_ptr = s_ptr;
            end_ptr = e_ptr;
        }

        // return sensor, processor and actuator instances
        trait_builder.finalize()
    }

    /// Combine *new_dna()* and *decode_dna()* into a single function call.
    pub fn new_genetics(
        &self,
        game_rng: &mut GameRng,
        avg_genome_len: usize,
    ) -> (Sensors, Processors, Actuators, Dna) {
        let dna = self.new_dna(game_rng, avg_genome_len);
        let (s, p, a, mut d) = self.decode_dna(&dna);
        d.raw = dna;
        (s, p, a, d)
    }

    fn decode_gene(
        &self,
        dna: &[u8],
        mut start_ptr: usize,
        mut end_ptr: usize,
        trait_builder: &mut TraitBuilder,
    ) -> (usize, usize) {
        // pointing at 0x00 now
        // println!("start_ptr at 0x00 = {}", start_ptr);
        start_ptr += 1;
        // read length
        // println!("start_ptr at len = {}", start_ptr);
        end_ptr = cmp::min(end_ptr, start_ptr + dna[start_ptr] as usize);
        start_ptr += 1;
        // println!("start_ptr at iteration start = {}", start_ptr);
        // println!("new end_ptr = {}", end_ptr);
        // read trait ids - actions and attributes
        for i in start_ptr..=end_ptr {
            // println!("iteration -> i = {}", i);
            // if we reached the end of the genome, return the current position
            if i >= dna.len() {
                return (i, end_ptr);
            }
            // take u8 word and map it to action/attribute
            match self.gray_to_trait.get(&dna[i]) {
                Some(Trait::TAttribute(attr)) => trait_builder.add_attribute(*attr),
                Some(Trait::TAction(actn)) => trait_builder.add_action(*actn),
                None => {}
            }
        }

        start_ptr = end_ptr + 1;
        end_ptr = dna.len();
        // println!("returning start_ptr {}, end_ptr {}", start_ptr, end_ptr);
        (start_ptr, end_ptr)
    }
}

#[derive(Default)]
struct TraitBuilder {
    sensors: Sensors,
    processors: Processors,
    actuators: Actuators,
    // accumulated traits, mapping trait to count
    sensor_action_acc: HashMap<TraitAction, i32>,
    processor_action_acc: HashMap<TraitAction, i32>,
    actuator_action_acc: HashMap<TraitAction, i32>,
    dna: Dna,
}

impl TraitBuilder {
    pub fn new() -> Self {
        TraitBuilder {
            sensors: Sensors::new(),
            processors: Processors::new(),
            actuators: Actuators::new(),
            sensor_action_acc: HashMap::new(),
            processor_action_acc: HashMap::new(),
            actuator_action_acc: HashMap::new(),
            dna: Dna {
                raw: Vec::new(),
                simplified: Vec::new(),
            },
        }
    }

    pub fn add_attribute(&mut self, attr: TraitAttribute) {
        match attr {
            TraitAttribute::SensingRange => {
                self.dna.simplified.push(TraitFamily::Sensing);
                self.sensors.sense_range += 1;
            }
            TraitAttribute::Hp => {
                self.dna.simplified.push(TraitFamily::Actuating);
                self.actuators.hp += 1;
            }
        }
    }

    pub fn add_action(&mut self, actn: TraitAction) {
        match actn {
            TraitAction::Sense => {
                // increase the counter for the given action or insert a 0 as default value;
                // below is the long form...
                //  let count = self.sensor_action_acc.entry(actn).or_insert(0);
                //  *count += 1;
                // ... which shortens to the following:
                self.dna.simplified.push(TraitFamily::Sensing);
                *self.sensor_action_acc.entry(actn).or_insert(0) += 1;
            }
            TraitAction::Primary | TraitAction::Secondary | TraitAction::Quick => {
                self.dna.simplified.push(TraitFamily::Processing);
                *self.processor_action_acc.entry(actn).or_insert(0) += 1;
            }
            TraitAction::Move | TraitAction::Attack | TraitAction::Defend | TraitAction::Rest => {
                self.dna.simplified.push(TraitFamily::Actuating);
                *self.actuator_action_acc.entry(actn).or_insert(0) += 1;
            }
        }
    }

    // Finalize all actions, return the super trait components and consume itself.
    //
    pub fn finalize(mut self) -> (Sensors, Processors, Actuators, Dna) {
        // instantiate an action or prototype with count as additional parameter
        self.sensors.actions = self
            .sensor_action_acc
            .iter()
            .map(|(trait_id, parameter)| ActionPrototype {
                trait_id: *trait_id,
                parameter: *parameter,
            })
            .collect();

        self.processors.actions = self
            .processor_action_acc
            .iter()
            .map(|(trait_id, parameter)| ActionPrototype {
                trait_id: *trait_id,
                parameter: *parameter,
            })
            .collect();

        self.actuators.actions = self
            .actuator_action_acc
            .iter()
            .map(|(trait_id, parameter)| ActionPrototype {
                trait_id: *trait_id,
                parameter: *parameter,
            })
            .collect();

        (self.sensors, self.processors, self.actuators, self.dna)
    }
}
