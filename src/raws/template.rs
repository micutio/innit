use serde::{Deserialize, Serialize};

use crate::entity::genetics;
use crate::entity::object;
/// Struct for spawning objects that requires an internal state.
/// Templates can be created from game data serialised with JSON.
///
/// Example:
/// ```
/// use crate::entity::genetics;
/// use crate::entity::object;
/// ObjectTemplate {
///     npc: "Virus".to_string(),
///     glyph: 'v',
///     physics: object::Physics {
///         is_blocking: true,
///         is_blocking_sight: true,
///         is_always_visible: false,
///         is_visible: false,
///     },
///     color: (90, 255, 0, 255),
///     item: None,
///     controller: Some("AI_VIRUS".to_string()),
///     dna_type: genetics::DnaType::Rna,
///     dna_template: DnaTemplate::Random { genome_len: 10 },
///     stability: 0.75,
/// }
/// ```
#[derive(Serialize, Deserialize, Clone)]
pub struct ObjectTemplate {
    pub npc: String,
    pub glyph: char,
    pub physics: object::Physics,
    pub color: (u8, u8, u8, u8),
    pub item: Option<ItemTemplate>,
    pub controller: Option<String>,
    pub dna_type: genetics::DnaType,
    pub dna_template: DnaTemplate,
    pub stability: f64,
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
pub struct ItemTemplate {
    pub name: String,
    pub action: String,
}

impl ObjectTemplate {
    pub fn _example() -> Vec<Self> {
        vec![ObjectTemplate {
            npc: "Virus".to_string(),
            glyph: 'v',
            physics: object::Physics {
                is_blocking: true,
                is_blocking_sight: true,
                is_always_visible: false,
                is_visible: false,
            },
            color: (90, 255, 0, 255),
            item: None,
            controller: Some("AI_VIRUS".to_string()),
            dna_type: genetics::DnaType::Rna,
            dna_template: DnaTemplate::Random { genome_len: 10 },
            stability: 0.75,
        }]
    }
}
