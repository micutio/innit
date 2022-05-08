//! # The complement system and cytokines
//! At the core the complement system is an information propagation mechanism that enables the
//! immune system to conduct large scale coordinated responses to pathogens but also for self-
//! regulation.
//!
//! Complement triggers the following immune functions:[12]
//! Membrane attack – by rupturing the cell wall of bacteria. (Classical Complement Pathway)
//! Phagocytosis – by opsonizing antigens. C3b has most important opsonizing activity. (Alternative
//! Complement Pathway)
//! Inflammation – by attracting macrophages and neutrophils. (Lectin pathway)
//!
//! The complement system can be triggered by "foreign" proteines on the surface membrane of
//! pathogens or host cells that have been damaged physically or by infection.
//!
//! ## General idea of interactions
//! - CauseInflammation up if pathogen present
//! - CauseInflammation up if CauseInflammation adjacent
//! - CauseInflammation up if unprocessed cell waste present
//! - web protein up if pathogen present AND CauseInflammation present
//! - stinging protein up if pathogen present AND CauseInflammation up
//! - inhibitor protein up if triggered? AND no pathogen present
//! - inhibitor protein up if processed cell waste present
//! - CauseInflammation down if inhibitor protein up

use crate::game;
use crate::world_gen::TileType;
use serde::{Deserialize, Serialize};
// use std::mem;

// #[derive(FromPrimitive)]
enum ComplementProtein {
    AttackMembrane = 0,    // first pathway
    MarkAsPathogen = 1,    // second pathway, alert phagocytes to attack a pathogen
    CauseInflammation = 2, // third pathway, sounding the alarm bells that a pathogen is detected
    InhibitCascade = 3,    // regulation of the complement system
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplementSystem {
    min_concentration: f32,
    max_concentration: f32,
    current_proteins: [f32; 4], // 4 <- number of complement system proteins
    next_proteins: [f32; 4],    // 4 <- number of complement system proteins
}

impl ComplementSystem {
    pub fn new() -> Self {
        let min_concentration = 0.0;
        let max_concentration = 0.99;
        let current_proteins = [0.0, 0.0, 0.0, 0.99];
        let next_proteins = [0.0, 0.0, 0.0, 0.99];

        ComplementSystem {
            min_concentration,
            max_concentration,
            current_proteins,
            next_proteins,
        }
    }

    pub fn detect_neighbor_concentration(&mut self, neighbor_tiles: game::objects::Neighborhood) {
        let mut accumulated_proteins = [0.0; 4];
        let mut neighbor_count = 0.0;
        neighbor_tiles
            .flatten()
            .filter(|obj| {
                if let Some(t) = &obj.tile {
                    matches!(t.typ, TileType::Floor)
                } else {
                    false
                }
            })
            .for_each(|obj| {
                neighbor_count += 1.0;
                if let Some(t) = &obj.tile {
                    for (rref, val) in accumulated_proteins
                        .iter_mut()
                        .zip(t.complement.current_proteins)
                    {
                        *rref += val;
                    }
                }
            });
        accumulated_proteins
            .iter_mut()
            .for_each(|val| *val /= neighbor_count);

        (0..accumulated_proteins.len()).for_each(|i| {
            self.next_proteins[i] = self.current_proteins[i]
                + ((accumulated_proteins[i] - self.current_proteins[i]) * 0.5);
        });
    }

    pub fn update(&mut self) {
        (self.current_proteins, self.next_proteins) = (self.next_proteins, self.current_proteins)
    }

    pub fn decay(&mut self) {
        for i in 0..self.current_proteins.len() {
            let decay_rate = f32::max(self.current_proteins[i] * 0.33, 0.01);
            self.current_proteins[i] = f32::max(
                self.current_proteins[i] - decay_rate,
                self.min_concentration,
            );
        }
    }
}

impl Default for ComplementSystem {
    fn default() -> Self {
        Self::new()
    }
}
