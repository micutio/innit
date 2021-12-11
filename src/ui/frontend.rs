use crate::entity::Object;
use crate::game::{self, objects::ObjectStore, position::Position, WORLD_WIDTH};
use crate::ui;
use crate::util::timer;
use crate::world_gen;

use rltk;

pub fn render_world(objects: &mut ObjectStore, _ctx: &mut rltk::Rltk) {
    // time rendering method for profiling purposes
    let mut timer = timer::Timer::new("render world");
    let mut draw_batch = rltk::DrawBatch::new();
    let world_col = ui::palette().world_bg;

    draw_batch.fill_region(
        rltk::Rect::with_size(0, 0, game::WORLD_WIDTH - 1, game::WORLD_HEIGHT - 1),
        rltk::ColorPair::new(world_col, world_col),
        rltk::to_cp437(' '),
    );

    update_visibility(objects);

    let mut to_draw: Vec<&Object> = objects
        .get_vector()
        .iter()
        .flatten()
        .filter(|o| {
            // Is there a better way than using `and_then`?
            game::innit_env().is_debug_mode
                || o.physics.is_visible
                || o.physics.is_always_visible
                || (o.tile.is_some() && *o.tile.as_ref().and_then(world_gen::is_explored).unwrap())
        })
        .collect();

    // sort, so that non-blocking objects come first
    to_draw.sort_by(|o1, o2| o1.physics.is_blocking.cmp(&o2.physics.is_blocking));
    // draw the objects in the list
    for object in &to_draw {
        draw_batch.set(
            rltk::Point::new(object.pos.x, object.pos.y),
            rltk::ColorPair::new(object.visual.fg_color, object.visual.bg_color),
            rltk::to_cp437(object.visual.glyph),
        );
    }

    let elapsed = timer.stop_silent();
    info!("render world in {}", timer::time_to_str(elapsed));

    draw_batch.submit(game::WORLD_CON_Z).unwrap()
}

fn update_visibility(objects: &mut ObjectStore) {
    // in debug mode everything is visible
    if game::innit_env().is_debug_mode {
        let bwft = ui::palette().world_bg_wall_fov_true;
        let bgft = ui::palette().world_bg_ground_fov_true;
        let fwft = ui::palette().world_fg_wall_fov_true;
        let fgft = ui::palette().world_fg_ground_fov_true;
        objects.get_vector_mut().iter_mut().flatten().for_each(|o| {
            // o.physics.is_visible = true;
            if o.tile.is_some() {
                if o.physics.is_blocking_sight {
                    o.visual.fg_color = fwft;
                    o.visual.bg_color = bwft;
                } else {
                    o.visual.fg_color = fgft;
                    o.visual.bg_color = bgft;
                }
            }
        });
        return;
    }

    let player_positions: Vec<(Position, i32)> = objects
        .get_vector()
        .iter()
        .flatten()
        .filter(|o| o.is_player())
        .map(|o| (o.pos, o.sensors.sensing_range))
        .collect();

    // set all objects invisible by default
    let mut dist_map: Vec<f32> = vec![
        f32::MAX;
        (game::WORLD_HEIGHT * game::WORLD_WIDTH) as usize
            + game::WORLD_WIDTH as usize
    ];
    for object_opt in objects.get_vector_mut() {
        if let Some(object) = object_opt {
            object.physics.is_visible = false;
            update_visual(object, -1, Position::default(), &mut dist_map);
        }
    }

    for (pos, range) in player_positions {
        let mut visible_pos = rltk::field_of_view(pos.into(), range, objects);
        visible_pos.retain(|p| {
            p.x >= 0 && p.x < game::WORLD_WIDTH && p.y >= 0 && p.y < game::WORLD_HEIGHT
        });

        for object_opt in objects.get_vector_mut() {
            if let Some(object) = object_opt {
                if visible_pos.contains(&object.pos.into()) {
                    object.physics.is_visible = true;
                    update_visual(object, range, pos, &mut dist_map);
                }
            }
        }
    }
}

/// Update the player's field of view and updated which tiles are visible/explored.
fn update_visual(
    object: &mut Object,
    player_sensing_range: i32,
    player_pos: Position,
    dist_map: &mut Vec<f32>,
) {
    use rltk::{RGB, RGBA};
    // go through all tiles and set their background color
    let bwft = RGB::from(RGBA::from(ui::palette().world_bg_wall_fov_true));
    let bwff = RGB::from(RGBA::from(ui::palette().world_bg_wall_fov_false));
    let bgft = RGB::from(RGBA::from(ui::palette().world_bg_ground_fov_true));
    let bgff = RGB::from(RGBA::from(ui::palette().world_bg_ground_fov_false));
    let fwft = RGB::from(RGBA::from(ui::palette().world_fg_wall_fov_true));
    let fwff = RGB::from(RGBA::from(ui::palette().world_fg_wall_fov_false));
    let fgft = RGB::from(RGBA::from(ui::palette().world_fg_ground_fov_true));
    let fgff = RGB::from(RGBA::from(ui::palette().world_fg_ground_fov_false));

    let wall = object.physics.is_blocking_sight;

    let idx = object.pos.y as usize * (WORLD_WIDTH as usize) + object.pos.x as usize;
    if idx >= dist_map.len() {
        panic!("Invalid object index!");
    }
    dist_map[idx] = dist_map[idx].min(object.pos.distance(&player_pos));

    // set tile foreground and background colors
    let (tile_color_fg, tile_color_bg) = match (object.physics.is_visible, wall) {
        // outside field of view:
        (false, true) => (fwff, bwff),
        (false, false) => (fgff, bgff),
        // inside fov:
        // (true, true) => COLOR_LIGHT_WALL,
        (true, true) => (
            fwft.lerp(fwff, dist_map[idx] / (player_sensing_range + 1) as f32),
            bwft.lerp(bwff, dist_map[idx] / (player_sensing_range + 1) as f32),
        ),
        // (true, false) => COLOR_ground_in_fov,
        (true, false) => (
            fgft.lerp(fgff, dist_map[idx] / (player_sensing_range + 1) as f32),
            bgft.lerp(bgff, dist_map[idx] / (player_sensing_range + 1) as f32),
        ),
    };

    if let Some(tile) = &mut object.tile {
        if object.physics.is_visible {
            tile.is_explored = true;
        }
        if tile.is_explored || game::innit_env().is_debug_mode {
            // show explored tiles only (any visible tile is explored already)
            object.visual.fg_color = (
                (tile_color_fg.r * 255.0) as u8,
                (tile_color_fg.g * 255.0) as u8,
                (tile_color_fg.b * 255.0) as u8,
                255 as u8,
            );
            object.visual.bg_color = (
                (tile_color_bg.r * 255.0) as u8,
                (tile_color_bg.g * 255.0) as u8,
                (tile_color_bg.b * 255.0) as u8,
                255 as u8,
            );
        }
    } else {
        object.visual.bg_color = (
            (tile_color_bg.r * 255.0) as u8,
            (tile_color_bg.g * 255.0) as u8,
            (tile_color_bg.b * 255.0) as u8,
            255 as u8,
        );
    }
}
