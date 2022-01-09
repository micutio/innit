pub mod color;
pub mod custom;
pub mod dialog;
pub mod frontend;
pub mod hud;
pub mod input;
pub mod menu;
pub mod particle;
pub mod rex_assets;

use crate::game::{self, Position};
use crate::ui;

use serde::{Deserialize, Serialize};
use std::convert::From;
use std::sync::{Mutex, MutexGuard};

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub fn new(r: u8, b: u8, g: u8, a: u8) -> Self {
        Rgba { r, g, b, a }
    }

    pub fn from_tuple(t: (u8, u8, u8, u8)) -> Self {
        Rgba {
            r: t.0,
            g: t.1,
            b: t.2,
            a: t.3,
        }
    }

    pub fn from_f32(r: f32, b: f32, g: f32, a: f32) -> Self {
        Rgba {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
            a: (a * 255.0) as u8,
        }
    }
}

impl From<rltk::RGBA> for Rgba {
    fn from(item: rltk::RGBA) -> Self {
        Rgba {
            r: (item.r * 255.0) as u8,
            g: (item.g * 255.0) as u8,
            b: (item.b * 255.0) as u8,
            a: (item.a * 255.0) as u8,
        }
    }
}

impl From<Rgba> for rltk::RGBA {
    fn from(item: Rgba) -> Self {
        rltk::RGBA::from_u8(item.r, item.g, item.b, item.a)
    }
}

lazy_static! {
    static ref PARTICLE_SYS: Mutex<particle::ParticleSystem> =
        Mutex::new(particle::ParticleSystem::new());
}

pub fn register_particle(
    pos: Position,
    col_fg: ui::Rgba,
    col_bg: ui::Rgba,
    glyph: char,
    lifetime: f32,
    start_delay: f32,
    scale: (f32, f32),
) {
    let mut particle_sys = PARTICLE_SYS.lock().unwrap();
    if game::env().is_particles_disabled {
        return;
    }

    particle_sys.particles.push(particle::Particle::new(
        pos.x() as f32,
        pos.y() as f32,
        col_fg,
        col_bg,
        glyph,
        lifetime,
        start_delay,
        scale,
    ));
}

pub fn register_particles(builder: particle::ParticleBuilder) {
    let mut particle_sys = PARTICLE_SYS.lock().unwrap();
    if game::env().is_particles_disabled {
        return;
    }
    particle_sys.particles.append(&mut builder.build());
}

pub fn particles<'a>() -> MutexGuard<'a, particle::ParticleSystem> {
    PARTICLE_SYS.lock().unwrap()
}

lazy_static! {
    static ref COLOR_PALETTE: Mutex<color::Palette> = Mutex::new(color::DEFAULT_PALETTE);
}

pub fn palette<'a>() -> MutexGuard<'a, color::Palette> {
    COLOR_PALETTE.lock().unwrap()
}
