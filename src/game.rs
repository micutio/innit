//! The top level representation of the game. Here the major game components are constructed and
//! the game loop is executed.

use crate::core::game_env::GameEnv;
use crate::core::game_objects::GameObjects;
use crate::core::game_state::{GameState, MessageLog, MsgClass, ObjectFeedback};
use crate::core::world::world_gen::WorldGen;
use crate::core::world::world_gen_organic::OrganicsWorldGenerator;
use crate::entity::action::{ActPass, Target, TargetCategory};
use crate::entity::control::Controller;
use crate::entity::genetics::{DnaType, GENE_LEN};
use crate::entity::object::Object;
use crate::entity::player::PlayerCtrl;
use crate::ui::color::Color;
use crate::ui::color_palette::ColorPalette;
use crate::ui::dialog::character::character_screen;
use crate::ui::dialog::InfoBox;
use crate::ui::frontend::render_world;
use crate::ui::game_input::{read_input, PlayerInput, UiAction};
use crate::ui::gui::{render_gui, Hud};
use crate::ui::menus::choose_action_menu::{choose_action_menu, ActionCategory, ActionItem};
use crate::ui::menus::game_over_menu::{game_over_menu, GameOverMenuItem};
use crate::ui::menus::main_menu::{main_menu, MainMenuItem};
use crate::ui::menus::{Menu, MenuItem};
use crate::ui::rex_assets::RexAssets;
use core::fmt;
use rltk::{GameState as Rltk_GameState, Rltk};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::{self, File};
use std::io::{Read, Write};

// environment constraints
// game window
pub const SCREEN_WIDTH: i32 = 100;
pub const SCREEN_HEIGHT: i32 = 60;
// world
pub const WORLD_WIDTH: i32 = 80;
pub const WORLD_HEIGHT: i32 = 60;
// sidebar
pub const SIDE_PANEL_WIDTH: i32 = 20;
pub const SIDE_PANEL_HEIGHT: i32 = 60;
// consoles
pub const WORLD_CON: usize = 0;
pub const HUD_CON: usize = 1;

pub const MENU_WIDTH: i32 = 20;

#[derive(Debug)]
pub enum RunState {
    MainMenu(Menu<MainMenuItem>),
    NewGame,
    LoadGame,
    ChooseActionMenu(Menu<ActionItem>),
    GameOver(Menu<GameOverMenuItem>),
    InfoBox(InfoBox),
    Ticking(bool), // flags to render world, hud
    CheckInput,
    ToggleDarkLightMode,
}

impl Display for RunState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RunState::MainMenu(_) => write!(f, "MainMenu"),
            RunState::NewGame => write!(f, "NewGame"),
            RunState::LoadGame => write!(f, "LoadGame"),
            RunState::ChooseActionMenu(_) => write!(f, "ChooseActionMenu"),
            RunState::GameOver(_) => write!(f, "GameOver"),
            RunState::InfoBox(_) => write!(f, "InfoBox"),
            RunState::Ticking(render) => write!(f, "Ticking({})", render),
            RunState::CheckInput => write!(f, "CheckInput"),
            RunState::ToggleDarkLightMode => write!(f, "ToggleDarkLightMode"),
        }
    }
}

pub struct Game {
    state: GameState,
    objects: GameObjects,
    run_state: Option<RunState>,
    hud: Hud,
    is_dark_color_palette: bool,
    rex_assets: RexAssets,
}

impl Game {
    pub fn new(env: GameEnv) -> Self {
        let state = GameState::new(env, 0);
        let objects = GameObjects::new();

        Game {
            state,
            objects,
            run_state: Some(RunState::MainMenu(main_menu())),
            hud: Hud::new(ColorPalette::get(true)),
            is_dark_color_palette: true,
            rex_assets: RexAssets::new(),
        }
    }

