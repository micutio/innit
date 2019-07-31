use crate::entity::action::*;
use crate::ui::game_input::PlayerInput;

/// The DNA contains all core information, excluding temporary info such as position etc. This
/// module allows to generate objects from DNA and modify them using mutation as well as crossing.
/// Decoding DNA delivers attributes and functions that fall into one of three gene types.
///
/// ## Gene Types
///
/// * sensor - gathering information of the environment
/// * processor - decision making
/// * actuator - interacting with other objects and the game world
///
/// ## Shape of the DNA
///
/// +------+--------------+---------------+-------------+
/// | 0x00 | gene type ID | genome length | trait genes |
/// +------+--------------+---------------+-------------+
///
/// ### sensor
///
/// #### Qualities
///
/// | Trait   | ID   | Attributes |
/// | ------- | ---- | ---------- |
/// | sensor  | 0x01 | range      |
///
/// ### processor
///
/// #### Qualities
///
/// | Trait         | ID   | Attributes |
/// | ------------- | ---- | ---------- |
/// | quick action  | 0x02 | count      |
///
/// ### actuator
///
/// #### Qualities
///
/// | Trait   | ID   | Attributes |
/// | ------- | ---- | ---------- |
/// | move    | 0x03 | speed      |
/// | attack  | 0x04 | damage     |
/// | defend  | 0x05 | health     |
/// | rest    | 0x06 | HP regen   |
///
/// A DNA Genome is implemented as a string of hexadecimal numbers. The start of a gene is marked
/// by the number zero. Genes can overlap, so that parsing the new gene resumes "in the middle" of
/// a previous gene. The genes should be small and encoding the presence of a quality. Attributes or
/// versatility is then controlled by the cumulative occurrence of a gene.
/// Basically: the more often a gene occurs, the stronger its trait will be.
// TODO: How to handle synergies/anti-synergies?
// TODO: How to calculate energy cost per action?
// TODO: Design a DNA parser and a mapping from symbol to trait struct.
// TODO: Can behavior be encoded in the genome too i.e., fight or flight?
pub struct DNA {
    sequence: [char],
}

// TODO: Maybe do away with type IDs and just have one long running list of genes.
// const START: u8 = 0x00;
// const TYPE_SENSOR: u8 = 0x01;
// const TYPE_PROCESSOR: u8 = 0x02;
// const TYPE_ACTUATOR: u8 = 0x03;

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
enum ActionItem {
    Sense {
        name:  String,
        range: i32,
    },
    QuickAction {
        name:  String,
        count: i32,
    },
    Move {
        name:      String,
        speed:     i32,
        direction: Direction,
    },
    Attack {
        name:      String,
        speed:     i32,
        target_id: i32,
    },
    Defend {
        name:   String,
        health: i32,
    },
    Rest {
        name:  String,
        regen: i32,
    },
}

