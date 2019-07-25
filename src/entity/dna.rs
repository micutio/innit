use crate::entity::action::Action;

/// The DNA contains all core information, excluding temporary info such as position etc. This
/// module allows to generate objects from DNA and modify them using mutation as well as crossing.
/// Decoding DNA delivers attributes and functions that fall into one of three gene types.
///
/// ## Gene Types
///
/// * perceptor - gathering information of the environment
/// * processor - decision making
/// * actuator - interacting with other objects and the game world
///
/// ## Shape of the DNA
///
/// +------+-----------+---------+
/// | 0x00 | gene type | details |
/// +------+-----------+---------+
///
/// ### perceptor details
///
/// +-------+----------+
/// | range | accuracy |
/// +-------+----------+
///
/// ### processor details
///
/// +----------+
/// | capacity |
/// +----------+
///
/// ### actuator details
///
/// +-------+------------+-----------+
/// | speed | randomness | direction |
/// +-------+------------+-----------+
///
/// A DNA Genome is implemented as a string of hexadecimal numbers. The start of a gene is marked
/// by the number zero. Genes can overlap, so that parsing the new gene resumes "in the middle" of
/// a previous gene. The genes should be small and encoding the presence of a quality. Versatility
/// is then controlled by the cumulative occurrence of a gene.
/// Basically: the more often a gene occurs, the stronger it's trait will be.
///
/// ## List of potential genes
///
/// | Gene | Primary Trait | Trait Attributes | Potential Synergy    | Potential Anti-Synergy |
/// | ---- | ------------- | ---------------- | -------------------- | ---------------------- |
/// |      | sensing       | range, accuracy  | movement (organelle) | camouflage?            |
/// |      | movement      | speed, direction | sensing (organelle)  | camouflage?            |
/// |      | attack        | energy, damage   |                      | defense                |
/// |      | defense       | membrane         |                      | attack                 |
/// |      | camouflage    | energy, accuracy |                      |                        |
///
// TODO: Design a DNA parser and a mapping from symbol to trait struct.
// TODO: Can behavior be encoded in the genome too i.e., fight or flight?
pub struct DNA {
    sequence: String,
}

/// Not to be confused with Rust traits, object traits are the attributes and functions that the
/// object receives via its DNA. This constructs the sensor, processor and actuator components of
/// an object.
// TODO: How to map genes to object traits?
pub struct ObjectTraitBuilder {}

/// This is a reverse ObjectTraitBuilder. Instead of constructing taits out of DNA, it generates a
/// DNA from a given set of object traits.
// TODO: How to map object traits to genes?
pub struct DnaGenerator {}

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
pub struct Sensor {
    range: i32,
    actions: Vec<Box<dyn Action>>,
}

/// Processors contain:
/// - attributes:
///   - capacity, a quantization/modifier of how energy-costly and complex the functions are
/// - functions:
///   - setting of primary/secondary actions [player]
///   - decision making algorithm [player/ai]
///   - ai control [ai]
pub struct Processor {
    capacity: i32,
    actions: Vec<Box<dyn Action>>,
}

/// Actuators can actually be concrete body parts e.g., organelles, spikes
/// Actuators contain:
/// - attributes:
///   - speed, a modifier of the energy cost of the functions
/// - functions:
///   - move
///   - attack
///   - defend
pub struct Actuator {
    speed: i32,
    actions: Vec<Box<dyn Action>>,
}
