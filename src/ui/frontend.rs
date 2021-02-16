use crate::core::game_objects::GameObjects;
use crate::core::game_state::{GameState, ObjectFeedback};
use crate::core::position::Position;
use crate::core::world::world_gen::is_explored;
use crate::entity::object::Object;
use crate::game::{RunState, WORLD_HEIGHT, WORLD_WIDTH};
use crate::ui::color::Color;
use crate::ui::color_palette::ColorPalette;
use crate::ui::menus::game_over_menu::game_over_menu;
use num::Float;
use rltk::{field_of_view, to_cp437, ColorPair, DrawBatch, Point, Rltk, RGB};

pub fn render_world(
    state: &mut GameState,
    objects: &mut GameObjects,
    _ctx: &mut Rltk,
    color_palette: &ColorPalette,
) {
    let mut draw_batch = DrawBatch::new();

    update_visibility(objects, color_palette);

    let mut to_draw: Vec<&Object> = objects
        .get_vector()
        .iter()
        .flatten()
        .filter(|o| {
            // Is there a better way than using `and_then`?
            o.physics.is_visible
                || o.physics.is_always_visible
                || (o.tile.is_some() && *o.tile.as_ref().and_then(is_explored).unwrap())
                || (o.tile.is_some() && state.env.debug_mode)
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
    draw_batch.submit(0).unwrap()
}

fn update_visibility(objects: &mut GameObjects, color_palette: &ColorPalette) {
    let player_positions: Vec<(Position, i32)> = objects
        .get_vector()
        .iter()
        .flatten()
        .filter(|o| o.is_player())
        .map(|o| (o.pos, o.sensors.sensing_range))
        .collect();

    // set all objects invisible by default
    let mut dist_map: Vec<f32> = vec![f32::max_value(); (WORLD_HEIGHT * WORLD_WIDTH) as usize];
    for object_opt in objects.get_vector_mut() {
        if let Some(object) = object_opt {
            object.physics.is_visible = false;
            update_visual(
                object,
                color_palette,
                -1,
                Position::default(),
                &mut dist_map,
            );
        }
    }

    for (pos, range) in player_positions {
        let mut visible_pos = field_of_view(pos.into(), range, objects);
        visible_pos.retain(|p| p.x >= 0 && p.x < WORLD_WIDTH && p.y >= 0 && p.y < WORLD_HEIGHT);

        for object_opt in objects.get_vector_mut() {
            if let Some(object) = object_opt {
                if visible_pos.contains(&object.pos.into()) {
                    object.physics.is_visible = true;
                    update_visual(object, color_palette, range, pos, &mut dist_map);
                }
            }
        }
    }
}

/// Update the player's field of view and updated which tiles are visible/explored.
fn update_visual(
    object: &mut Object,
    coloring: &ColorPalette,
    player_sensing_range: i32,
    player_pos: Position,
    dist_map: &mut Vec<f32>,
) {
    // go through all tiles and set their background color
    let bwft: RGB = coloring.bg_wall_fov_true.into();
    let bwff: RGB = coloring.bg_wall_fov_false.into();
    let bgft: RGB = coloring.bg_ground_fov_true.into();
    let bgff: RGB = coloring.bg_ground_fov_false.into();
    let fwft: RGB = coloring.fg_wall_fov_true.into();
    let fwff: RGB = coloring.fg_wall_fov_false.into();
    let fgft: RGB = coloring.fg_ground_fov_true.into();
    let fgff: RGB = coloring.fg_ground_fov_false.into();

    let wall = object.physics.is_blocking_sight;

    let idx = object.pos.y as usize * (WORLD_WIDTH as usize) + object.pos.x as usize;
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
        if tile.is_explored {
            // show explored tiles only (any visible tile is explored already)
            object.visual.fg_color = Color::from(tile_color_fg);
            object.visual.bg_color = Color::from(tile_color_bg);
        }
    } else {
        object.visual.bg_color = Color::from(tile_color_bg);
    }
}

// TODO: Refactor this to 'process_animations'!
// pub fn visualize_feedback(
//     _state: &mut GameState,
//     _objects: &mut GameObjects,
//     _ctx: &mut Rltk,
//     feedback: Vec<ObjectFeedback>,
// ) -> RunState {
//     let mut re_render = false;
//     for f in feedback {
//         match f {
//             // no action has been performed, repeat the turn and try again
//             ObjectFeedback::NoAction => {}
//
//             // action has been completed, but nothing needs to be done about it
//             ObjectFeedback::NoFeedback => {}
//
//             ObjectFeedback::Render => {
//                 re_render = true;
//             }
//
//             ObjectFeedback::Animate {
//                 anim_type: _,
//                 origin: _,
//             } => {
//                 // TODO: Play animation, if origin is in player FOV
//                 info!("animation");
//                 re_render = true;
//             }
//             ObjectFeedback::GameOver => {
//                 return RunState::GameOver(game_over_menu());
//             }
//         }
//     }
//     // TODO: Change boolean flag to true only if any object feedback
//     RunState::Ticking(re_render)
// }
