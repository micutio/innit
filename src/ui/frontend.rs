use crate::core::game_objects::GameObjects;
use crate::core::game_state::{GameState, ObjectFeedback};
use crate::core::position::Position;
use crate::core::world::world_gen::is_explored;
use crate::entity::action::TargetCategory;
use crate::entity::object::Object;
use crate::game::{save_game, Game, RunState, WORLD_HEIGHT, WORLD_WIDTH};
use crate::ui::color::Color;
use crate::ui::color_palette::ColorPalette;
use crate::ui::dialog::character::character_screen;
use crate::ui::game_input::UiAction;
use crate::ui::menus::choose_action_menu::{choose_action_menu, ActionCategory};
use crate::ui::menus::main_menu::main_menu;
use num::Float;
use rltk::{field_of_view, to_cp437, ColorPair, DrawBatch, Point, Rltk, RGB};

pub fn render_world(game: &mut Game, _ctx: &mut Rltk) {
    let mut draw_batch = DrawBatch::new();

    update_visibility(game);

    let mut to_draw: Vec<&Object> = game
        .objects
        .get_vector()
        .iter()
        .flatten()
        .filter(|o| {
            // FIXME: there must be a better way than using `and_then`.
            o.physics.is_visible
                || o.physics.is_always_visible
                || (o.tile.is_some() && *o.tile.as_ref().and_then(is_explored).unwrap())
                || (o.tile.is_some() && game.state.env.debug_mode)
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
}

fn update_visibility(game: &mut Game) {
    let player_positions: Vec<(Position, i32)> = game
        .objects
        .get_vector()
        .iter()
        .flatten()
        .filter(|o| o.is_player())
        .map(|o| (o.pos, o.sensors.sensing_range))
        .collect();

    // set all objects invisible by default
    game.objects.get_vector().iter_mut().flatten().map(|o| {
        o.physics.is_visible = false;
        // TODO: Does this need to be enabled?
        // update_visual(o, &game.color_palette, -1, Position::default());
    });

    let mut dist_map: Vec<f32> = vec![f32::max_value(); (WORLD_HEIGHT * WORLD_WIDTH) as usize];
    for (pos, range) in player_positions {
        let mut visible_pos = field_of_view(pos.to_point(), range, &game.objects);
        visible_pos.retain(|p| p.x >= 0 && p.x < WORLD_WIDTH && p.y >= 0 && p.y < WORLD_HEIGHT);
        game.objects
            .get_vector()
            .iter_mut()
            .flatten()
            .filter(|o| visible_pos.contains(&pos.to_point()))
            .map(|o| {
                o.physics.is_visible = true;
                update_visual(o, &game.color_palette, range, pos, &mut dist_map);
            });
    }
}

/// Update the player's field of view and updated which tiles are visible/explored.
// TODO: This can be moved into a non-frontend module.
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
            fwft.lerp(fwff, dist_map[idx] / player_sensing_range as f32),
            bwft.lerp(bwff, dist_map[idx] / player_sensing_range as f32),
        ),
        // (true, false) => COLOR_ground_in_fov,
        (true, false) => (
            fgft.lerp(fgff, dist_map[idx] / player_sensing_range as f32),
            bgft.lerp(bgff, dist_map[idx] / player_sensing_range as f32),
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
pub fn process_visual_feedback(
    _state: &mut GameState,
    _objects: &mut GameObjects,
    _ctx: &mut Rltk,
    feedback: Vec<ObjectFeedback>,
) {
    for f in feedback {
        match f {
            // no action has been performed, repeat the turn and try again
            ObjectFeedback::NoAction => {}

            // action has been completed, but nothing needs to be done about it
            ObjectFeedback::NoFeedback => {}

            ObjectFeedback::Animate {
                anim_type: _,
                origin: _,
            } => {
                // TODO: Play animation, if origin is in player FOV
                info!("animation");
            }
        }
    }
}

// TODO: Move this somewhere more sensible.
pub fn handle_meta_actions(game: &mut Game, _ctx: &mut Rltk, action: UiAction) -> RunState {
    // TODO: Screens for key mapping, primary and secondary action selection, dna operations.
    debug!("received action {:?}", action);
    match action {
        UiAction::ExitGameLoop => {
            let result = save_game(&game.state, &game.objects);
            result.unwrap();
            RunState::MainMenu(main_menu())
        }
        UiAction::ToggleDarkLightMode => {
            // TODO
            RunState::Ticking
        }
        UiAction::CharacterScreen => {
            RunState::InfoBox(character_screen(&game.state, &game.objects))
        }
        UiAction::ChoosePrimaryAction => {
            if let Some(ref mut player) = game.objects[game.state.current_player_index] {
                let action_items = get_available_actions(
                    player,
                    &[
                        TargetCategory::Any,
                        TargetCategory::EmptyObject,
                        TargetCategory::BlockingObject,
                    ],
                );
                RunState::ChooseActionMenu(choose_action_menu(
                    action_items,
                    ActionCategory::Primary,
                ))
            } else {
                RunState::Ticking
            }
        }
        UiAction::ChooseSecondaryAction => {
            if let Some(ref mut player) = game.objects[game.state.current_player_index] {
                let action_items = get_available_actions(
                    player,
                    &[
                        TargetCategory::Any,
                        TargetCategory::EmptyObject,
                        TargetCategory::BlockingObject,
                    ],
                );
                RunState::ChooseActionMenu(choose_action_menu(
                    action_items,
                    ActionCategory::Secondary,
                ))
            } else {
                RunState::Ticking
            }
        }
        UiAction::ChooseQuick1Action => {
            if let Some(ref mut player) = game.objects[game.state.current_player_index] {
                let action_items = get_available_actions(player, &[TargetCategory::None]);
                RunState::ChooseActionMenu(choose_action_menu(action_items, ActionCategory::Quick1))
            } else {
                RunState::Ticking
            }
        }
        UiAction::ChooseQuick2Action => {
            if let Some(ref mut player) = game.objects[game.state.current_player_index] {
                let action_items = get_available_actions(player, &[TargetCategory::None]);
                RunState::ChooseActionMenu(choose_action_menu(action_items, ActionCategory::Quick2))
            } else {
                RunState::Ticking
            }
        }
    }
    // re_render(state, frontend, objects, "");
    // false
}

fn get_available_actions(obj: &mut Object, targets: &[TargetCategory]) -> Vec<String> {
    obj.actuators
        .actions
        .iter()
        .chain(obj.processors.actions.iter())
        .chain(obj.sensors.actions.iter())
        .filter(|a| targets.contains(&a.get_target_category()))
        .map(|a| a.get_identifier())
        .collect()
}
