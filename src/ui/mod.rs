pub mod color_palette;
pub mod custom;
pub mod dialog;
pub mod frontend;
pub mod game_input;
pub mod hud;
pub mod menu;
pub mod particle;
pub mod rex_assets;

use std::sync::{Mutex, MutexGuard};

use crate::game::position::Position;
use crate::ui::color_palette::{ColorPalette, PALETTE_DEFAULT};
use crate::ui::particle::{Particle, ParticleBuilder, ParticleSystem};

lazy_static! {
    static ref PARTICLE_SYS: Mutex<ParticleSystem> = Mutex::new(ParticleSystem::new());
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
    particle_sys.particles.push(Particle::new(
        pos.x as f32,
        pos.y as f32,
        col_fg,
        col_bg,
        glyph,
        lifetime,
        0.0,
        scale,
    ));
}

pub fn register_particles(builder: ParticleBuilder) {
    let mut particle_sys = PARTICLE_SYS.lock().unwrap();
    particle_sys.particles.append(&mut builder.build());
}

pub fn particles<'a>() -> MutexGuard<'a, ParticleSystem> {
    PARTICLE_SYS.lock().unwrap()
}

lazy_static! {
    static ref COLOR_PALETTE: Mutex<ColorPalette> = Mutex::new(PALETTE_DEFAULT);
}

pub fn palette<'a>() -> MutexGuard<'a, ColorPalette> {
    COLOR_PALETTE.lock().unwrap()
}
