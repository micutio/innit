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

#[derive(FromPrimitive)]
enum ComplementProtein {
    AttackMembrane = 0,    // first pathway
    MarkAsPathogen = 1,    // second pathway, alert phagocytes to attack a pathogen
    CauseInflammation = 2, // third pathway, sounding the alarm bells that a pathogen is detected
    InhibitCascade = 3,    // regulation of the complement system
}

struct ComplementSystem {
    min_concentration: f32,
    max_concentration: f32,
    protein_concentration: [f32; 4], // 4 <- number of complement system proteins
}

impl ComplementSystem {
    fn new() -> Self {
        let min_concentration = 0.0;
        let max_concentration = 100.0;
        let protein_concentration = [0.0, 0.0, 0.0, 0.0];

        ComplementSystem {
            min_concentration,
            max_concentration,
            protein_concentration,
        }
    }

    fn update(&mut self, neighbor_protein_concentration: [f32; 4]) {}

    fn decay(&mut self) {
        for i in self.protein_concentration.len {
            self.protein_concentration[i] =
                min(self.protein_concentration - 0.1, self.min_concentration);
        }
    }
}
