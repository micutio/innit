//! # The Spawn System
//!
//! - transition table assigns spawn likelyhood per level range
//! - spawn table accumulates spawn likelyhoods for all NPCs and chooses one at random
//! - after choosing monster concrete DNA or template is used to initialise objects
//! - object is placed in the world

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Transition<T> {
    pub level: u32,
    pub value: T,
}

/// Struct for spawning objects that requires an internal state.
#[derive(Serialize, Deserialize, Clone)]
pub struct Spawn {
    pub npc: String,
    pub spawn_transitions: Vec<Transition<u32>>,
    // pub dna_transitions: Vec<Transition<DnaTemplate>>,
}

impl Spawn {
    pub fn example() -> Vec<Self> {
        vec![
            Spawn {
                npc: "Virus".to_string(),
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
                // dna_transitions: vec![
                //     Transition {
                //         level: 1,
                //         value: DnaTemplate::Random { genome_len: 13 },
                //     },
                //     Transition {
                //         level: 6,
                //         value: DnaTemplate::Distributed {
                //             s_rate: 20,
                //             p_rate: 45,
                //             a_rate: 35,
                //         },
                //     },
                //     Transition {
                //         level: 8,
                //         value: DnaTemplate::Defined {
                //             traits: vec!["foo".to_string(), "bar".to_string(), "baz".to_string()],
                //         },
                //     },
                // ],
            },
            Spawn {
                npc: "Virus".to_string(),
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
                // dna_transitions: vec![
                //     Transition {
                //         level: 1,
                //         value: DnaTemplate::Random { genome_len: 13 },
                //     },
                //     Transition {
                //         level: 6,
                //         value: DnaTemplate::Distributed {
                //             s_rate: 20,
                //             p_rate: 45,
                //             a_rate: 35,
                //         },
                //     },
                //     Transition {
                //         level: 8,
                //         value: DnaTemplate::Defined {
                //             traits: vec!["foo".to_string(), "bar".to_string(), "baz".to_string()],
                //         },
                //     },
                // ],
            },
        ]
    }
}

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
