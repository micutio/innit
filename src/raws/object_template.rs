use serde::{Deserialize, Serialize};

use crate::entity::genetics::DnaType;
use crate::entity::object::Physics;
/// Struct for spawning objects that requires an internal state.
#[derive(Serialize, Deserialize, Clone)]
pub struct ObjectTemplate {
    pub npc: String,
    pub glyph: char,
    pub physics: Physics,
    pub color: (u8, u8, u8),
    pub item: Option<InvItem>,
    pub controller: String,
    pub dna_type: DnaType,
    pub dna_template: DnaTemplate,
    pub stability: f64,
    // pub dna_transitions: Vec<Transition<DnaTemplate>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum DnaTemplate {
    Random {
        genome_len: usize,
    },
    Distributed {
        s_rate: u8,
        p_rate: u8,
        a_rate: u8,
        genome_len: usize,
    },
    Defined {
        traits: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InvItem {
    pub name: String,
    pub action: String,
}

impl ObjectTemplate {
    pub fn example() -> Vec<Self> {
        vec![ObjectTemplate {
            npc: "Virus".to_string(),
            glyph: 'v',
            physics: Physics {
                is_blocking: true,
                is_blocking_sight: true,
                is_always_visible: false,
                is_visible: false,
            },
            color: (90, 255, 0),
            item: None,
            controller: "AI_VIRUS".to_string(),
            dna_type: DnaType::Rna,
            dna_template: DnaTemplate::Random { genome_len: 10 },
            stability: 0.75,
        }]
    }
}
