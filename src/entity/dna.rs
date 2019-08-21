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
    Sensing,
    Processing,
    Actuating,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum SubTrait {
    StAttribute(TraitAttribute),
    StAction(TraitAction),
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum TraitAttribute {
    SensingRange,
    Hp,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum TraitAction {
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
    pub trait_id:  TraitAction,
    pub parameter: i32,
}

/// Construct a new player action from a given key code.
/// Get player's action item that corresponds with the player input and construct a new action
/// from the parameters in both
// NOTE: In the future we'll have to consider mouse clicks as well.
pub fn get_player_action(input: PlayAction, prototype: &ActionPrototype) -> Box<dyn Action> {
    use self::TraitAction::*;
    use ui::game_input::PlayActionParameter::*;
    match input {
        PlayAction {
            trait_id: Move,
            param: Orientation(dir),
        } => Box::new(MoveAction::new(dir, prototype.parameter)),
        // TODO: Check if we can actually move!
        // (PlayInput(Move(TraitAction::Move, Cardinal(Direction))), Some(action_prototype)) =>
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
    actions:     Vec<ActionPrototype>,
    sense_range: i32,
    // attributes: Vec<AttributeObject>,
}

impl Sensor {
    pub fn new() -> Self {
        Sensor {
            actions:     Vec::new(),
            sense_range: 0,
        }
    }

    // TODO: Implement creating action(prototypes) from given action traits and prototypes.
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
    hp:      i32,
}

impl Actuator {
    pub fn new() -> Self {
        Actuator {
            actions: Vec::new(),
            hp:      0,
        }
    }
}

/// Gene Records hold all necessary information for a single gene.
/// Genes can either encode actions, attributes or both.
#[derive(Serialize, Deserialize, Debug)]
pub struct GeneRecord {
    name:        String,
    super_trait: SuperTrait,
    action:      TraitAction,
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
    // TODO: Re-use enum TraitAction to identify actions instead. They are basically already doing
    // it.
    gray_to_trait: HashMap<u8, SubTrait>,
    /// This one should be straight forward. Lets the custom traits make use of supertrait specific
    /// attributes.
    trait_to_super: HashMap<SubTrait, SuperTrait>,
    /// As mentioned above, re-use TraitIDs to allow mappings to actions.
    // trait_to_action: HashMap<u8, TraitAction>,
    /// Vector of gray code with index corresponding to its binary representation
    gray_code: Vec<u8>,
    /// Count the number of traits we have, sort of as a running id.
    trait_count: usize,
}

impl GeneLibrary {
    pub fn new() -> Self {
        let traits: Vec<SubTrait> = vec![
            SubTrait::StAttribute(TraitAttribute::SensingRange),
            SubTrait::StAction(TraitAction::Sense),
            SubTrait::StAction(TraitAction::Quick),
            SubTrait::StAction(TraitAction::Primary),
            SubTrait::StAction(TraitAction::Secondary),
            SubTrait::StAction(TraitAction::Move),
            SubTrait::StAction(TraitAction::Attack),
            SubTrait::StAction(TraitAction::Defend),
            SubTrait::StAttribute(TraitAttribute::Hp),
            SubTrait::StAction(TraitAction::Rest),
        ];
        // TODO: Introduce constant N for total number of traits to assert gray code vector length.
        let gray_code = generate_gray_code(4);
        let gray_to_trait: HashMap<u8, SubTrait> = traits
            .iter()
            .enumerate()
            .map(|(x, y)| (gray_code[x], *y))
            .collect();

        // TODO: This is really unwieldy.
        // At least extract to it's own function or better yet, try to make it more generic.
        let mut trait_to_super: HashMap<SubTrait, SuperTrait> = HashMap::new();
        trait_to_super.insert(
            SubTrait::StAttribute(TraitAttribute::SensingRange),
            SuperTrait::Sensing,
        );
        trait_to_super.insert(SubTrait::StAction(TraitAction::Sense), SuperTrait::Sensing);
        trait_to_super.insert(
            SubTrait::StAction(TraitAction::Quick),
            SuperTrait::Processing,
        );
        trait_to_super.insert(
            SubTrait::StAction(TraitAction::Primary),
            SuperTrait::Processing,
        );
        trait_to_super.insert(
            SubTrait::StAction(TraitAction::Secondary),
            SuperTrait::Processing,
        );
        trait_to_super.insert(SubTrait::StAction(TraitAction::Move), SuperTrait::Actuating);
        trait_to_super.insert(
            SubTrait::StAction(TraitAction::Attack),
            SuperTrait::Actuating,
        );
        trait_to_super.insert(
            SubTrait::StAction(TraitAction::Defend),
            SuperTrait::Actuating,
        );
        trait_to_super.insert(
            SubTrait::StAttribute(TraitAttribute::Hp),
            SuperTrait::Actuating,
        );
        trait_to_super.insert(SubTrait::StAction(TraitAction::Rest), SuperTrait::Actuating);

        // actual constructor
        GeneLibrary {
            gray_to_trait,
            trait_to_super,
            gray_code: generate_gray_code(4),
            trait_count: 0,
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
                error!("[dna] Enable to read gene file!");
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
            // add length // TODO: encode length in TraitAction
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
        let mut trait_builder: TraitBuilder = TraitBuilder::new();

        while start_ptr < dna.len() {
            let (s_ptr, e_ptr) = self.decode_gene(dna, start_ptr, end_ptr, &mut trait_builder);
            start_ptr = s_ptr;
            end_ptr = e_ptr;
        }

        // return sensor, processor and actuator instances
        trait_builder.finalize()
    }

    fn decode_gene(
        &self,
        dna: &[u8],
        mut start_ptr: usize,
        mut end_ptr: usize,
        trait_builder: &mut TraitBuilder,
    ) -> (usize, usize) {
        // pointing at 0x00 now
        start_ptr += 1;
        // read length
        end_ptr = cmp::min(end_ptr, start_ptr + dna[start_ptr] as usize);
        start_ptr += 1;
        // read trait ids - actions and attributes
        for i in start_ptr..end_ptr {
            // if we reached the end of the genome, return the current position
            if i >= dna.len() {
                return (i, end_ptr);
            }
            // take u8 word and map it to action/attribute
            match self.gray_to_trait.get(&dna[i]) {
                Some(SubTrait::StAttribute(attr)) => trait_builder.add_attribute(*attr),
                Some(SubTrait::StAction(actn)) => trait_builder.add_action(*actn),
                None => {}
            }
        }

        (start_ptr, end_ptr)
    }
}

#[derive(Default)]
struct TraitBuilder {
    sensor:               Sensor,
    processor:            Processor,
    actuator:             Actuator,
    sensor_action_acc:    HashMap<TraitAction, i32>,
    processor_action_acc: HashMap<TraitAction, i32>,
    actuator_action_acc:  HashMap<TraitAction, i32>,
}

impl TraitBuilder {
    pub fn new() -> Self {
        TraitBuilder {
            sensor:               Sensor::new(),
            processor:            Processor::new(),
            actuator:             Actuator::new(),
            sensor_action_acc:    HashMap::new(),
            processor_action_acc: HashMap::new(),
            actuator_action_acc:  HashMap::new(),
        }
    }

    pub fn add_attribute(&mut self, attr: TraitAttribute) {
        match attr {
            TraitAttribute::SensingRange => self.sensor.sense_range += 1,
            TraitAttribute::Hp => self.actuator.hp += 1,
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
                *self.sensor_action_acc.entry(actn).or_insert(0) += 1;
            }
            TraitAction::Primary | TraitAction::Secondary | TraitAction::Quick => {
                *self.processor_action_acc.entry(actn).or_insert(0) += 1;
            }
            TraitAction::Move | TraitAction::Attack | TraitAction::Defend | TraitAction::Rest => {
                *self.actuator_action_acc.entry(actn).or_insert(0) += 1;
            }
        }
    }

    // Finalize all actions, return the super trait components and consume itself.
    pub fn finalize(mut self) -> (Sensor, Processor, Actuator) {
        // TODO: count occurences of all actions in sensor, processor and actuator action lists
        // instantiate an action or prototype with count as additional parameter
        self.sensor.actions = self
            .sensor_action_acc
            .iter()
            .map(|(trait_id, parameter)| ActionPrototype {
                trait_id:  *trait_id,
                parameter: *parameter,
            })
            .collect();

        self.processor.actions = self
            .processor_action_acc
            .iter()
            .map(|(trait_id, parameter)| ActionPrototype {
                trait_id:  *trait_id,
                parameter: *parameter,
            })
            .collect();

        self.actuator.actions = self
            .actuator_action_acc
            .iter()
            .map(|(trait_id, parameter)| ActionPrototype {
                trait_id:  *trait_id,
                parameter: *parameter,
            })
            .collect();

        (self.sensor, self.processor, self.actuator)
    }
}
