//! The top level representation of the game. Here the major game components are constructed and
//! the game loop is executed.

use crate::core::game_env::GameEnv;
use crate::core::game_objects::GameObjects;
use crate::core::game_state::{GameState, MessageLog, MsgClass};
use crate::core::world::world_gen::WorldGen;
use crate::core::world::world_gen_organic::OrganicsWorldGenerator;
use crate::entity::action::{ActPass, Target};
use crate::entity::control::Controller;
use crate::entity::genetics::{DnaType, GENE_LEN};
use crate::entity::object::Object;
use crate::entity::player::PlayerCtrl;
use crate::ui::color_palette::ColorPalette;
use crate::ui::frontend::render;
use crate::ui::game_frontend::{handle_meta_actions, process_visual_feedback};
use crate::ui::game_input::{GameInput, PlayerInput};
use crate::ui::menu::{display_main_menu, MenuInstance};
use rltk::{GameState as Rltk_GameState, Rltk, RGB};
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::time::Duration;

pub const MS_PER_FRAME: Duration = Duration::from_millis(16.0 as u64);

// environment constraints
// game window
pub const SCREEN_WIDTH: i32 = 160;
pub const SCREEN_HEIGHT: i32 = 90;
// world
pub const WORLD_WIDTH: i32 = 110;
pub const WORLD_HEIGHT: i32 = 90;
// sidebar
pub const SIDE_PANEL_WIDTH: i32 = 50;
pub const SIDE_PANEL_HEIGHT: i32 = 90;
pub const LIMIT_FPS: i32 = 60; // target fps

pub(crate) enum RunState {
    Menu(MenuInstance),
    Ticking,
    CheckInput,
}

pub struct Game {
    pub state: GameState,
    pub objects: GameObjects,
    pub input: GameInput,
    pub run_state: RunState,
    pub color_palette: ColorPalette,
    is_light_mode: bool,
}

impl Game {
    pub fn new(env: GameEnv, ctx: &mut Rltk, color_palette: ColorPalette) -> Self {
        let (state, objects) = Game::new_game(env, ctx);
        Game {
            state,
            objects,
            input: GameInput::new(),
            run_state: RunState::Menu(MenuInstance::MainMenu(None)),
            color_palette,
            is_light_mode: false,
        }
    }

    pub fn reset(&mut self, state: GameState, objects: GameObjects) {
        self.state = state;
        self.objects = objects;
    }

    /// Create a new game by instantiating the game engine, game state and object vector.
    pub fn new_game(env: GameEnv, ctx: &mut Rltk) -> (GameState, GameObjects) {
        // create game state holding game-relevant information
        let level = 1;
        let mut state = GameState::new(env, level);

        // create blank game world
        let mut objects = GameObjects::new();
        objects.blank_world(&mut state);

        // generate world terrain
        // let mut world_generator = RogueWorldGenerator::new();
        let mut world_generator = OrganicsWorldGenerator::new();
        world_generator.make_world(&mut state, ctx, &mut objects, level);
        // objects.set_tile_dna_random(&mut state.rng, &state.gene_library);
        objects.set_tile_dna(
            vec![
                "cell membrane".to_string(),
                "cell membrane".to_string(),
                "cell membrane".to_string(),
                "energy store".to_string(),
                "energy store".to_string(),
                "receptor".to_string(),
            ],
            &state.gene_library,
        );

        // create object representing the player
        let (new_x, new_y) = world_generator.get_player_start_pos();
        let player = Object::new()
            .position(new_x, new_y)
            .living(true)
            .visualize("player", '@', RGB::from_u8(255, 255, 255))
            .physical(true, false, false)
            .control(Controller::Player(PlayerCtrl::new()))
            .genome(
                0.99,
                state
                    .gene_library
                    .new_genetics(&mut state.rng, DnaType::Nucleus, false, GENE_LEN),
            );

        trace!("created player object {}", player);
        trace!("player sensors: {:?}", player.sensors);
        trace!("player processors: {:?}", player.processors);
        trace!("player actuators: {:?}", player.actuators);
        trace!("player dna: {:?}", player.dna);
        trace!(
            "player default action: {:?}",
            player.get_primary_action(Target::Center).to_text()
        );
        objects.set_player(player);

        // a warm welcoming message
        state.log.add(
            "Welcome microbe! You're innit now. Beware of bacteria and viruses",
            MsgClass::Story,
        );

        (state, objects)
    }