    fn reset(&mut self, state: GameState, objects: GameObjects) {
        self.state = state;
        self.objects = objects;

        if let Some(player) = &self.objects[self.state.player_idx] {
            self.hud
                .update_ui_items(player, ColorPalette::get(self.is_dark_color_palette));
        };
    }

    /// Create a new game by instantiating the game engine, game state and object vector.
    fn new_game(env: GameEnv) -> (GameState, GameObjects) {
        // create game state holding game-relevant information
        let level = 1;
        let mut state = GameState::new(env, level);

        // create blank game world
        let mut objects = GameObjects::new();
        objects.blank_world(&mut state);

        // generate world terrain
        // let mut world_generator = RogueWorldGenerator::new();
        let mut world_generator = OrganicsWorldGenerator::new();
        world_generator.make_world(&mut state, &mut objects, level);
        // objects.set_tile_dna_random(&mut state.rng, &state.gene_library);
        objects.set_tile_dna(
            vec![
                "Cell Membrane".to_string(),
                "Cell Membrane".to_string(),
                "Cell Membrane".to_string(),
                "Energy Store".to_string(),
                "Energy Store".to_string(),
                "Receptor".to_string(),
            ],
            &state.gene_library,
        );

        // create object representing the player
        let (new_x, new_y) = world_generator.get_player_start_pos();
        let player = Object::new()
            .position(new_x, new_y)
            .living(true)
            .visualize("player", '@', Color::from((255, 255, 255)))
            .physical(true, false, true)
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
        // TODO: Create dialog_s with error message!
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
        let mut new_run_state = self.run_state.take().unwrap();
        let color_palette = ColorPalette::get(self.is_dark_color_palette);

        trace!("run state: {}", new_run_state);

        if let RunState::Ticking(render) = new_run_state {
            ctx.set_active_console(HUD_CON);
            ctx.cls();

            if render {
                ctx.set_active_console(WORLD_CON);
                ctx.cls();
                render_world(&mut self.state, &mut self.objects, ctx, color_palette);
            }

            ctx.set_active_console(HUD_CON);
            let player = self
                .objects
                .extract_by_index(self.state.player_idx)
                .unwrap();
            render_gui(&self.state, &mut self.hud, ctx, &color_palette, &player);
            self.state.log.is_changed = false;
            self.objects.replace(self.state.player_idx, player);
        } else if self.hud.require_refresh {
            ctx.set_active_console(HUD_CON);
            ctx.cls();
            ctx.set_active_console(WORLD_CON);
            ctx.cls();
            render_world(&mut self.state, &mut self.objects, ctx, color_palette);
            ctx.set_active_console(HUD_CON);
            let player = self
                .objects
                .extract_by_index(self.state.player_idx)
                .unwrap();
            render_gui(&self.state, &mut self.hud, ctx, &color_palette, &player);
            self.objects.replace(self.state.player_idx, player);
            self.state.log.is_changed = false;
        }

        new_run_state = match new_run_state {
            RunState::MainMenu(ref mut instance) => {
                ctx.set_active_console(WORLD_CON);
                ctx.cls();
                ctx.render_xp_sprite(&self.rex_assets.menu, 0, 0);
                ctx.print_color_centered_at(
                    SCREEN_WIDTH / 2,
                    SCREEN_HEIGHT - 2,
                    color_palette.yellow,
                    color_palette.bg_hud,
                    "by Michael Wagner",
                );
                match instance.display(ctx, color_palette) {
                    Some(option) => {
                        MainMenuItem::process(&mut self.state, &mut self.objects, instance, &option)
                    }
                    None => RunState::MainMenu(instance.clone()),
                }
            }
            RunState::GameOver(ref mut instance) => {
                ctx.set_active_console(WORLD_CON);
                ctx.cls();
                ctx.render_xp_sprite(&self.rex_assets.menu, 0, 0);
                ctx.print_color_centered_at(
                    SCREEN_WIDTH / 2,
                    1,
                    color_palette.yellow,
                    color_palette.bg_hud,
                    "GAME OVER", // TODO: Add more text
                );
                match instance.display(ctx, color_palette) {
                    Some(option) => GameOverMenuItem::process(
                        &mut self.state,
                        &mut self.objects,
                        instance,
                        &option,
                    ),
                    None => RunState::GameOver(instance.clone()),
                }
            }
            RunState::ChooseActionMenu(ref mut instance) => {
                match instance.display(ctx, color_palette) {
                    Some(option) => {
                        ActionItem::process(&mut self.state, &mut self.objects, instance, &option)
                    }
                    None => RunState::ChooseActionMenu(instance.clone()),
                }
            }
            RunState::Ticking(_) => {
                let mut feedback;
                // Let the game engine process objects until we have to re-render the world or UI.
                // Re-rendering is necessary either because the world changed or messages need to
                // be printed to the log.
                'processing: loop {
                    feedback = self.state.process_object(&mut self.objects);
                    println!("processing: feedback={:#?}", &feedback);
                    // match feedback {
                    //     // ObjectFeedback::NoAction => {}
                    //     ObjectFeedback::NoFeedback => {}
                    //     _ => break 'processing,
                    // }
                    if feedback != ObjectFeedback::NoFeedback || self.state.log.is_changed {
                        break 'processing;
                    }
                }

                match feedback {
                    ObjectFeedback::GameOver => RunState::GameOver(game_over_menu()),
                    ObjectFeedback::Render => RunState::Ticking(true),
                    // if there is no reason to re-render, check whether we're waiting on user input
                    _ => {
                        if self.state.is_players_turn()
                            && self.state.player_energy_full(&self.objects)
                        {
                            println!(">>>>>>> checking input");
                            RunState::CheckInput
                        } else {
                            println!(">>>>>>> ticking");
                            RunState::Ticking(false)
                        }
                    }
                }
            }
            RunState::CheckInput => {
                match read_input(&mut self.state, &mut self.objects, &mut self.hud, ctx) {
                    PlayerInput::MetaInput(meta_action) => {
                        trace!("process meta action: {:#?}", meta_action);
                        handle_meta_actions(&mut self.state, &mut self.objects, ctx, meta_action)
                    }
                    PlayerInput::PlayInput(in_game_action) => {
                        trace!("inject in-game action {:#?} to player", in_game_action);
                        if let Some(ref mut player) = self.objects[self.state.player_idx] {
                            use crate::ui::game_input::PlayerAction::*;
                            let a = match in_game_action {
                                PrimaryAction(dir) => player.get_primary_action(dir),
                                SecondaryAction(dir) => player.get_secondary_action(dir),
                                Quick1Action => player.get_quick1_action(),
                                Quick2Action => player.get_quick2_action(),
                                PassTurn => Box::new(ActPass),
                            };
                            // TODO: Turn this into a queue to detach action sources from user input!
                            player.set_next_action(Some(a));
                            RunState::Ticking(false)
                        } else {
                            RunState::Ticking(false)
                        }
                    }
                    PlayerInput::Undefined => RunState::CheckInput,
                }
            }
            RunState::InfoBox(infobox) => {
                match infobox.display(ctx, ColorPalette::get(self.is_dark_color_palette)) {
                    Some(infobox) => RunState::InfoBox(infobox),
                    None => RunState::Ticking(false),
                }
            }
            RunState::ToggleDarkLightMode => {
                self.is_dark_color_palette = !self.is_dark_color_palette;
                RunState::Ticking(true)
            }
            RunState::NewGame => {
                // start new game
                let (new_state, new_objects) = Game::new_game(self.state.env);
                self.reset(new_state, new_objects);
                RunState::Ticking(true)
            }
            RunState::LoadGame => {
                // load game from file
                match load_game() {
                    Ok((state, objects)) => {
                        self.reset(state, objects);
                        RunState::Ticking(true)
                    }
                    Err(_e) => {
                        // TODO: Show alert to user... or not? Maybe have inactive buttons.
                        // msg_box(frontend, &mut None, "", "\nNo saved game to load\n", 24);
                        RunState::MainMenu(main_menu())
                    }
                }
            }
        };
        self.run_state.replace(new_run_state);

