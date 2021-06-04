//! This module contains the particle/animation system

use rltk::Rltk;

use crate::core::position::Position;

pub struct Particle {
    pub pos: Position,
    pub col_fg: (u8, u8, u8),
    pub col_bg: (u8, u8, u8),
    pub glyph: char,
    /// Lifetime of the particle, given in [ms]
    pub lifetime: f32,
}

impl Particle {
    pub(crate) fn new(
        pos: Position,
        col_fg: (u8, u8, u8),
        col_bg: (u8, u8, u8),
        glyph: char,
        lifetime: f32,
    ) -> Self {
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
    pub fn new() -> Self {
        ParticleSystem {
            particles: Vec::new(),
        }
    }

    /// Advance the particle lifetimes and cull all those that have expired.
    /// Returns true if some particles expired in this call.
    pub fn update(&mut self, ctx: &Rltk) -> bool {
        let start_size: usize = self.particles.len();
        self.particles
            .iter_mut()
            .for_each(|p| p.lifetime -= ctx.frame_time_ms);

        self.particles.retain(|p| p.lifetime > 0.0);
        self.particles.len() < start_size
    }
}
