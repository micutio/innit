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
//! +------+---------------+------------+
//! | 0x00 | genome length | trait name |
//! +------+---------------+------------+
//!
//!
//! A DNA Genome is implemented as a string of hexadecimal numbers. The start of a gene is marked
//! by the number zero. Genes can overlap, so that parsing the new gene resumes "in the middle" of
//! a previous gene. The genes should be small and encoding the presence of a quality. Attributes or
//! versatility is then controlled by the cumulative occurrence of a gene.
//! Basically: the more often a gene occurs, the stronger its trait will be.
//! Things to think about:
//! - How to handle synergies/anti-synergies?
//! - How to calculate energy cost per action?
//! - Can behavior be encoded in the genome too i.e., fight or flight?
//! - Should attributes be fix on trait level or full-on generic as list of attribute objects?
//! - How to best model synergies and anti-synergies across traits?

use crate::entity::action::hereditary::{
    ActAttack, ActBinaryFission, ActKillSwitch, ActMetabolise, ActMove,
};
use crate::entity::action::inventory::ActPickUpItem;
use crate::entity::action::Action;
use crate::entity::genetics::DnaType::Nucleoid;
use crate::util::game_rng::GameRng;
use crate::util::generate_gray_code;
use core::fmt;
use rand::distributions::WeightedIndex;
use rand::prelude::{Distribution, SliceRandom};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::cmp;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

pub const GENE_LEN: usize = 30;

/// All traits belong to one of three major categories, called trait families.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum TraitFamily {
    Sensing,
    Processing,
    Actuating,
    Ltr,
    Junk(u8),
    // We want to be able to decode/encode genome and trait back and forth which requires junk to
    // keep track of the gene that caused it because it's not unique.
}

