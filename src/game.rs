//! The top level representation of the game. Here the major game components are constructed and
//! the game loop is executed.

use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::thread::{self};
use std::time::{Duration, Instant};

use tcod::colors;

use crate::core::game_env::GameEnv;
use crate::core::game_objects::GameObjects;
use crate::core::game_state::{GameState, MessageLog, ObjectProcResult};
use crate::core::world::world_gen::WorldGen;
use crate::core::world::world_gen_organic::OrganicsWorldGenerator;
use crate::entity::action::{PassAction, Target};
use crate::entity::genetics::GENE_LEN;
use crate::entity::object::Object;
use crate::entity::player::PLAYER;
use crate::ui::game_frontend::{handle_meta_actions, process_visual_feedback, GameFrontend};
use crate::ui::game_input::{GameInput, PlayerInput};

pub const MS_PER_FRAME: Duration = Duration::from_millis(16.0 as u64);

// world constraints
pub const WORLD_WIDTH: i32 = 80;
pub const WORLD_HEIGHT: i32 = 43;

/// Create a new game by instantiating the game engine, game state and object vector.
pub fn new_game(env: GameEnv, game_frontend: &mut GameFrontend) -> (GameState, GameObjects) {
    // create game state holding game-relevant information
    let level = 1;
    let mut game_state = GameState::new(env, level);

    // create blank game world
    let mut game_objects = GameObjects::new();
    game_objects.blank_world(&env);

    // generate world terrain
    // let mut world_generator = RogueWorldGenerator::new();
    let mut world_generator = OrganicsWorldGenerator::new();
    world_generator.make_world(
        &env,
        game_frontend,
        &mut game_objects,
        &mut game_state.rng,
        &mut game_state.gene_library,
        level,
    );
    game_objects.set_tiles_dna(&mut game_state.rng, &game_state.gene_library);

    // create object representing the player
    let (new_x, new_y) = world_generator.get_player_start_pos();
    let player = Object::new()
        .position(new_x, new_y)
        .living(true)
        .visualize("player", '@', colors::WHITE)
        .physical(true, false, false)
        .genome(
            0.99,
            game_state
                .gene_library
                .new_genetics(&mut game_state.rng, GENE_LEN),
        );

    debug!("created player object {}", player);
    debug!("player sensors: {:?}", player.sensors);
    debug!("player processors: {:?}", player.processors);
    debug!("player actuators: {:?}", player.actuators);
    debug!("player dna: {:?}", player.dna);
    debug!(
        "player default action: {:?}",
        player.get_primary_action(Target::Center).to_text()
    );
    game_objects.set_player(player);

    // a warm welcoming message
    game_state.log.add(
        "Welcome microbe! You're innit now. Beware of bacteria and viruses",
        game_frontend.coloring.fg_dialog_border,
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
    // use cpuprofiler::PROFILER;
    // PROFILER.lock().unwrap().start("./profile.out");

    let mut start_time = Instant::now();
    while !game_frontend.root.window_closed() {
        // ensure that the player action from previous turns is consumed
        assert!(game_input.is_action_consumed());

        // let the game engine process an object
        let action_result = game_state.process_object(game_objects, &game_frontend.fov);

        // limit frames
        if game_state.is_players_turn() {
            if let ObjectProcResult::NoAction = action_result {
                let elapsed = start_time.elapsed();
                // println!("time since last inactive: {:#?}", elapsed);
                if let Some(slow_down) = MS_PER_FRAME.checked_sub(elapsed) {
                    // println!("sleep for {:#?}", slow_down);
                    thread::sleep(slow_down);
                }
                start_time = Instant::now();
            }
        }

        // render action vfx
        process_visual_feedback(
            game_state,
            game_frontend,
            game_input,
            game_objects,
            action_result,
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
            Some(PlayerInput::PlayInput(in_game_action)) => {
                trace!("inject in-game action {:#?} to player", in_game_action);
                if let Some(ref mut player) = game_objects[PLAYER] {
                    use crate::ui::game_input::PlayerAction::*;
                    let a = match in_game_action {
                        PrimaryAction(dir) => player.get_primary_action(dir),
                        SecondaryAction(dir) => player.get_secondary_action(dir),
                        Quick1Action() => player.get_quick1_action(),
                        Quick2Action() => player.get_quick2_action(),
                        PassTurn => Box::new(PassAction),
                    };
                    player.set_next_action(Some(a))
                }
            }
            None => {
                trace!("no player input detected");
            }
        }

        // sync game loop to 60 fps to avoid eating the CPU alive
        // if let Some(time_to_next_step) = MS_PER_FRAME.checked_sub(start_time.elapsed()) {
        //     println!("sleep for {:#?}", time_to_next_step);
        //     thread::sleep(time_to_next_step);
        // }
    }
    // PROFILER.lock().unwrap().stop();
}

/// Load an existing savegame and instantiates GameState & Objects
/// from which the game is resumed in the game loop.
pub fn load_game() -> Result<(GameState, GameObjects), Box<dyn Error>> {
    // TODO: Add proper UI error output if any of this fails!
    if let Some(mut save_file) = dirs::data_local_dir() {
        save_file.push("innit");
        save_file.push("savegame");
        let mut file = File::open(save_file)?;
        let mut json_save_state = String::new();
        file.read_to_string(&mut json_save_state)?;
        let result = serde_json::from_str::<(GameState, GameObjects)>(&json_save_state)?;
        Ok(result)
    } else {
        error!("CANNOT ACCESS SYSTEM DATA DIR");
        panic!("CANNOT ACCESS SYSTEM DATA DIR");
    }
}

/// Serialize and store GameState and Objects into a JSON file.
pub fn save_game(game_state: &GameState, objects: &GameObjects) -> Result<(), Box<dyn Error>> {
    if let Some(mut env_data) = dirs::data_local_dir() {
        env_data.push("innit");
        fs::create_dir_all(&env_data)?;
        env_data.push("savegame");

        let mut save_file = File::create(env_data)?;
        let save_data = serde_json::to_string(&(game_state, objects))?;
        save_file.write_all(save_data.as_bytes())?;
        debug!("SAVED GAME TO FILE");
        Ok(())
    } else {
        // TODO: Create dialog with error message!
        error!("CANNOT CREATE SAVE FILE!");
        Ok(())
    }
}
