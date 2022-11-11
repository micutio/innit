use serde::{Deserialize, Serialize};

use crate::entity;
use crate::game;
use crate::ui;

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

/// Struct for spawning objects that requires an internal state.
/// Templates can be created from game data serialised with JSON.
///
/// Example:
/// ```
/// use innit::entity::genetics;
/// use innit::entity::object;
/// use innit::raws::templating::{DnaTemplate, ObjectTemplate};
/// use innit::ui;
///
/// ObjectTemplate {
///     npc: "Virus".to_string(),
///     glyph: 'v',
///     physics: object::Physics {
///         is_blocking: true,
///         is_blocking_sight: true,
///         is_always_visible: false,
///         is_visible: false,
///     },
///     color: ui::Rgba::new(90, 255, 0, 255),
///     item: None,
///     controller: Some("AI_VIRUS".to_string()),
///     dna_type: genetics::DnaType::Rna,
///     dna_template: DnaTemplate::Random { genome_len: 10 },
///     stability: 0.75,
/// };
/// ```
#[derive(Serialize, Deserialize, Clone)]
pub struct ObjectTemplate {
    pub npc: String,
    pub glyph: char,
    pub physics: entity::object::Physics,
    pub color: ui::Rgba,
    pub item: Option<ItemTemplate>,
    pub controller: Option<String>,
    pub dna_type: entity::genetics::DnaType,
    pub dna_template: DnaTemplate,
    pub stability: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ItemTemplate {
    pub name: String,
    pub action: String,
}

impl ObjectTemplate {
    #[must_use]
    pub fn _example() -> Vec<Self> {
        let is_visible = matches!(game::env().debug_mode, game::env::GameOption::Enabled);
        vec![Self {
            npc: "Virus".to_string(),
            glyph: 'v',
            physics: entity::object::Physics {
                is_blocking: true,
                is_blocking_sight: false,
                is_always_visible: false,
                is_visible,
            },
            color: ui::Rgba::new(90, 255, 0, 255),
            item: None,
            controller: Some("AI_VIRUS".to_string()),
            dna_type: entity::genetics::DnaType::Rna,
            dna_template: DnaTemplate::Random { genome_len: 10 },
            stability: 0.75,
        }]
    }
}
