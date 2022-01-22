use crate::entity::Object;
use crate::game::{self, consts, ObjectStore, Position};
use crate::ui;
use crate::util;

use bracket_lib::prelude as rltk;

pub fn render_world(objects: &mut ObjectStore, ctx: &mut rltk::BTerm, vis_update: bool) {
    // time rendering method for profiling purposes
    let mut timer = util::Timer::new("render world");
    ctx.set_active_console(game::consts::WORLD_CON);
    if vis_update && !game::env().is_debug_mode {
        draw_updated_visibility(objects);
    } else {
        draw_direct(objects);
    }
    let elapsed = timer.stop_silent();
    trace!("render world in {}", util::timer::format(elapsed));
}

struct TileColorsRgb {
    // bg wall fov true
    pub bwt: rltk::RGB,
    // bg ground fov true
    pub bft: rltk::RGB,
    // fg wall fov true
    pub fwt: rltk::RGB,
    // fg ground fov true
    pub fft: rltk::RGB,
    // bg wall fov false
    pub bwf: rltk::RGB,
    // bg ground fov false
    pub bff: rltk::RGB,
    // fg wall fov false
    pub fwf: rltk::RGB,
    // fg ground fov false
    pub fff: rltk::RGB,
}

impl TileColorsRgb {
    fn new() -> Self {
        use rltk::{RGB, RGBA};
        // default tile foreground and background colors
        let bwt = RGB::from(RGBA::from(ui::palette().world_bg_wall_fov_true));
        let bwf = RGB::from(RGBA::from(ui::palette().world_bg_wall_fov_false));
        let bft = RGB::from(RGBA::from(ui::palette().world_bg_floor_fov_true));
        let bff = RGB::from(RGBA::from(ui::palette().world_bg_floor_fov_false));
        let fwt = RGB::from(RGBA::from(ui::palette().world_fg_wall_fov_true));
        let fwf = RGB::from(RGBA::from(ui::palette().world_fg_wall_fov_false));
        let fft = RGB::from(RGBA::from(ui::palette().world_fg_floor_fov_true));
        let fff = RGB::from(RGBA::from(ui::palette().world_fg_floor_fov_false));
        TileColorsRgb {
            bwt,
            bft,
            fwt,
            fft,
            bwf,
            bff,
            fwf,
            fff,
        }
    }
}

fn draw_direct(objects: &ObjectStore) {
    let mut draw_batch_tile = rltk::DrawBatch::new();
    let mut draw_batch_nbl = rltk::DrawBatch::new();
    let mut draw_batch_blk = rltk::DrawBatch::new();

    objects
        .get_vector()
        .iter()
        .flatten()
        .filter(|o| !o.is_void())
        .for_each(|obj| {
            if obj.tile.is_some() {
                draw_batch_tile.set(
                    obj.pos.into(),
                    rltk::ColorPair::new(obj.visual.fg_color, obj.visual.bg_color),
                    rltk::to_cp437(obj.visual.glyph),
                );
            } else if obj.physics.is_visible {
                if !obj.physics.is_blocking {
                    draw_batch_nbl.set(
                        obj.pos.into(),
                        rltk::ColorPair::new(obj.visual.fg_color, obj.visual.bg_color),
                        rltk::to_cp437(obj.visual.glyph),
                    );
                } else {
                    draw_batch_blk.set(
                        obj.pos.into(),
                        rltk::ColorPair::new(obj.visual.fg_color, obj.visual.bg_color),
                        rltk::to_cp437(obj.visual.glyph),
                    );
                }
            }
        });
    draw_batch_tile.submit(game::consts::WORLD_TILE_Z).unwrap();
    draw_batch_nbl.submit(game::consts::WORLD_NBL_Z).unwrap();
    draw_batch_blk.submit(game::consts::WORLD_BLK_Z).unwrap();
}

