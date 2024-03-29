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
//! - `CauseInflammation` up if pathogen present
//! - `CauseInflammation` up if `CauseInflammation` adjacent
//! - `CauseInflammation` up if unprocessed cell waste present
//! - web protein up if pathogen present AND `CauseInflammation` present
//! - stinging protein up if pathogen present AND `CauseInflammation` up
//! - inhibitor protein up if triggered? AND no pathogen present
//! - inhibitor protein up if processed cell waste present
//! - `CauseInflammation` down if inhibitor protein up

use crate::game;
use crate::world_gen::TileType;
use serde::{Deserialize, Serialize};

// #[derive(FromPrimitive)]
// enum ComplementProtein {
//     AttackMembrane = 0,    // first pathway
//     MarkAsPathogen = 1,    // second pathway, alert phagocytes to attack a pathogen
//     CauseInflammation = 2, // third pathway, sounding the alarm bells that a pathogen is detected
//     InhibitCascade = 3,    // regulation of the complement system
// }

const MIN_CONCENTRATION: f32 = 0.0;
const MAX_CONCENTRATION: f32 = 0.99;

#[derive(Debug, Serialize, Deserialize)]
pub struct Proteins {
    pub current_proteins: [f32; 4], // 4 <- number of complement system proteins
    next_proteins: [f32; 4],        // 4 <- number of complement system proteins
}

impl Proteins {
    pub const fn new() -> Self {
        let current_proteins = [0.0, 0.0, 0.0, 0.0];
        let next_proteins = [0.0, 0.0, 0.0, 0.0];

        Self {
            current_proteins,
            next_proteins,
        }
    }

    pub fn cause_inflammation(&mut self) {
        self.current_proteins[2] = f32::min(self.current_proteins[2] + 0.99, MAX_CONCENTRATION);
    }

    pub fn detect_neighbor_concentration(&mut self, neighbor_tiles: game::objects::Neighborhood) {
        let mut accumulated_proteins = [0.0; 4];
        let mut neighbor_count = 0.0;
        neighbor_tiles
            .flatten()
            .filter(|obj| {
                obj.tile
                    .as_ref()
                    .map_or(false, |t| matches!(t.typ, TileType::Floor))
            })
            .for_each(|obj| {
                if let Some(t) = &obj.tile {
                    neighbor_count += 1.0;
                    for (rref, val) in accumulated_proteins
                        .iter_mut()
                        .zip(t.complement.current_proteins)
                    {
                        *rref += val;
                    }
                }
            });

        if neighbor_count > 0.0 {
            accumulated_proteins
                .iter_mut()
                .for_each(|val| *val /= neighbor_count);

            (0..accumulated_proteins.len()).for_each(|i| {
                let delta = accumulated_proteins[i] - self.current_proteins[i];
                let new_value = delta.mul_add(0.5, self.current_proteins[i]);
                self.next_proteins[i] =
                    f32::max(f32::min(new_value, MAX_CONCENTRATION), MIN_CONCENTRATION);
            });
        }
    }

    pub fn update(&mut self) {
        std::mem::swap(&mut self.current_proteins, &mut self.next_proteins);
    }

    pub fn decay(&mut self) {
        let inhibitor_idx: usize = 3;
        (0..self.current_proteins.len()).for_each(|i| {
            let decay_rate = f32::max(self.current_proteins[i] * 0.33, 0.01);
            self.current_proteins[i] =
                f32::max(self.current_proteins[i] - decay_rate, MIN_CONCENTRATION);
        });

        // if there are inhibitor proteins present, accelerate the complement decay
        (0..inhibitor_idx).for_each(|i| {
            let inhibition_rate = self.current_proteins[i] - self.current_proteins[inhibitor_idx];
            if inhibition_rate < 0.0 {
                self.current_proteins[i] = f32::max(
                    self.current_proteins[i] - inhibition_rate,
                    MIN_CONCENTRATION,
                );
            }
        });
    }
}

impl Default for Proteins {
    fn default() -> Self {
        Self::new()
    }
}
