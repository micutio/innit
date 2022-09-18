//! This module contains the particle/animation system

use crate::game;

use bracket_lib::prelude as rltk;

const TIME_MS_PER_FRAME: f32 = 1000.0 / 60.0;

pub struct Particle {
    pub pos: rltk::PointF,
    pub col_fg: rltk::RGBA,
    pub col_bg: rltk::RGBA,
    pub glyph: char,
    /// Lifetime of the particle, given in [ms]
    pub lifetime: f32,
    /// delay until the particle is displayed, given in [ms]
    pub start_delay: f32,
    pub scale: (f32, f32),
}

impl Particle {
    #[allow(clippy::too_many_arguments)]
    pub fn new<NumT, RgbT>(
        x: NumT,
        y: NumT,
        col_fg: RgbT,
        col_bg: RgbT,
        glyph: char,
        lifetime: f32,
        start_delay: f32,
        scale: (f32, f32),
    ) -> Self
    where
        NumT: TryInto<f32>,
        RgbT: Into<rltk::RGBA>,
    {
        Self {
            // For some reason the y-coordinate needs to be adjusted by 1 for the particle to be
            // correct, no idea why.
            pos: rltk::PointF::new(
                x.try_into().ok().unwrap_or(0.0),
                y.try_into().ok().unwrap_or(0.0) + 1.0,
            ),
            col_fg: col_fg.into(),
            col_bg: col_bg.into(),
            glyph,
            lifetime,
            start_delay,
            scale,
        }
    }
}

pub struct Builder {
    pos: rltk::PointF,
    col_fg: rltk::RGBA,
    col_bg: rltk::RGBA,
    glyph: char,
    lifetime: f32,
    start_delay: f32,
    end_pos: Option<rltk::PointF>,
    end_col: Option<(rltk::RGBA, rltk::RGBA)>,
    scale: Option<((f32, f32), (f32, f32))>,
}

impl Builder {
    pub fn new<NumT, RgbT>(
        x: NumT,
        y: NumT,
        col_fg: RgbT,
        col_bg: RgbT,
        glyph: char,
        lifetime: f32,
    ) -> Self
    where
        NumT: TryInto<f32>,
        RgbT: Into<rltk::RGBA>,
    {
        Self {
            pos: rltk::PointF::new(
                x.try_into().ok().unwrap_or(0.0),
                y.try_into().ok().unwrap_or(0.0),
            ),
            col_fg: col_fg.into(),
            col_bg: col_bg.into(),
            glyph,
            lifetime,
            start_delay: 0.0,
            end_pos: None,
            end_col: None,
            scale: None,
        }
    }

    pub const fn _with_delay(mut self, start_delay: f32) -> Self {
        self.start_delay = start_delay;
        self
    }

    pub fn with_moving_to<NumT: TryInto<f32>>(mut self, x: NumT, y: NumT) -> Self {
        self.end_pos = Some(rltk::PointF::new(
            x.try_into().ok().unwrap_or(0.0),
            y.try_into().ok().unwrap_or(0.0),
        ));
        self
    }

    pub fn with_end_color<T: Into<rltk::RGBA>>(mut self, col_fg: T, col_bg: T) -> Self {
        self.end_col = Some((col_fg.into(), col_bg.into()));
        self
    }

    pub const fn with_scale(mut self, start_scale: (f32, f32), end_scale: (f32, f32)) -> Self {
        self.scale = Some((start_scale, end_scale));
        self
    }

    // TODO: Extract each branch into a separate helper function.
    pub fn build(self) -> Vec<Particle> {
        let mut particles = Vec::new();

        // if we have multiple particles, then render one per frame
        if self.end_pos.is_some() || self.end_col.is_some() {
            let pos_start = rltk::PointF::new(self.pos.x as f32, self.pos.y as f32);

            let mut t = 0.0;
            while t < self.lifetime {
                let progress = t / self.lifetime;
                let pos = self.end_pos.map_or(pos_start, |pos_end| {
                    rltk::PointF::new(
                        progress.mul_add(pos_end.x as f32 - pos_start.x, pos_start.x),
                        progress.mul_add(pos_end.y as f32 - pos_start.y, pos_start.y),
                    )
                });
                let col = self.end_col.map_or((self.col_fg, self.col_bg), |c| {
                    (
                        self.col_fg.lerp(c.0, progress),
                        self.col_fg.lerp(c.1, progress),
                    )
                });
                let scale = self.scale.map_or((1.0, 1.0), |(start_sc, end_sc)| {
                    (
                        progress.mul_add(end_sc.0 - start_sc.0, start_sc.0),
                        progress.mul_add(end_sc.1 - start_sc.1, start_sc.1),
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
                    scale,
                );
                particles.push(particle);
                t += TIME_MS_PER_FRAME;
            }
        }
        particles
    }
}

pub struct System {
    pub particles: Vec<Particle>,
    vignette: Vec<(game::Position, (u8, u8, u8, u8))>,
}

impl System {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            vignette: create_vignette(),
        }
    }

    pub fn render(&self, ctx: &mut rltk::BTerm) {
        ctx.set_active_console(game::consts::PAR_CON);
        ctx.cls();
        let mut draw_batch = rltk::DrawBatch::new();
        for particle in &self.particles {
            if particle.start_delay <= 0.0 {
                draw_batch.set_fancy(
                    particle.pos,
                    1,
                    rltk::Degrees::new(0.0),
                    particle.scale.into(),
                    rltk::ColorPair::new(particle.col_fg, particle.col_bg),
                    rltk::to_cp437(particle.glyph),
                );
            }
        }

        self.vignette.iter().for_each(|(pos, bg_col)| {
            let color = rltk::ColorPair::new((0, 0, 0, 0), *bg_col);
            draw_batch.print_color(rltk::Point::new(pos.x(), pos.y()), " ", color);
        });

        draw_batch.submit(game::consts::PAR_CON_Z).unwrap();
    }

    /// Advance the particle lifetimes and cull all those that have expired.
    /// Returns `true` if some particles expired in this call.
    pub fn update(&mut self, ctx: &rltk::BTerm) -> bool {
        let mut has_changed = false;
        self.particles.iter_mut().for_each(|p| {
            if p.start_delay > 0.0 {
                p.start_delay -= ctx.frame_time_ms;
                has_changed |= p.start_delay < 0.0;
            } else {
                p.lifetime -= ctx.frame_time_ms;
                has_changed |= p.lifetime < 0.0;
            }
        });

        self.particles.retain(|p| p.lifetime > 0.0);
        has_changed
    }
}

fn create_vignette() -> Vec<(game::Position, (u8, u8, u8, u8))> {
    let center_x = (game::consts::WORLD_WIDTH / 2) - 1;
    let center_y = (game::consts::WORLD_HEIGHT / 2) - 1;
    let center_pos = game::Position::from_xy(center_x, center_y);
    let center_point = rltk::Point::new(center_x, center_y);

    let start_radius = center_x - 2;
    let end_radius = center_x + 1;
    let mut vignette = Vec::new();
    for radius in start_radius..end_radius {
        for point in rltk::BresenhamCircleNoDiag::new(center_point, radius) {
            let pos = game::Position::from_xy(point.x, point.y);
            let dist = pos.distance(&center_pos);
            let ratio = (dist - start_radius as f32) / (end_radius as f32 - start_radius as f32);
            let alpha: u8 = (255.0 * ratio) as u8;
            vignette.push((pos, (0, 0, 0, alpha)));
        }
    }
    vignette
}