impl Display for TraitFamily {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TraitFamily::Sensing => write!(f, "Sense"),
            TraitFamily::Processing => write!(f, "Process"),
            TraitFamily::Actuating => write!(f, "Actuate"),
            TraitFamily::Ltr => write!(f, "Ltr"),
            TraitFamily::Junk(_) => write!(f, "Junk"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum TraitAttribute {
    SensingRange,
    Hp,
    Volume,
    Metabolism,
    Storage,
    Receptor,
    LifeExpectancy,
    None,
}

/// Genetic traits are linked to actions and attributes.
/// Actions are supposed to be linked to key inputs.
/// Relationships:
///      - an attribute can be influenced by multiple traits
///      - an action can be influenced by multiple traits
///      - traits need to know how often they appear in the genome
///      - attributes and actions need to know this too!
///
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct GeneticTrait {
    pub trait_name: String,
    pub trait_family: TraitFamily,
    pub attribute: TraitAttribute,       // Vec<TraitAttribute>
    pub action: Option<Box<dyn Action>>, // TraitActions
    pub position: u32,                   // position of the gene within the genome
}

impl GeneticTrait {
    fn new(
        name: &str,
        trait_family: TraitFamily,
        attribute: TraitAttribute,
        action: Option<Box<dyn Action>>,
    ) -> Self {
        GeneticTrait {
            trait_name: name.to_string(),
            trait_family,
            attribute,
            action,
            position: 0,
        }
    }

    fn junk(value: u8) -> Self {
        GeneticTrait {
            trait_name: "Junk".to_string(),
            trait_family: TraitFamily::Junk(value),
            attribute: TraitAttribute::None,
            action: None,
            position: 0,
        }
    }
}

fn create_trait_list() -> Vec<GeneticTrait> {
    // use TraitAttribute::*;
    use TraitFamily::*;
    vec![
        GeneticTrait::new(
            "Move",
            Actuating,
            TraitAttribute::None,
            Some(Box::new(ActMove::new())),
        ),
        GeneticTrait::new(
            "Attack",
            Actuating,
            TraitAttribute::None,
            Some(Box::new(ActAttack::new())),
        ),
        GeneticTrait::new(
            "Binary Fission",
            TraitFamily::Actuating,
            TraitAttribute::None,
            Some(Box::new(ActBinaryFission::new())),
        ),
        GeneticTrait::new("Cell Membrane", Actuating, TraitAttribute::Hp, None),
        GeneticTrait::new("Cell Volume", Actuating, TraitAttribute::Volume, None),
        GeneticTrait::new(
            "Life Expectancy",
            Processing,
            TraitAttribute::LifeExpectancy,
            None,
        ),
        GeneticTrait::new(
            "Optical Sensor",
            Sensing,
            TraitAttribute::SensingRange,
            None,
        ),
        // enzymes are stand-ins for metabolism for now
        // TODO: separate into catabolism and anabolism
        GeneticTrait::new("Enzyme", Processing, TraitAttribute::Metabolism, None),
        GeneticTrait::new("Energy Store", Processing, TraitAttribute::None, None),
        GeneticTrait::new(
            "Metabolism",
            Processing,
            TraitAttribute::Storage,
            Some(Box::new(ActMetabolise::new())),
        ),
        GeneticTrait::new("Receptor", Processing, TraitAttribute::Receptor, None),
        GeneticTrait::new(
            "Kill Switch",
            TraitFamily::Processing,
            TraitAttribute::None,
            Some(Box::new(ActKillSwitch::new())),
        ),
        GeneticTrait::new("LTR marker", TraitFamily::Ltr, TraitAttribute::None, None),
    ]
}

/// This may or may not be body parts. Actuators like organelles can also benefit the attributes.
/// Sensors contain:
/// - attributes
///   - range of effective sensing
///   - accuracy of sensing [future feature]
/// - functions:
///   - sense environment
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[derive(Debug, Default)] //, PartialEq)]
pub struct Sensors {
    pub actions: Vec<Box<dyn Action>>,
    pub sensing_range: i32,
}

impl Sensors {
    pub fn new() -> Self {
        Sensors {
            actions: Vec::new(),
            sensing_range: 1,
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
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[derive(Debug, Default)] //, PartialEq)]
pub struct Processors {
    pub actions: Vec<Box<dyn Action>>,
    pub metabolism: i32,     // energy production per turn
    pub energy_storage: i32, // maximum energy store
    pub energy: i32,
    pub life_expectancy: i32, // total life time, given in turns
    pub life_elapsed: i32,    // life time already past, given in turns
    pub receptors: Vec<Receptor>,
}

impl Processors {
    pub fn new() -> Self {
        Processors {
            actions: Vec::new(),
            metabolism: 1,
            energy_storage: 1,
            energy: 0,
            life_expectancy: 45,
            life_elapsed: 0,
            receptors: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Receptor {
    pub typ: u32,
}

/// Actuators can actually be concrete body parts e.g., organelles, spikes
/// Actuators contain:
/// - attributes:
///   - speed, a modifier of the energy cost of the functions
/// - functions:
///   - move
///   - attack
///   - defend
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[derive(Debug, Default)] //, PartialEq)]
pub struct Actuators {
    pub actions: Vec<Box<dyn Action>>,
    pub max_hp: i32,
    pub hp: i32,
    pub volume: i32,
}

impl Actuators {
    pub fn new() -> Self {
        Actuators {
            actions: Vec::new(),
            max_hp: 1,
            hp: 1,
            volume: 5,
        }
    }
}

// /// Gene Records hold all necessary information for a single gene.
// /// Genes can either encode actions, attributes or both.
// #[derive(Serialize, Deserialize, Debug)]
// pub struct GeneRecord {
//     name: String,
//     super_trait: TraitFamily,
//     action: TraitAction,
//     /* attributes: Vec<?>,
//      * synergies: Vec<?>,
//      * anti-synergies: Vec<?>, */
// }

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum DnaType {
    Nucleus,  // eukaryotic cells
    Nucleoid, // bacteria or very large viruses
    Rna,      // most viruses
    Plasmid,  // plasmids (duh...!)
}

impl Default for DnaType {
    fn default() -> Self {
        DnaType::Nucleoid
    }
}

/// DNA encodes all properties and actions available to an object.
/// For now objects hold DNA either contained in an organelle (Nucleus), free floating in the cell
/// (Nucleoid) or in form of a ring structure that can be exchanged or picked up by certain other
/// objects (Plasmid). This is indicated by the `dna_type`.
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[derive(Debug, Default, Clone)]
pub struct Dna {
    pub dna_type: DnaType,
    pub raw: Vec<u8>,
    pub simplified: Vec<GeneticTrait>,
}

impl Dna {
    pub fn new() -> Dna {
        Dna {
            dna_type: Nucleoid,
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
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[derive(Debug, Default)]
pub struct GeneLibrary {
    /// Traits are now supposed to be generic, so enums are no longer the way to go.
    trait_vec: Vec<GeneticTrait>,
    /// Traits are encoded in gray code.
    gray_to_trait: HashMap<u8, String>,
    /// Reverse mapping for encoding traits into dna
    trait_to_gray: HashMap<String, u8>,
    /// Vector of gray code with index corresponding to its binary representation
    gray_code: Vec<u8>,
    /// Count the number of traits we have, sort of as a running id.
    trait_count: usize,
}

impl GeneLibrary {
    pub fn new() -> Self {
        let trait_vec: Vec<GeneticTrait> = create_trait_list();
        let trait_count = trait_vec.len();
        let gray_code = generate_gray_code(4);
        let gray_to_trait: HashMap<u8, String> = trait_vec
            .iter()
            .enumerate()
            .map(|(code, gene_trait)| (gray_code[code + 1], gene_trait.trait_name.clone()))
            .collect();
        debug!("gray to trait map: {:#?}", gray_to_trait);
        let trait_to_gray: HashMap<String, u8> = trait_vec
            .iter()
            .enumerate()
            .map(|(code, gene_trait)| (gene_trait.trait_name.clone(), gray_code[code + 1]))
            .collect();
        debug!("trait to gray map: {:#?}", trait_to_gray);
        // actual constructor
        GeneLibrary {
            trait_vec,
            gray_to_trait,
            trait_to_gray,
            gray_code,
            trait_count,
        }
    }

    /// Generate a new random binary DNA code from a given length and possibly with LTR markers.
    pub fn new_dna(&self, rng: &mut GameRng, has_ltr: bool, avg_genome_len: usize) -> Vec<u8> {
        let mut dna: Vec<u8> = Vec::new();

        if has_ltr {
            if let Some(ltr_code) = self.trait_to_gray.get("LTR marker") {
                dna.push(0 as u8);
                dna.push(1 as u8);
                dna.push(*ltr_code);
            }
        }

        // randomly grab a trait and add trait id, length and random attribute value
        for _ in 0..avg_genome_len {
            // push 0x00 first as the genome start symbol
            dna.push(0 as u8);
            // add length
            dna.push(1 as u8);
            // pick random trait number from list and add trait id
            let trait_num = rng.gen_range(1..self.trait_count);
            trace!(
                "sampled genetic trait {} ({})",
                trait_num,
                self.gray_code[trait_num]
            );
            dna.push(self.gray_code[trait_num] as u8);
            //
            // // add random attribute value
            // dna.push(game_rng.gen_range(0, 16) as u8);
        }

        if has_ltr {
            if let Some(ltr_code) = self.trait_to_gray.get("LTR Marker") {
                dna.push(0 as u8);
                dna.push(1 as u8);
                dna.push(*ltr_code);
            }
        }

        // debug!("new dna generated: {:?}", dna);
        dna
    }

    /// Encode a slice of genetic trait objects into binary DNA code.
    pub fn g_traits_to_dna(&self, traits: &[GeneticTrait]) -> Vec<u8> {
        let mut dna: Vec<u8> = Vec::new();
        for t in traits {
            // push 0x00 first as the genome start symbol
            dna.push(0);
            // add length
            dna.push(1);
            if let Some(gray) = self.trait_to_gray.get(&t.trait_name) {
                dna.push(*gray);
            } else {
                if let TraitFamily::Junk(value) = t.trait_family {
                    dna.push(value);
                    // Don't do random anymore
                    // let defined_range = self.trait_to_gray.len() as u8;
                    // dna.push(rng.gen_range(defined_range..=255));
                } else {
                    panic!(
                        "unknown genetic trait: {} , {}",
                        t.trait_name, t.trait_family
                    );
                }
            }
            //
            // // add random attribute value
            // dna.push(game_rng.gen_range(0, 16) as u8);
        }
        // debug!("new dna generated: {:?}", dna);
        dna
    }

    /// Encode a slice of genetic trait object references into binary DNA code.
    pub fn g_trait_refs_to_dna(&self, traits: &[&GeneticTrait]) -> Vec<u8> {
        let mut dna: Vec<u8> = Vec::new();
        for t in traits {
            // push 0x00 first as the genome start symbol
            dna.push(0);
            // add length
            dna.push(1);
            if let Some(gray) = self.trait_to_gray.get(&t.trait_name) {
                dna.push(*gray);
            } else {
                if let TraitFamily::Junk(value) = t.trait_family {
                    dna.push(value);
                    // Don't do random anymore
                    // let defined_range = self.trait_to_gray.len() as u8;
                    // dna.push(rng.gen_range(defined_range..=255));
                } else {
                    panic!(
                        "unknown genetic trait: {} , {}",
                        t.trait_name, t.trait_family
                    );
                }
            }
            //
            // // add random attribute value
            // dna.push(game_rng.gen_range(0, 16) as u8);
        }
        // debug!("new dna generated: {:?}", dna);
        dna
    }

    /// Encode a vector of genetic trait names into binary DNA code.
    pub fn trait_strs_to_dna(&self, rng: &mut GameRng, traits: &[String]) -> Vec<u8> {
        let mut dna: Vec<u8> = Vec::new();
        for t in traits {
            // push 0x00 first as the genome start symbol
            dna.push(0);
            // add length
            dna.push(1);
            if let Some(gray) = self.trait_to_gray.get(t) {
                dna.push(*gray);
            } else {
                let defined_range = self.trait_to_gray.len() as u8;
                dna.push(rng.gen_range(defined_range..=255));
                panic!("unknown genetic trait: {}", t);
            }
            //
            // // add random attribute value
            // dna.push(game_rng.gen_range(0, 16) as u8);
        }
        // debug!("new dna generated: {:?}", dna);
        dna
    }

    /// Generate a new binary DNA code from a given weighted distribution of trait families.
    pub fn dna_from_distribution(
        &self,
        rng: &mut GameRng,
        weights: &[u8],
        choices: &[TraitFamily],
        has_ltr: bool,
        genome_len: usize,
    ) -> Vec<u8> {
        let mut dna: Vec<u8> = Vec::new();
        let mut start_idx = 0;
        let mut end_idx = genome_len;
        if has_ltr {
            start_idx = 1;
            dna.push(*self.trait_to_gray.get("LTR marker").unwrap());
            end_idx = genome_len - 1;
        }
        let gene_dist: WeightedIndex<u8> = WeightedIndex::new(weights).unwrap();
        let mut traits_map: HashMap<TraitFamily, Vec<&GeneticTrait>> = HashMap::new();
        choices.iter().for_each(|x| {
            traits_map.insert(
                *x,
                self.trait_vec
                    .iter()
                    .filter(|t| t.trait_family == *x)
                    .collect(),
            );
        });
        for _ in start_idx..end_idx {
            let chosen_trait = choices[gene_dist.sample(rng)];
            let gene = traits_map.get(&chosen_trait).unwrap().choose(rng).unwrap();
            dna.push(*self.trait_to_gray.get(&gene.trait_name).unwrap());
        }
        if has_ltr {
            dna.push(*self.trait_to_gray.get("LTR marker").unwrap());
        }
        dna
    }

    /// Decode DNA from binary representation into genetic trait objects
    pub fn dna_to_traits(
        &self,
        dna_type: DnaType,
        raw_dna: &[u8],
    ) -> (Sensors, Processors, Actuators, Dna) {
        assert!(!raw_dna.is_empty());
        let mut start_ptr: usize = 0;
        let mut end_ptr: usize = raw_dna.len();
        let mut trait_builder: TraitBuilder = TraitBuilder::new(dna_type, raw_dna);
        let mut position: u32 = 0;

        while start_ptr < raw_dna.len() - 2 {
            let (s_ptr, e_ptr) =
                self.decode_gene(raw_dna, start_ptr, end_ptr, position, &mut trait_builder);
            start_ptr = s_ptr;
            end_ptr = e_ptr;
            position += 1;
        }

        // return sensor, processor and actuator instances
        trait_builder.finalize(&self.trait_vec)
    }

    /// Combine *new_dna()* and *decode_dna()* into a single function call.
    pub fn new_genetics(
        &self,
        rng: &mut GameRng,
        dna_type: DnaType,
        has_ltr: bool,
        avg_genome_len: usize,
    ) -> (Sensors, Processors, Actuators, Dna) {
        let dna = self.new_dna(rng, has_ltr, avg_genome_len);
        let (s, p, a, mut d) = self.dna_to_traits(dna_type, &dna);
        d.raw = dna;
        (s, p, a, d)
    }

    /// Decodes one complete gene from the bit vector, starting at `start_ptr`.
    /// Returns the new positions for `start_ptr` and `end_ptr` after decoding is done.
    fn decode_gene(
        &self,
        dna: &[u8],
        mut start_ptr: usize,
        mut end_ptr: usize,
        position: u32,
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
            // match self.gray_to_trait.get(&dna[i]) {
            //     Some(Trait::TAttribute(attr)) => trait_builder.add_attribute(*attr),
            //     Some(Trait::TAction(actn)) => trait_builder.add_action(*actn),
            //     None => {}
            // }
            if let Some(trait_name) = self.gray_to_trait.get(&dna[i]) {
                // println!("gtt[{} (dna[{}])] -> {}", &dna[i], i, trait_name);
                if let Some(genetic_trait) = self
                    .trait_vec
                    .iter()
                    .find(|gt| gt.trait_name.eq(trait_name))
                {
                    // trace!("found genetic trait {}", genetic_trait.trait_name);
                    let mut this_trait = genetic_trait.clone();
                    this_trait.position = position;
                    trait_builder.add_action(&this_trait);
                    trait_builder.add_attribute(&this_trait);
                    trait_builder.record_trait(this_trait);
                } else {
                    error!("no trait for id {}", trait_name);
                }
            } else {
                trait_builder.record_trait(GeneticTrait::junk(dna[i]));
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
    sensor_action_count: HashMap<String, i32>,
    processor_action_count: HashMap<String, i32>,
    actuator_action_count: HashMap<String, i32>,
    dna: Dna,
}

impl TraitBuilder {
    pub fn new(dna_type: DnaType, raw_dna: &[u8]) -> Self {
        assert!(!raw_dna.is_empty());
        TraitBuilder {
            sensors: Sensors::new(),
            processors: Processors::new(),
            actuators: Actuators::new(),
            sensor_action_count: HashMap::new(),
            processor_action_count: HashMap::new(),
            actuator_action_count: HashMap::new(),
            dna: Dna {
                dna_type,
                raw: raw_dna.to_vec(),
                simplified: Vec::new(),
            },
        }
    }

    pub fn add_attribute(&mut self, g_trait: &GeneticTrait) {
        match g_trait.attribute {
            TraitAttribute::SensingRange => {
                self.sensors.sensing_range += 1;
            }
            TraitAttribute::Hp => {
                self.actuators.max_hp += 1;
                self.actuators.hp += 1;
            }
            TraitAttribute::Volume => {
                self.actuators.volume += 1;
            }
            TraitAttribute::Metabolism => {
                self.processors.metabolism += 1;
            }
            TraitAttribute::Storage => {
                self.processors.energy_storage += 1;
            }
            TraitAttribute::Receptor => {
                self.processors.receptors.push(Receptor {
                    typ: g_trait.position,
                });
            }
            TraitAttribute::LifeExpectancy => {
                self.processors.life_expectancy += 50;
            }
            TraitAttribute::None => {}
        }
    }

    pub fn add_action(&mut self, genetic_trait: &GeneticTrait) {
        match genetic_trait.trait_family {
            TraitFamily::Actuating => {
                // increase the counter for the given action or insert a 0 as default value;
                // below is the long form...
                //  let count = self.sensor_action_acc.entry(actn).or_insert(0);
                //  *count += 1;
                // ... which shortens to the following:
                *self
                    .actuator_action_count
                    .entry(genetic_trait.trait_name.clone())
                    .or_insert(0) += 1;
            }
            TraitFamily::Sensing => {
                *self
                    .sensor_action_count
                    .entry(genetic_trait.trait_name.clone())
                    .or_insert(0) += 1;
            }
            TraitFamily::Processing => {
                *self
                    .processor_action_count
                    .entry(genetic_trait.trait_name.clone())
                    .or_insert(0) += 1;
            }
            TraitFamily::Ltr => {}
            TraitFamily::Junk(_) => {}
        }
    }

    pub fn record_trait(&mut self, g_trait: GeneticTrait) {
        self.dna.simplified.push(g_trait);
    }

    // Finalize all actions, return the super trait components and consume itself.
    //
    pub fn finalize(mut self, trait_vec: &[GeneticTrait]) -> (Sensors, Processors, Actuators, Dna) {
        // instantiate an action or prototype with count as additional parameter
        self.sensors.actions = self
            .sensor_action_count
            .iter()
            .map(|(trait_name, parameter)| {
                let genetic_trait = trait_vec
                    .iter()
                    .find(|gt| gt.trait_name.eq(trait_name))
                    .unwrap();
                if let Some(a) = &genetic_trait.action {
                    let mut boxed_action = a.clone_action();
                    boxed_action.set_level(*parameter);
                    Some(boxed_action)
                } else {
                    None
                }
            })
            .filter_map(|o| o)
            .collect();

        self.processors.actions = self
            .processor_action_count
            .iter()
            .map(|(trait_name, parameter)| {
                let genetic_trait = trait_vec
                    .iter()
                    .find(|gt| gt.trait_name.eq(trait_name))
                    .unwrap();
                if let Some(a) = &genetic_trait.action {
                    let mut boxed_action = a.clone_action();
                    boxed_action.set_level(*parameter);
                    Some(boxed_action)
                } else {
                    None
                }
            })
            .filter_map(|o| o)
            .collect();

        self.actuators.actions = self
            .actuator_action_count
            .iter()
            .map(|(trait_name, parameter)| {
                let genetic_trait = trait_vec
                    .iter()
                    .find(|gt| gt.trait_name.eq(trait_name))
                    .unwrap();
                if let Some(a) = &genetic_trait.action {
                    let mut boxed_action = a.clone_action();
                    boxed_action.set_level(*parameter);
                    Some(boxed_action)
                } else {
                    None
                }
            })
            .filter_map(|o| o)
            .collect();

        // Space for 'post-processing'
        // Add equip function for anything but viruses and plasmids
        if matches!(self.dna.dna_type, DnaType::Nucleoid)
            || matches!(self.dna.dna_type, DnaType::Nucleus)
        {
            self.actuators.actions.push(Box::new(ActPickUpItem))
        }

        (self.sensors, self.processors, self.actuators, self.dna)
    }
}