        ctx.set_active_console(WORLD_CON);
        ctx.print(1, 1, &format!("FPS: {}", ctx.fps));
        // ctx.set_active_console(HUD_CON);
        // ctx.print(0, 0, "");
        rltk::render_draw_buffer(ctx).unwrap();
        // debug!("current state: {:#?}", self.run_state);
    }
}

pub fn handle_meta_actions(
    state: &mut GameState,
    objects: &mut GameObjects,
    _ctx: &mut Rltk,
    action: UiAction,
) -> RunState {
    debug!("received action {:?}", action);
    match action {
        UiAction::ExitGameLoop => {
            let result = save_game(&state, &objects);
            result.unwrap();
            RunState::MainMenu(main_menu())
        }
        UiAction::ToggleDarkLightMode => {
            // game.toggle_dark_light_mode();
            // RunState::Ticking(true)
            RunState::ToggleDarkLightMode
        }
        UiAction::CharacterScreen => RunState::InfoBox(character_screen(state, objects)),
        UiAction::ChoosePrimaryAction => {
            if let Some(ref mut player) = objects[state.player_idx] {
                let action_items = get_available_actions(
                    player,
                    &[
                        TargetCategory::Any,
                        TargetCategory::EmptyObject,
                        TargetCategory::BlockingObject,
                    ],
                );
                if !action_items.is_empty() {
                    RunState::ChooseActionMenu(choose_action_menu(
                        action_items,
                        ActionCategory::Primary,
                    ))
                } else {
                    state.log.add(
                        "You have no actions available! Try modifying your genome.",
                        MsgClass::Alert,
                    );
                    RunState::Ticking(false)
                }
            } else {
                RunState::Ticking(false)
            }
        }
        UiAction::ChooseSecondaryAction => {
            if let Some(ref mut player) = objects[state.player_idx] {
                let action_items = get_available_actions(
                    player,
                    &[
                        TargetCategory::Any,
                        TargetCategory::EmptyObject,
                        TargetCategory::BlockingObject,
                    ],
                );
                if !action_items.is_empty() {
                    RunState::ChooseActionMenu(choose_action_menu(
                        action_items,
                        ActionCategory::Secondary,
                    ))
                } else {
                    state.log.add(
                        "You have no actions available! Try modifying your genome.",
                        MsgClass::Alert,
                    );
                    RunState::Ticking(false)
                }
            } else {
                RunState::Ticking(false)
            }
        }
        UiAction::ChooseQuick1Action => {
            if let Some(ref mut player) = objects[state.player_idx] {
                let action_items = get_available_actions(player, &[TargetCategory::None]);
                if !action_items.is_empty() {
                    RunState::ChooseActionMenu(choose_action_menu(
                        action_items,
                        ActionCategory::Quick1,
                    ))
                } else {
                    state.log.add(
                        "You have no actions available! Try modifying your genome.",
                        MsgClass::Alert,
                    );
                    RunState::Ticking(false)
                }
            } else {
                RunState::Ticking(false)
            }
        }
        UiAction::ChooseQuick2Action => {
            if let Some(ref mut player) = objects[state.player_idx] {
                let action_items = get_available_actions(player, &[TargetCategory::None]);
                if !action_items.is_empty() {
                    RunState::ChooseActionMenu(choose_action_menu(
                        action_items,
                        ActionCategory::Quick2,
                    ))
                } else {
                    state.log.add(
                        "You have no actions available! Try modifying your genome.",
                        MsgClass::Alert,
                    );
                    RunState::Ticking(false)
                }
            } else {
                RunState::Ticking(false)
            }
        }
    }
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