    pub fn toggle_dark_light_mode(&mut self) {
        if self.is_light_mode {
            self.is_light_mode = false;
            self.color_palette = ColorPalette::dark();
        } else {
            self.is_light_mode = true;
            self.color_palette = ColorPalette::light();
        }
    }
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
pub fn save_game(state: &GameState, objects: &GameObjects) -> Result<(), Box<dyn Error>> {
    if let Some(mut env_data) = dirs::data_local_dir() {
        env_data.push("innit");
        fs::create_dir_all(&env_data)?;
        env_data.push("savegame");

        let mut save_file = File::create(env_data)?;
        let save_data = serde_json::to_string(&(state, objects))?;
        save_file.write_all(save_data.as_bytes())?;
        debug!("SAVED GAME TO FILE");
        Ok(())
    } else {
        // TODO: Create dialog with error message!
        error!("CANNOT CREATE SAVE FILE!");
        Ok(())
    }
}

impl Rltk_GameState for Game {
    /// Central function of the game.
    /// - process player input
    /// - render game world
    /// - let NPCs take their turn
    fn tick(&mut self, ctx: &mut Rltk) {
        // let mut start_time = Instant::now();
        // ensure that the player action from previous turns is consumed
        assert!(self.input.is_action_consumed());

        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_active_console(0);
        ctx.cls();

        // render everything
        render(self, ctx);

        self.run_state = match &self.run_state {
            RunState::Menu(MenuInstance::MainMenu(mut instance)) => {
                display_main_menu(self, ctx, &self.color_palette, instance.take())
            }
            RunState::Ticking => {
                // let the game engine process an object
                let action_feedback = self.state.process_object(&mut self.objects);

                if action_feedback.is_empty() {
                    RunState::Ticking
                } else {
                    // render action vfx
                    process_visual_feedback(
                        &mut self.state,
                        &self.input,
                        &mut self.objects,
                        ctx,
                        action_feedback,
                    );
                    RunState::CheckInput
                }
            }
            RunState::CheckInput => {
                // once processing is done, check whether we have a new user input
                self.input
                    .check_for_player_actions(&mut self.state, &mut self.objects, ctx);

                // distinguish between in-game action and ui (=meta) actions
                // TODO: Enable multi-key/mouse actions e.g., select target & attack.
                match self.input.get_next_action() {
                    Some(PlayerInput::MetaInput(meta_action)) => {
                        debug!("process meta action: {:#?}", meta_action);
                        handle_meta_actions(
                            &mut self.state,
                            &mut self.objects,
                            &mut Some(&mut self.input),
                            ctx,
                            meta_action,
                        )
                    }
                    Some(PlayerInput::PlayInput(in_game_action)) => {
                        trace!("inject in-game action {:#?} to player", in_game_action);
                        if let Some(ref mut player) = self.objects[self.state.current_player_index]
                        {
                            use crate::ui::game_input::PlayerAction::*;
                            let a = match in_game_action {
                                PrimaryAction(dir) => player.get_primary_action(dir),
                                SecondaryAction(dir) => player.get_secondary_action(dir),
                                Quick1Action() => player.get_quick1_action(),
                                Quick2Action() => player.get_quick2_action(),
                                PassTurn => Box::new(ActPass),
                            };
                            player.set_next_action(Some(a));
                            RunState::Ticking
                        } else {
                            RunState::Ticking
                        }
                    }
                    None => RunState::Ticking,
                }
            }
        };

        ctx.print(1, 1, &format!("FPS: {}", ctx.fps));
    }
}
