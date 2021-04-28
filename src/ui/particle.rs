//! This module contains the particle/animation system

use rltk::Rltk;

use crate::{core::position::Position, ui::color::Color};

pub struct Particle {
    pub pos: Position,
    pub col_fg: Color,
    pub col_bg: Color,
    pub glyph: char,
    /// Lifetime of the particle, given in [ms]
    pub lifetime: f32,
}

impl Particle {
    fn new(pos: Position, col_fg: Color, col_bg: Color, glyph: char, lifetime: f32) -> Self {
        Particle {
            pos,
            col_fg,
            col_bg,
            glyph,
            lifetime,
        }
    }
}

pub struct ParticleSystem {
    pub particles: Vec<Particle>,
}

impl ParticleSystem {
    fn new() -> Self {
        ParticleSystem {
            particles: Vec::new(),
        }
    }

    fn is_active(&self) -> bool {
        !self.particles.is_empty()
    }

    /// Advance the particle lifetimes and cull all those that have expired.
    fn update(&mut self, ctx: &Rltk) {
        self.particles
            .iter_mut()
            .filter(|p| p.lifetime - ctx.frame_time_ms < 0.0)
            .for_each(|p| p.lifetime -= ctx.frame_time_ms);
    }
}
