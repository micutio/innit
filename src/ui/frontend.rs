use crate::core::innit_env;
use crate::core::position::Position;
use crate::core::world::is_explored;
use crate::entity::object::Object;
use crate::game::{WORLD_HEIGHT, WORLD_WIDTH};
use crate::util::timer::{time_from, Timer};
use crate::{core::game_objects::GameObjects, ui::palette};
use rltk::{field_of_view, to_cp437, ColorPair, DrawBatch, Point, Rect, Rltk, RGB};

pub fn render_world(objects: &mut GameObjects, _ctx: &mut Rltk) {
    // time rendering method for profiling purposes
    let mut timer = Timer::new("render world");
    let mut draw_batch = DrawBatch::new();
    let world_col = palette().world_bg;

    draw_batch.fill_region(
        Rect::with_size(0, 0, WORLD_WIDTH, WORLD_HEIGHT),
        ColorPair::new(world_col, world_col),
        to_cp437(' '),
    );

    update_visibility(objects);

    let mut to_draw: Vec<&Object> = objects
        .get_vector()
        .iter()
        .flatten()
        .filter(|o| {
            // Is there a better way than using `and_then`?
            innit_env().is_debug_mode
                || o.physics.is_visible
                || o.physics.is_always_visible
                || (o.tile.is_some() && *o.tile.as_ref().and_then(is_explored).unwrap())
        })
        .collect();

    // sort, so that non-blocking objects come first
    to_draw.sort_by(|o1, o2| o1.physics.is_blocking.cmp(&o2.physics.is_blocking));
    // draw the objects in the list
    for object in &to_draw {
        draw_batch.set(
            Point::new(object.pos.x, object.pos.y),
            ColorPair::new::<RGB, RGB>(
                object.visual.fg_color.into(),
                object.visual.bg_color.into(),
            ),
            to_cp437(object.visual.glyph),
        );
    }

    let elapsed = timer.stop_silent();
    info!("render world in {}", time_from(elapsed));

    draw_batch.submit(0).unwrap()
}

fn update_visibility(objects: &mut GameObjects) {
    // in debug mode everything is visible
    if innit_env().is_debug_mode {
        let bwft = palette().world_bg_wall_fov_true;
        let bgft = palette().world_bg_ground_fov_true;
        let fwft = palette().world_fg_wall_fov_true;
        let fgft = palette().world_fg_ground_fov_true;
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
    let mut dist_map: Vec<f32> =
        vec![f32::MAX; (WORLD_HEIGHT * WORLD_WIDTH) as usize + WORLD_WIDTH as usize];
    for object_opt in objects.get_vector_mut() {
        if let Some(object) = object_opt {
            object.physics.is_visible = false;
            update_visual(object, -1, Position::default(), &mut dist_map);
        }
    }

    for (pos, range) in player_positions {
        let mut visible_pos = field_of_view(pos.into(), range, objects);
        visible_pos.retain(|p| p.x >= 0 && p.x < WORLD_WIDTH && p.y >= 0 && p.y < WORLD_HEIGHT);

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
    // go through all tiles and set their background color
    let bwft: RGB = palette().world_bg_wall_fov_true.into();
    let bwff: RGB = palette().world_bg_wall_fov_false.into();
    let bgft: RGB = palette().world_bg_ground_fov_true.into();
    let bgff: RGB = palette().world_bg_ground_fov_false.into();
    let fwft: RGB = palette().world_fg_wall_fov_true.into();
    let fwff: RGB = palette().world_fg_wall_fov_false.into();
    let fgft: RGB = palette().world_fg_ground_fov_true.into();
    let fgff: RGB = palette().world_fg_ground_fov_false.into();

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
        if tile.is_explored || innit_env().is_debug_mode {
            // show explored tiles only (any visible tile is explored already)
            object.visual.fg_color = (
                (tile_color_fg.r * 255.0) as u8,
                (tile_color_fg.g * 255.0) as u8,
                (tile_color_fg.b * 255.0) as u8,
            );
            object.visual.bg_color = (
                (tile_color_bg.r * 255.0) as u8,
                (tile_color_bg.g * 255.0) as u8,
                (tile_color_bg.b * 255.0) as u8,
            );
        }
    } else {
        object.visual.bg_color = (
            (tile_color_bg.r * 255.0) as u8,
            (tile_color_bg.g * 255.0) as u8,
            (tile_color_bg.b * 255.0) as u8,
        );
    }
}
