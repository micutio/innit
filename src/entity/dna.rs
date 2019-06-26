/// Module DNA
///
/// The DNA contains all core information, excluding temporary info such as
/// position etc. This module allows to generate objects from DNA and modify
/// them using mutation as well as crossing.
/// Decoding DNA delivers attributes and functions that fall into one of three
/// categories: perception, orientation (a.k.a. processing), actuation.
///
use entity::object::Object;

pub fn generate_dna(object: &Object) -> String {
    unimplemented!();
}

pub fn parse_dna_to_object(dna: &str) -> Object {
    unimplemented!();
}

enum ParsingState {
    Init,
    ReadAttribute,
    ReadSkill,
}

struct DnaParser {
    state: ParsingState,
}
