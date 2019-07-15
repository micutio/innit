/// Module Game
/// 
/// This is the top level representation of the game
/// and all its components. Here the components and
/// game loop are constructed and executed.

use tcod::colors;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};

use core::game_state::{GameState, MessageLog, ObjectProcResult, PLAYER};
use core::game_objects::GameObjects;
use entity::action::AttackAction;
use entity::object::Object;
use entity::fighter::{DeathCallback, Fighter};
use ui::game_frontend::{GameFrontend, InputHandler, recompute_fov, re_render, handle_ui_actions};
use ui::game_input::{PlayerAction, get_player_action_instance};

// world constraints
pub const WORLD_WIDTH: i32 = 80;
pub const WORLD_HEIGHT: i32 = 43;

/// Create a new game by instantiating the game engine, game state and object vector.
pub fn new_game() -> (GameState, GameObjects) {
    // create object representing the player
    let mut player = Object::new(0, 0, "player", '@', colors::WHITE, true, false, false);
    player.alive = true;
    player.fighter = Some(Fighter {
        base_max_hp: 100,
        hp: 100,
        base_defense: 1,
        base_power: 2,
        on_death: DeathCallback::Player,
        xp: 0,
    });
    player.attack_action = Some(AttackAction::new(2, 0));

    // create array holding all GameObjects
    let mut objects = GameObjects::new();
    objects.set_player(player);
    let level = 1;

    // create game state holding most game-relevant information
    //  - also creates map and player starting position
    let mut game_state = GameState::new(&mut objects, level);

    // a warm welcoming message
    game_state.log.add(
        "Welcome microbe! You're innit now. Beware of bacteria and viruses",
        colors::RED,
    );

    (game_state, objects)
}

/// Central function of the game.
/// - process player input
/// - render game world
/// - let NPCs take their turn
pub fn game_loop(
    game_state: &mut GameState,
    game_frontend: &mut GameFrontend,
    game_input: &mut InputHandler,
    objects: &mut GameObjects,
) {

    while !game_frontend.root.window_closed() {
        game_input.reset_next_action();
        // let the game engine process an object
        match game_state.process_object(&game_frontend.fov, objects) {
            // no action has been performed, repeat the turn and try again
            ObjectProcResult::NoAction => {}

            // action has been completed, but nothing needs to be done about it
            ObjectProcResult::NoFeedback => {}

            // the player's FOV has been updated, thus we also need to re-render
            ObjectProcResult::UpdateFOV => {
                recompute_fov(game_frontend, objects);
                re_render(
                    game_frontend,
                    game_state,
                    objects,
                    &game_input.names_under_mouse,
                );
            }

            // the player hasn't moved but something happened within fov
            ObjectProcResult::ReRender => {
                re_render(
                    game_frontend,
                    game_state,
                    objects,
                    &game_input.names_under_mouse,
                );
            }

            ObjectProcResult::Animate { anim_type } => {
                // TODO: Play animation.
                println!("animation");
            }

            _ => {}
        }

        // once processing is done, check whether we have a new user input
        game_input.check_for_next_action(game_frontend, game_state, objects);

        // distinguish between in-game action and ui (=meta) actions
        match game_input.get_next_action() {
            Some(PlayerAction::MetaAction(actual_action)) => {
                println!("[game loop] process UI action: {:?}", actual_action);
                let is_exit_game = handle_ui_actions(
                    game_frontend,
                    game_state,
                    objects,
                    &mut Some(game_input),
                    actual_action,
                );
                if is_exit_game {
                    game_input.stop_concurrent_input();
                    break;
                }
            }
            Some(ingame_action) => {
                // let mut player = objects.mut_obj(PLAYER);
                // *player.set_next_action(Some(get_player_action_instance(next_action)));
                // objects.mut_obj(PLAYER).unwrap().set_next_action(Some(get_player_action_instance(next_action)));
                println!(
                    "[game loop] inject ingame action {:?} to player",
                    ingame_action
                );
                if let Some(ref mut player) = objects[PLAYER] {
                    let player_next_action = Some(get_player_action_instance(ingame_action));
                    println!("[game loop] player action object: {:?}", player_next_action);
                    player.set_next_action(player_next_action);
                };
            }
            None => {}
        }

        // level up if needed
        // level_up(objects, game_state, game_frontend);
    }
}

/// Load an existing savegame and instantiates GameState & Objects
/// from which the game is resumed in the game loop.
pub fn load_game() -> Result<(GameState, GameObjects), Box<Error>> {
    let mut json_save_state = String::new();
    let mut file = File::open("savegame")?;
    file.read_to_string(&mut json_save_state)?;
    let result = serde_json::from_str::<(GameState, GameObjects)>(&json_save_state)?;
    Ok(result)
}

/// Serialize and store GameState and Objects into a JSON file.
pub fn save_game(game_state: &GameState, objects: &GameObjects) -> Result<(), Box<Error>> {
    let save_data = serde_json::to_string(&(game_state, objects))?;
    let mut file = File::create("savegame")?;
    file.write_all(save_data.as_bytes())?;
    Ok(())
}