/// Construct a new player action from a given key code.
/// Get player's action item that corresponds with the player input and construct a new action
/// from the parameters in both
// NOTE: In the future we'll have to consider mouse clicks as well.
pub fn get_player_action(player_input: PlayerInput) -> Box<dyn Action> {
    // TODO: Use actual action energy costs.
    // No need to map `Esc` since we filter out exiting before instantiating
    // any player actions.
    // println!("player action: {:?}", player_action);
    match player_input {
        // TODO: Check if we can actually move!
        PlayerInput::Move(direction) => Box::new(MoveAction::new(direction, 0)),
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
    range:   i32,
    actions: Vec<ActionItem>,
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
    capacity: i32,
    actions:  Vec<ActionItem>,
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
    speed:   i32,
    actions: Vec<ActionItem>,
}

/// ActionFactory takes an Action items with its parameters, add in-game situational parameters and
/// returns a complete instantiated Action.
pub struct ActionFactory {
    abstract_item: Option<Box<dyn Action>>,
}

// impl ActionFactory {
//     fn new() -> Self {
//         ActionFactory {
//             abstract_item: None,
//         }
//     }

//     fn abstract_action_from(self, action_item: ActionItem) -> ActionFactory {
//         let abstract_item = match action_item {
//             ActionItem::Move{code, name, speed} => {
//                 Some(Box::new(MoveAction::new()))
//             }
//             _ => None
//         };
//         self
//     }
// }

/// This is a reverse ObjectTraitBuilder. Instead of constructing taits out of DNA, it generates a
/// DNA from a given set of object traits.
// TODO: How to map object traits to genes?
pub struct DnaGenerator {}

/// Not to be confused with Rust traits, object traits are the attributes and functions that the
/// object receives via its DNA. This constructs the sensor, processor and actuator components of
/// an object.
// TODO: How to map genes to object traits?
// pub struct ObjectTraitBuilder {
//     pointer: usize,
// }

// impl ObjectTraitBuilder {
//     pub fn new() -> Self {
//         ObjectTraitBuilder { pointer: 0 }
//     }

//     pub fn parse_dna(self, dna: &[char]) -> Self {

//         self
//     }
// }
pub fn build_object_traits(dna: &[u8]) -> (Sensor, Processor, Actuator) {
    let mut sensor = Sensor {
        range:   0,
        actions: vec![],
    };
    let mut processor = Processor {
        capacity: 0,
        actions:  vec![],
    };
    let mut actuator = Actuator {
        speed:   0,
        actions: vec![],
    };

    let mut ptr = 1;

    while ptr < dna.len() {
        // in case the byte is greater than 3, "wrap around" and repeat the cycle 1, 2, 3
        match (dna[ptr] % 4) as u8 {
            START => {
                ptr += 1;
            }
            TYPE_SENSOR => {
                ptr = read_sensor(dna, &ptr, &mut sensor);
            }
            TYPE_PROCESSOR => {
                ptr = read_processor(dna, &ptr, &mut processor);
            }
            TYPE_ACTUATOR => {
                ptr = read_actuator(dna, &ptr, &mut actuator);
            }
            _x => panic!("[dna] read unknown gene {}", _x),
        }
    }

    // TODO: How do we get the actions and how do we avoid duplicates?
    (sensor, processor, actuator)
}

/// Read a gene from the dna and return the position of the next gene start.
fn read_sensor(dna: &[u8], ptr: &usize, sensor: &mut Sensor) -> usize {
    let mut next_start_ptr: usize = ptr + 1;
    // read range
    match get_value_at(dna, next_start_ptr) {
        -1 => {
            return next_start_ptr;
        }
        0 => {}
        _x => {
            sensor.range = (sensor.range + _x) / 2;
            next_start_ptr += 1;
        }
    }
    // TODO: add accuracy

    next_start_ptr
}

/// Read a gene from the dna and return the position of the next gene start.
fn read_processor(dna: &[u8], ptr: &usize, processor: &mut Processor) -> usize {
    let mut next_start_ptr: usize = ptr + 1;
    // read range
    match get_value_at(dna, next_start_ptr) {
        -1 => {
            return next_start_ptr;
        }
        0 => {}
        _x => {
            processor.capacity = (processor.capacity + _x) / 2;
            next_start_ptr += 1;
        }
    }

    next_start_ptr
}

/// Read a gene from the dna and return the position of the next gene start.
fn read_actuator(dna: &[u8], ptr: &usize, actuator: &mut Actuator) -> usize {
    let mut next_start_ptr: usize = ptr + 1;
    // read range
    match get_value_at(dna, next_start_ptr) {
        -1 => {
            return next_start_ptr;
        }
        0 => {}
        _x => {
            actuator.speed = (actuator.speed + _x) / 2;
            next_start_ptr += 1;
        }
    }

    next_start_ptr
}

fn get_value_at(dna: &[u8], ptr: usize) -> i32 {
    if dna.len() > ptr {
        dna[ptr] as i32
    } else {
        -1
    }
}
