//! # The Spawn System
//!
//! - transition table assigns spawn likelyhood per level range
//! - spawn table accumulates spawn likelyhoods for all NPCs and chooses one at random
//! - after choosing monster concrete DNA or template is used to initialise objects
//! - object is placed in the world

use crate::entity::genetics::DnaType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum DnaTemplate {
    Random { genome_len: u8 },
    Distributed { s_rate: u8, p_rate: u8, a_rate: u8 },
    Defined { traits: Vec<String> },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Transition<T> {
    pub level: u32,
    pub value: T,
}

/// Struct for spawning objects that requires an internal state.
#[derive(Serialize, Deserialize, Clone)]
pub struct Spawn {
    pub npc: String,
    pub glyph: char,
    pub dna_type: DnaType,
    pub stability: f64,
    pub spawn_transitions: Vec<Transition<u32>>,
    pub dna_transitions: Vec<Transition<DnaTemplate>>,
}

impl Spawn {
    pub fn example() -> Vec<Self> {
        vec![
            Spawn {
                npc: "Virus".to_string(),
                glyph: 'v',
                dna_type: DnaType::Rna,
                stability: 0.75,
                spawn_transitions: vec![
                    Transition {
                        level: 1,
                        value: 34,
                    },
                    Transition {
                        level: 3,
                        value: 21,
                    },
                ],
                dna_transitions: vec![
                    Transition {
                        level: 1,
                        value: DnaTemplate::Random { genome_len: 13 },
                    },
                    Transition {
                        level: 6,
                        value: DnaTemplate::Distributed {
                            s_rate: 20,
                            p_rate: 45,
                            a_rate: 35,
                        },
                    },
                    Transition {
                        level: 8,
                        value: DnaTemplate::Defined {
                            traits: vec!["foo".to_string(), "bar".to_string(), "baz".to_string()],
                        },
                    },
                ],
            },
            Spawn {
                npc: "Virus".to_string(),
                glyph: 'v',
                dna_type: DnaType::Rna,
                stability: 0.75,
                spawn_transitions: vec![
                    Transition {
                        level: 1,
                        value: 34,
                    },
                    Transition {
                        level: 3,
                        value: 21,
                    },
                ],
                dna_transitions: vec![
                    Transition {
                        level: 1,
                        value: DnaTemplate::Random { genome_len: 13 },
                    },
                    Transition {
                        level: 6,
                        value: DnaTemplate::Distributed {
                            s_rate: 20,
                            p_rate: 45,
                            a_rate: 35,
                        },
                    },
                    Transition {
                        level: 8,
                        value: DnaTemplate::Defined {
                            traits: vec!["foo".to_string(), "bar".to_string(), "baz".to_string()],
                        },
                    },
                ],
            },
        ]
    }
}

// pub(crate) fn new_monster(
//     state: &mut GameState,
//     monster: Monster,
//     x: i32,
//     y: i32,
//     _level: u32,
// ) -> Object {
//     // append LTR markers
//     match monster {
//         Monster::Virus => Object::new()
//             .position(x, y)
//             .living(true)
//             .visualize("Virus", 'v', palette().entity_virus)
//             .physical(true, false, false)
//             // TODO: Pull genome create out of here. It's not the same for every NPC.
//             .genome(
//                 0.75,
//                 state
//                     .gene_library
//                     .new_genetics(&mut state.rng, DnaType::Rna, true, GENE_LEN),
//             )
//             .control(Controller::Npc(Box::new(AiVirus::new()))),
//         Monster::Bacteria => Object::new()
//             .position(x, y)
//             .living(true)
//             .visualize("Bacteria", 'b', palette().entity_bacteria)
//             .physical(true, false, false)
//             .genome(
//                 0.9,
//                 state
//                     .gene_library
//                     .new_genetics(&mut state.rng, DnaType::Nucleoid, false, GENE_LEN),
//             )
//             .control(Controller::Npc(Box::new(AiRandom::new()))),
//         Monster::Plasmid => Object::new()
//             .position(x, y)
//             .living(true)
//             .visualize("Plasmid", 'p', palette().entity_plasmid)
//             .physical(false, false, false)
//             .inventory_item(InventoryItem::new(
//                 "Plasmids can transfer DNA between cells and bacteria and help manipulate it.",
//                 Some(Box::new(ActEditGenome::new())),
//             ))
//             .genome(
//                 1.0,
//                 state.gene_library.new_genetics(
//                     &mut state.rng,
//                     DnaType::Plasmid,
//                     false,
//                     _level as usize,
//                 ),
//             ),
//     }
// }

/// Return a value that depends on dungeon level.
/// The table specifies what value occurs after each level, default is 0.
pub fn from_dungeon_level<T>(table: &[Transition<T>], level: u32) -> T
where
    T: Default + Clone,
{
    table
        .iter()
        .rev()
        .find(|transition| level >= transition.level)
        .map_or(T::default(), |transition| transition.value.clone())
}
