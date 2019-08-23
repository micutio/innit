//! The top level representation of the game. Here the major game components are constructed and
//! the game loop is executed.

use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};

use tcod::colors;

use crate::core::game_objects::GameObjects;
use crate::core::game_state::{GameState, MessageLog};
use crate::core::world::world_gen::WorldGen;
use crate::core::world::world_gen_rogue::RogueWorldGenerator;
use crate::entity::dna::{build_player_action, SubTrait, SuperTrait};
use crate::entity::object::Object;
use crate::ui::game_frontend::{handle_meta_actions, process_visual_feedback, GameFrontend};
use crate::ui::game_input::{GameInput, PlayerInput};
use crate::ui::player::PLAYER;

const SAVEGAME: &str = "data/savegame";

// world constraints
pub const WORLD_WIDTH: i32 = 80;
pub const WORLD_HEIGHT: i32 = 43;

/// Create a new game by instantiating the game engine, game state and object vector.
pub fn new_game() -> (GameState, GameObjects) {
    // create game state holding game-relevant information
    let level = 1;
    let mut game_state = GameState::new(level);

    // create blank game world
    let mut game_objects = GameObjects::new();
    game_objects.init_world(&mut game_state.game_rng, &mut game_state.gene_library);

    // create object representing the player
    // TODO: store player starting position in world generator and set player object later
    let dna = game_state
        .gene_library
        .new_dna(&mut game_state.game_rng, 10);
    let (sensors, processors, actuators) = game_state.gene_library.decode_dna(&dna);
    let mut player = Object::new(
        0,
        0,
        dna,
        "player",
        '@',
        colors::WHITE,
        true,
        false,
        false,
        sensors,
        processors,
        actuators,
        None,
    );
    player.alive = true;
    game_objects.set_player(player);

    // generate world terrain
    let mut world_generator = RogueWorldGenerator::new();
    world_generator.make_world(
        &mut game_objects,
        &mut game_state.game_rng,
        &mut game_state.gene_library,
        level,
    );

    // a warm welcoming message
    game_state.log.add(
        "Welcome microbe! You're innit now. Beware of bacteria and viruses",
        colors::RED,
    );

    (game_state, game_objects)
}

/// Central function of the game.
/// - process player input
/// - render game world
/// - let NPCs take their turn
pub fn game_loop(
    game_state: &mut GameState,
    game_frontend: &mut GameFrontend,
    game_input: &mut GameInput,
    game_objects: &mut GameObjects,
) {
    while !game_frontend.root.window_closed() {
        // ensure that the player action from previous turns is consumed
        assert!(game_input.is_action_consumed());

        // let the game engine process an object
        let process_result = game_state.process_object(game_objects, &game_frontend.fov);
        process_visual_feedback(
            game_state,
            game_frontend,
            game_input,
            game_objects,
            process_result,
        );

        // once processing is done, check whether we have a new user input
        game_input.check_for_player_actions(game_state, game_frontend, game_objects);

        // distinguish between in-game action and ui (=meta) actions
        // TODO: Enable multi-key/mouse actions e.g., select target & attack.
        match game_input.get_next_action() {
            Some(PlayerInput::MetaInput(meta_action)) => {
                debug!("process meta action: {:#?}", meta_action);
                let is_exit_game = handle_meta_actions(
                    game_frontend,
                    game_state,
                    game_objects,
                    &mut Some(game_input),
                    meta_action,
                );
                if is_exit_game {
                    game_input.stop_concurrent_input();
                    break;
                }
            }
            Some(PlayerInput::PlayInput(ingame_action)) => {
                debug!("inject ingame action {:#?} to player", ingame_action);
                if let Some(ref mut player) = game_objects[PLAYER] {
                    use self::SuperTrait::*;
                    if let SubTrait::StAction(action_trait) = ingame_action.trait_id {
                        match game_state
                            .gene_library
                            .trait_to_super
                            .get(&ingame_action.trait_id)
                        {
                            Some(Sensing) => {
                                if let Some(prototype) = player
                                    .sensors
                                    .actions
                                    .iter()
                                    .find(|a| a.trait_id == action_trait)
                                {
                                    let next_action =
                                        Some(build_player_action(ingame_action, prototype));
                                    debug!("player action object: {:#?}", next_action);
                                    player.set_next_action(next_action);
                                }
                            }
                            Some(Processing) => {
                                if let Some(prototype) = player
                                    .processors
                                    .actions
                                    .iter()
                                    .find(|a| a.trait_id == action_trait)
                                {
                                    let next_action =
                                        Some(build_player_action(ingame_action, prototype));
                                    debug!("player action object: {:#?}", next_action);
                                    player.set_next_action(next_action);
                                }
                            }
                            Some(Actuating) => {
                                if let Some(prototype) = player
                                    .actuators
                                    .actions
                                    .iter()
                                    .find(|a| a.trait_id == action_trait)
                                {
                                    let next_action =
                                        Some(build_player_action(ingame_action, prototype));
                                    debug!("player action object: {:#?}", next_action);
                                    player.set_next_action(next_action);
                                }
                            }
                            None => {}
                        }
                    }

                    // match ingame_action.trait_id {
                    //     StAction(Move) => {

                    //     }
                    //     _ => {}
                    // }

                    // debug!(
                    //     "attempting to get prototype {:#?}",
                    //     player
                    //         .actions
                    //         .iter()
                    //         .find(|a| a.trait_id == ingame_action.trait_id)
                    // );
                    // if let Some(prototype) = player
                    //     .actions
                    //     .iter()
                    //     .find(|a| a.trait_id == ingame_action.trait_id)
                    // {
                    //     let next_action = Some(build_player_action(ingame_action, prototype));
                    //     debug!("player action object: {:#?}", next_action);
                    //     player.set_next_action(next_action);
                    // }
                };
            }
            None => {}
        }
    }
}

/// Load an existing savegame and instantiates GameState & Objects
/// from which the game is resumed in the game loop.
pub fn load_game() -> Result<(GameState, GameObjects), Box<dyn Error>> {
    let mut json_save_state = String::new();
    let mut file = File::open(SAVEGAME)?;
    file.read_to_string(&mut json_save_state)?;
    let result = serde_json::from_str::<(GameState, GameObjects)>(&json_save_state)?;
    Ok(result)
}

/// Serialize and store GameState and Objects into a JSON file.
pub fn save_game(game_state: &GameState, objects: &GameObjects) -> Result<(), Box<dyn Error>> {
    let save_data = serde_json::to_string(&(game_state, objects))?;
    let mut file = File::create(SAVEGAME)?;
    file.write_all(save_data.as_bytes())?;
    Ok(())
}
