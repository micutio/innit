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

#[derive(FromPrimitive)]
enum ComplementComponent {
    C3 = 0,
    C3a = 1,
    C3b = 2,
}

struct ComplementSystem {}
