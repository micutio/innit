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
pub enum TraitID {
    Sense,
    QuickAction,
    PrimaryAction,
    SecondaryAction,
    Move,
    Attack,
    Defend,
    Rest,
}

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct ActionPrototype {
    pub super_trait: SuperTrait,
    pub trait_id:    TraitID,
    pub name:        String,
    pub parameter:   i32,
}

// TODO: Add parameters to control distribution of sense, process and actuate!
// TODO: Use above parameters for NPC definitions, readable from datafiles!
#[derive(PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct DnaGenerator {
    traits:    Vec<TraitID>,
    gray_code: Vec<u8>,
    avg_len:   u32,
}

impl DnaGenerator {
    pub fn new() -> Self {
        use self::TraitID::*;
        let traits = vec![
            Sense,
            QuickAction,
            PrimaryAction,
            SecondaryAction,
            Move,
            Attack,
            Defend,
            Rest,
        ];
        let traits_len = traits.len();
        DnaGenerator {
            traits,
            gray_code: generate_gray_code(traits_len as u8),
            avg_len: 10,
        }
    }

    pub fn new_dna(&self, game_rng: &mut GameRng) -> Vec<char> {
        let mut dna: Vec<char> = vec![];
        // randomly grab a trait and add trait id, length and random attribute value
        for _ in 0..10 {
            // push 0x00 first as the genome start symbol
            dna.push(0 as u8 as char);
            // pick random trait number from list
            let trait_num = game_rng.gen_range(0, self.traits.len());
            // add trait id
            dna.push(self.gray_code[trait_num] as char);
            // add length // TODO: encode length in TraitID
            dna.push(1 as char);
            // add random attribute value
            dna.push(game_rng.gen_range(0, 16) as u8 as char);
        }
        debug!("new dna generated: {:?}", dna);
        dna
    }
}

/// Construct a new player action from a given key code.
/// Get player's action item that corresponds with the player input and construct a new action
/// from the parameters in both
// NOTE: In the future we'll have to consider mouse clicks as well.
pub fn get_player_action(input: PlayAction, prototype: &ActionPrototype) -> Box<dyn Action> {
    use self::TraitID::*;
    use ui::game_input::PlayActionParameter::*;
    match input {
        PlayAction {
            trait_id: Move,
            param: Orientation(dir),
        } => Box::new(MoveAction::new(dir, prototype.parameter)),
        // TODO: Check if we can actually move!
        // (PlayInput(Move(TraitID::Move, Cardinal(Direction))), Some(action_prototype)) =>
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
#[derive(Debug, Serialize, Deserialize)]
pub struct Sensor {
    actions: Vec<ActionPrototype>,
}

/// Processors contain:
/// - attributes:
///   - capacity, a quantization/modifier of how energy-costly and complex the functions are
/// - functions:
///   - setting of primary/secondary actions [player]
///   - decision making algorithm [player/ai]
///   - ai control [ai]
#[derive(Debug, Serialize, Deserialize)]
pub struct Processor {
    actions: Vec<ActionPrototype>,
}

/// Actuators can actually be concrete body parts e.g., organelles, spikes
/// Actuators contain:
/// - attributes:
///   - speed, a modifier of the energy cost of the functions
/// - functions:
///   - move
///   - attack
///   - defend
#[derive(Debug, Serialize, Deserialize)]
pub struct Actuator {
    actions: Vec<ActionPrototype>,
}
