pub mod color;
pub mod custom;
pub mod dialog;
pub mod frontend;
pub mod hud;
pub mod input;
pub mod menu;
pub mod particle;
pub mod rex_assets;

use crate::game::Position;

use std::sync::{Mutex, MutexGuard};

lazy_static! {
    static ref PARTICLE_SYS: Mutex<particle::ParticleSystem> =
        Mutex::new(particle::ParticleSystem::new());
}

pub fn register_particle(
    pos: Position,
    col_fg: (u8, u8, u8, u8),
    col_bg: (u8, u8, u8, u8),
    glyph: char,
    lifetime: f32,
    scale: (f32, f32),
) {
    let mut particle_sys = PARTICLE_SYS.lock().unwrap();
    particle_sys.particles.push(particle::Particle::new(
        pos.x() as f32,
        pos.y() as f32,
        col_fg,
        col_bg,
        glyph,
        lifetime,
        0.0,
        scale,
    ));
}

pub fn register_particles(builder: particle::ParticleBuilder) {
    let mut particle_sys = PARTICLE_SYS.lock().unwrap();
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
