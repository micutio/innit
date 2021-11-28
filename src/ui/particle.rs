//! This module contains the particle/animation system

use crate::core::position::Position;
use rltk::{Bresenham, Point, Rltk, RGBA};

pub struct Particle {
    pub pos: Position,
    pub col_fg: RGBA,
    pub col_bg: RGBA,
    pub glyph: char,
    /// Lifetime of the particle, given in [ms]
    pub lifetime: f32,
    /// delay until the particle is displayed, given in [ms]
    pub start_delay: f32,
}

impl Particle {
    pub(crate) fn new<T: Into<RGBA>>(
        pos: Position,
        col_fg: T,
        col_bg: T,
        glyph: char,
        lifetime: f32,
        start_delay: f32,
    ) -> Self {
        Particle {
            pos,
            col_fg: col_fg.into(),
            col_bg: col_bg.into(),
            glyph,
            lifetime,
            start_delay,
        }
    }
}

pub struct ParticleBuilder {
    pos: Position,
    col_fg: RGBA,
    col_bg: RGBA,
    glyph: char,
    lifetime: f32,
    start_delay: f32,
    end_pos: Option<Position>,
    end_col: Option<(RGBA, RGBA)>,
}

impl ParticleBuilder {
    pub fn new<T: Into<RGBA>>(
        pos: Position,
        col_fg: T,
        col_bg: T,
        glyph: char,
        lifetime: f32,
    ) -> Self {
        ParticleBuilder {
            pos,
            col_fg: col_fg.into(),
            col_bg: col_bg.into(),
            glyph,
            lifetime,
            start_delay: 0.0,
            end_pos: None,
            end_col: None,
        }
    }

    pub fn _with_delay(mut self, start_delay: f32) -> Self {
        self.start_delay = start_delay;
        self
    }

    pub fn with_moving_to(mut self, end_pos: Position) -> Self {
        self.end_pos = Some(end_pos);
        self
    }

    pub fn with_end_color<T: Into<RGBA>>(mut self, col_fg: T, col_bg: T) -> Self {
        self.end_col = Some((col_fg.into(), col_bg.into()));
        self
    }

    // TODO: Extract each branch into a separate helper function.
    pub fn build(self) -> Vec<Particle> {
        let mut particles = Vec::new();
        if let Some(p) = self.end_pos {
            let mut path: Vec<Point> = Bresenham::new(self.pos.into(), p.into()).collect();
            path.push(p.into());
            let path_len = path.len() as f32;
            println!("BRESENHAM PATH LEN: {}", path_len);
            let time_per_part = self.lifetime / path_len;
            let mut delay = self.start_delay;
            for (idx, point) in path.into_iter().enumerate() {
                let part_ratio = idx as f32 / path_len;
                let part_colors = if let Some((col_fg, col_bg)) = self.end_col {
                    let mut fg = RGBA::from(self.col_fg);
                    fg = fg.lerp(col_fg.into(), part_ratio);
                    let mut bg = RGBA::from(self.col_bg);
                    bg = bg.lerp(col_bg.into(), part_ratio);
                    (fg, bg)
                } else {
                    (self.col_fg, self.col_bg)
                };
                let part_delay = delay;
                delay += time_per_part;
                let particle = Particle::new(
                    point.into(),
                    part_colors.0,
                    part_colors.1,
                    self.glyph,
                    time_per_part,
                    part_delay,
                );
                particles.push(particle);
            }
        } else {
            if let Some((col_fg, col_bg)) = self.end_col {
                let time_per_part = self.lifetime / 2.0;
                let particle1 = Particle::new(
                    self.pos,
                    self.col_fg,
                    self.col_bg,
                    self.glyph,
                    time_per_part,
                    self.start_delay,
                );

                let particle2 = Particle::new(
                    self.pos,
                    col_fg,
                    col_bg,
                    self.glyph,
                    time_per_part,
                    self.start_delay + time_per_part,
                );

                particles.push(particle1);
                particles.push(particle2);
            } else {
                let particle = Particle::new(
                    self.pos,
                    self.col_fg,
                    self.col_bg,
                    self.glyph,
                    self.lifetime,
                    self.start_delay,
                );
                particles.push(particle);
            }
        }
        particles
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
    /// Returns `true` if some particles expired in this call.
    pub fn update(&mut self, ctx: &Rltk) -> bool {
        let start_size: usize = self.particles.len();
        self.particles.iter_mut().for_each(|p| {
            if p.start_delay > 0.0 {
                p.start_delay -= ctx.frame_time_ms
            } else {
                p.lifetime -= ctx.frame_time_ms
            }
        });

        self.particles.retain(|p| p.lifetime > 0.0);
        self.particles.len() < start_size
    }
}
