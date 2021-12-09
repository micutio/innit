//! This module contains the particle/animation system

use crate::core::position::Position;
use rltk::{PointF, Rltk, RGBA};

const TIME_MS_PER_FRAME: f32 = 1000.0 / 60.0;

pub struct Particle {
    pub pos: PointF,
    pub col_fg: RGBA,
    pub col_bg: RGBA,
    pub glyph: char,
    /// Lifetime of the particle, given in [ms]
    pub lifetime: f32,
    /// delay until the particle is displayed, given in [ms]
    pub start_delay: f32,
}

impl Particle {
    pub fn new<NumT, RgbT>(
        x: NumT,
        y: NumT,
        col_fg: RgbT,
        col_bg: RgbT,
        glyph: char,
        lifetime: f32,
        start_delay: f32,
    ) -> Self
    where
        NumT: TryInto<f32>,
        RgbT: Into<RGBA>,
    {
        Particle {
            pos: PointF::new(
                x.try_into().ok().unwrap_or(0.0),
                y.try_into().ok().unwrap_or(0.0),
            ),
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

        // if we have multiple particles, then render one per frame
        if self.end_pos.is_some() || self.end_col.is_some() {
            let pos_f = PointF::new(self.pos.x as f32, self.pos.y as f32);

            let mut t = 0.0;
            while t < self.lifetime {
                let progress = t / self.lifetime;
                println!("PROGRESS: {}%", progress);
                let pos = self.end_pos.map_or(pos_f, |p| {
                    PointF::new(
                        pos_f.x + (progress * (p.x as f32 - pos_f.x)),
                        pos_f.y + (progress * (p.y as f32 - pos_f.y)),
                    )
                });
                let col = self.end_col.map_or((self.col_fg, self.col_bg), |c| {
                    (
                        RGBA::from(self.col_fg).lerp(c.0, progress),
                        RGBA::from(self.col_fg).lerp(c.1, progress),
                    )
                });
                let particle = Particle::new(
                    pos.x,
                    pos.y,
                    col.0,
                    col.1,
                    self.glyph,
                    TIME_MS_PER_FRAME,
                    self.start_delay + t,
                );
                particles.push(particle);
                t += TIME_MS_PER_FRAME;
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