fn draw_updated_visibility(objects: &mut ObjectStore) {
    // let tcU8 = TileColorsU8::new();
    let mut draw_batch_tile = rltk::DrawBatch::new();
    let mut draw_batch_nbl = rltk::DrawBatch::new();
    let mut draw_batch_blk = rltk::DrawBatch::new();
    let tc_rgb = TileColorsRgb::new();

    let player_views: Vec<(Position, i32)> = objects
        .get_non_tiles()
        .iter()
        .flatten()
        .filter(|o| o.is_player())
        .map(|o| (o.pos, o.sensors.sensing_range))
        .collect();
    let visible_positions: Vec<rltk::Point> = player_views
        .iter()
        .map(|(pos, range)| {
            rltk::field_of_view(rltk::Point::new(pos.x(), pos.y()), *range, objects)
        })
        .flatten()
        .collect();

    objects
        .get_vector_mut()
        .iter_mut()
        .flatten()
        .filter(|o| !o.is_void())
        .for_each(|obj| {
            let closest_player_view = player_views
                .iter()
                .min_by_key(|x| obj.pos.distance(&x.0) as i32);
            if let Some((pos, range)) = closest_player_view {
                update_visual(obj, *pos, *range, &visible_positions, &tc_rgb);
            }
            if obj.tile.is_some() {
                draw_batch_tile.set(
                    obj.pos.into(),
                    rltk::ColorPair::new(obj.visual.fg_color, obj.visual.bg_color),
                    rltk::to_cp437(obj.visual.glyph),
                );
            } else if obj.physics.is_visible {
                if !obj.physics.is_blocking {
                    draw_batch_nbl.set(
                        obj.pos.into(),
                        rltk::ColorPair::new(obj.visual.fg_color, obj.visual.bg_color),
                        rltk::to_cp437(obj.visual.glyph),
                    );
                } else {
                    draw_batch_blk.set(
                        obj.pos.into(),
                        rltk::ColorPair::new(obj.visual.fg_color, obj.visual.bg_color),
                        rltk::to_cp437(obj.visual.glyph),
                    );
                }
            }
        });
    draw_batch_tile.submit(game::consts::WORLD_TILE_Z).unwrap();
    draw_batch_nbl.submit(game::consts::WORLD_NBL_Z).unwrap();
    draw_batch_blk.submit(game::consts::WORLD_BLK_Z).unwrap();
}

/// Update the player's field of view and updated which tiles are visible/explored.
fn update_visual(
    object: &mut Object,
    player_pos: Position,
    player_sensing_range: i32,
    visible_positions: &[rltk::Point],
    tc: &TileColorsRgb,
) {
    let dist_to_player = object.pos.distance(&player_pos);
    let vis_ratio = dist_to_player / (player_sensing_range as f32 + 1.0);
    object.physics.is_visible =
        visible_positions.contains(&rltk::Point::new(object.pos.x(), object.pos.y()));

    let obj_vis = object.physics.is_visible;
    let obj_opaque = object.physics.is_blocking_sight;
    // set tile foreground and background colors
    let (tile_color_fg, tile_color_bg) = match (obj_vis, obj_opaque) {
        // outside field of view:
        (false, true) => (tc.fwf, tc.bwf),
        (false, false) => (tc.fff, tc.bff),
        // inside fov:
        // (true, true) => COLOR_LIGHT_WALL,
        (true, true) => (
            tc.fwt.lerp(tc.fwf, vis_ratio),
            tc.bwt.lerp(tc.bwf, vis_ratio),
        ),
        // (true, false) => COLOR_floor_in_fov,
        (true, false) => (
            tc.fft.lerp(tc.fff, vis_ratio),
            tc.bft.lerp(tc.bff, vis_ratio),
        ),
    };

    // set new background color for object
    object.visual.bg_color =
        ui::Rgba::from_f32(tile_color_bg.r, tile_color_bg.g, tile_color_bg.b, 1.0);

    // if we're dealing with a tile, then change foreground color as well
    if object.tile.is_some() {
        object.visual.fg_color =
            ui::Rgba::from_f32(tile_color_fg.r, tile_color_fg.g, tile_color_fg.b, 1.0);
    }
}

pub struct ShaderCell {
    pub x: i32,
    pub y: i32,
    pub fg_col: rltk::RGBA,
    pub bg_col: rltk::RGBA,
    pub glyph: char,
}

impl ShaderCell {
    pub fn new(x: i32, y: i32) -> Self {
        ShaderCell {
            x,
            y,
            fg_col: rltk::RGBA::from_f32(0.0, 0.0, 0.0, 0.2),
            bg_col: rltk::RGBA::from_f32(0.0, 0.0, 0.0, 0.2),
            glyph: ' ',
        }
    }
}

pub fn create_shader(objects: &ObjectStore) -> Vec<ShaderCell> {
    (0..objects.get_vector().len())
        .into_iter()
        .map(|i| {
            let (x, y) = game::objects::idx_to_coord(consts::WORLD_WIDTH as usize, i);
            ShaderCell::new(x, y)
        })
        .collect()
}

pub fn render_shader(
    shader: &mut Vec<ShaderCell>,
    objects: &ObjectStore,
    ctx: &mut rltk::BTerm,
    vis_update: bool,
) {
    if vis_update {
        shader.iter_mut().for_each(|cell| {
            cell.fg_col.r = 0.0;
            cell.fg_col.g = 0.0;
            cell.fg_col.b = 0.0;
            cell.fg_col.a = 0.2;
            cell.bg_col.r = 0.0;
            cell.bg_col.g = 0.0;
            cell.bg_col.b = 0.0;
            cell.bg_col.a = 0.2;
        });
        let default_range = 4.0;

        objects.get_non_tiles().iter().flatten().for_each(|obj| {
            if !obj.is_player() {
                rltk::field_of_view(
                    rltk::Point::new(obj.pos.x(), obj.pos.y()),
                    default_range as i32,
                    objects,
                )
                .iter()
                .for_each(|point| {
                    let dist = obj.pos.distance(&game::Position::from_xy(point.x, point.y));
                    let is_visible_and_not_wall =
                        if let Some(o) = objects.get_tile_at(point.x, point.y) {
                            o.physics.is_visible && !o.physics.is_blocking_sight
                        } else {
                            false
                        };
                    if dist <= default_range && is_visible_and_not_wall {
                        // get rgb foreground color of object
                        let mut rgba: rltk::RGBA = obj.visual.fg_color.into();
                        // turn it into HSV to easily shift saturation and value
                        let mut hsv: rltk::HSV = rltk::HSV::from(rgba);
                        let percent = 1.0 - (dist / default_range);
                        // hsv.s = 0.50 + (0.5 * percent);
                        hsv.v = 0.8 * percent;
                        // turn it back into rgba to align alpha and print it
                        rgba = hsv.into();
                        rgba.a = 0.2 * (1.0 - percent);
                        let idx =
                            game::objects::coord_to_idx(consts::WORLD_WIDTH, point.x, point.y);
                        let mut adjusted_col = shader[idx].fg_col.lerp(rgba, percent);
                        adjusted_col.a = 0.2;
                        shader[idx].fg_col = adjusted_col;
                        shader[idx].bg_col = adjusted_col;
                    }
                })
            }
        });
    }

    ctx.set_active_console(consts::SHADER_CON);
    let mut draw_batch = rltk::DrawBatch::new();
    shader.iter().for_each(|cell| {
        draw_batch.print_color(
            rltk::Point::new(cell.x, cell.y),
            cell.glyph,
            rltk::ColorPair::new(cell.fg_col, cell.bg_col),
        );
    });

    draw_batch.submit(game::consts::SHADER_CON_Z).unwrap();
}
