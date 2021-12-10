//! The top level representation of the game. Here the major game components are constructed and
//! the game loop is executed.

use crate::core::game_objects::GameObjects;
use crate::core::game_state::{GameState, MessageLog, MsgClass, ObjectFeedback};
use crate::core::innit_env;
use crate::core::world::world_gen_organic::OrganicsWorldGenerator;
use crate::core::world::WorldGen;
use crate::entity::action::hereditary::ActPass;
use crate::entity::action::inventory::ActDropItem;
use crate::entity::action::{Action, Target, TargetCategory};
use crate::entity::control::Controller;
use crate::entity::genetics::{DnaType, TraitFamily, GENOME_LEN};
use crate::entity::object::Object;
use crate::entity::player::PlayerCtrl;
use crate::raws::object_template::ObjectTemplate;
use crate::raws::spawn::Spawn;
use crate::raws::{load_object_templates, load_spawns};
use crate::ui::custom::genome_editor::{GenomeEditingState, GenomeEditor};
use crate::ui::dialog::character::character_screen;
use crate::ui::dialog::controls::controls_screen;
use crate::ui::dialog::InfoBox;
use crate::ui::frontend::render_world;
use crate::ui::game_input::{read_input, PlayerInput, UiAction};
use crate::ui::hud::{render_gui, Hud};
use crate::ui::menu::choose_action_menu::{choose_action_menu, ActionCategory, ActionItem};
use crate::ui::menu::game_over_menu::{game_over_menu, GameOverMenuItem};
use crate::ui::menu::main_menu::{main_menu, MainMenuItem};
use crate::ui::menu::{Menu, MenuItem};
use crate::ui::palette;
use crate::ui::particles;
use crate::ui::rex_assets::RexAssets;
use crate::util::timer::{time_from, Timer};
use core::fmt;
use rltk::{to_cp437, ColorPair, Degrees, DrawBatch, GameState as Rltk_GameState, Rltk};
use std::error::Error;
use std::fmt::{Display, Formatter};
#[cfg(not(target_arch = "wasm32"))]
use std::fs::{self, File};
#[cfg(not(target_arch = "wasm32"))]
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
pub const WORLD_CON_Z: usize = 1000;
pub const HUD_CON: usize = 1;
pub const HUD_CON_Z: usize = 10000;
pub const PAR_CON: usize = 2;
pub const PAR_CON_Z: usize = 20000;

pub const MENU_WIDTH: i32 = 20;

#[derive(Debug)]
pub enum RunState {
    MainMenu(Menu<MainMenuItem>),
    NewGame,
    LoadGame,
    ChooseActionMenu(Menu<ActionItem>),
    GameOver(Menu<GameOverMenuItem>),
    InfoBox(InfoBox),
    GenomeEditing(GenomeEditor),
    Ticking,
    CheckInput,
    WorldGen,
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
            RunState::GenomeEditing(_) => write!(f, "GenomeEditing"),
            RunState::Ticking => write!(f, "Ticking"),
            RunState::CheckInput => write!(f, "CheckInput"),
            RunState::WorldGen => write!(f, "WorldGen"),
        }
    }
}

pub struct Game {
    state: GameState,
    objects: GameObjects,
    run_state: Option<RunState>,
    // world generation state start
    spawns: Vec<Spawn>,
    object_templates: Vec<ObjectTemplate>,
    // world_generator = RogueWorldGenerator::new();
    world_generator: OrganicsWorldGenerator,
    // world generation state end
    hud: Hud,
    re_render: bool,
    rex_assets: RexAssets,
    /// This workaround is required because each mouse click is registered twice (press & release),
    /// Without it each mouse event is fired twice in a row and toggles are useless.
    mouse_workaround: bool,
    /// Keep track of the time to warn if the game runs too slow.
    slowest_tick: u128,
}

impl Game {
    pub fn new() -> Self {
        let state = GameState::new(1);
        let objects = GameObjects::new();

        Game {
            state,
            objects,
            // spawns: load_spawns(),
            // object_templates: load_object_templates(),
            run_state: Some(RunState::MainMenu(main_menu())),
            spawns: load_spawns(),
            object_templates: load_object_templates(),

            // let mut world_generator : RogueWorldGenerator::new(),
            world_generator: OrganicsWorldGenerator::new(),
            hud: Hud::new(),
            re_render: false,
            rex_assets: RexAssets::new(),
            mouse_workaround: false,
            slowest_tick: 0,
        }
    }

    fn reset(&mut self, state: GameState, objects: GameObjects) {
        self.state = state;
        self.objects = objects;

        if let Some(player) = &self.objects[self.state.player_idx] {
            self.hud.update_ui_items(player);
        };
    }

    /// Create a new game by instantiating the game engine, game state and object vector.
    fn new_game() -> (GameState, GameObjects) {
        // create game state holding game-relevant information
        let state = GameState::new(1);

        // initialise game object vector
        let mut objects = GameObjects::new();
        objects.blank_world();

        // prepare world generation
        // load spawn and object templates from raw files
        // let spawns = load_spawns();
        // let object_templates = load_object_templates();

        // // let mut world_generator = RogueWorldGenerator::new();
        // let mut world_generator = OrganicsWorldGenerator::new();

        (state, objects)
    }

    fn world_gen(&mut self) -> RunState {
        let new_runstate = self.world_generator.make_world(
            &mut self.state,
            &mut self.objects,
            &self.spawns,
            &self.object_templates,
        );

        match new_runstate {
            RunState::WorldGen => {}
            _ => {
                // world gen is now done
                // objects.set_tile_dna_random(&mut state.rng, &state.gene_library);
                self.objects.set_tile_dna(
                    &mut self.state.rng,
                    vec![
                        "Cell Membrane".to_string(),
                        "Cell Membrane".to_string(),
                        "Cell Membrane".to_string(),
                        "Energy Store".to_string(),
                        "Energy Store".to_string(),
                        "Receptor".to_string(),
                        "Binary Fission".to_string(),
                        "Kill Switch".to_string(),
                        "Life Expectancy".to_string(),
                        "Life Expectancy".to_string(),
                        "Life Expectancy".to_string(),
                    ],
                    &self.state.gene_library,
                );

                if !innit_env().is_spectating {
                    // create object representing the player
                    let (new_x, new_y) = self.world_generator.get_player_start_pos();
                    let dna = self.state.gene_library.dna_from_distribution(
                        &mut self.state.rng,
                        &[3, 2, 5],
                        &[
                            TraitFamily::Sensing,
                            TraitFamily::Processing,
                            TraitFamily::Actuating,
                        ],
                        false,
                        GENOME_LEN,
                    );
                    let player = Object::new()
                        .position(new_x, new_y)
                        .living(true)
                        .visualize("You", '@', (255, 255, 255, 255))
                        .physical(true, false, true)
                        .control(Controller::Player(PlayerCtrl::new()))
                        .genome(
                            0.99,
                            self.state
                                .gene_library
                                .dna_to_traits(DnaType::Nucleus, &dna),
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

                    self.objects.set_player(player);

                    // a warm welcoming message
                    self.state.log.add(
                        "Welcome microbe! You're innit now. Beware of bacteria and viruses",
                        MsgClass::Story,
                    );
                }
            }
        }

        new_runstate
    }
}

/// Load an existing savegame and instantiates GameState & Objects
/// from which the game is resumed in the game loop.
#[cfg(not(target_arch = "wasm32"))]
fn load_game() -> Result<(GameState, GameObjects), Box<dyn Error>> {
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

/// Dummy game loading function for building innit with WebAssembly.
/// In this case loading is disabled and attempted use will simply redirect to the main menu.
#[cfg(target_arch = "wasm32")]
fn load_game() -> Result<(GameState, GameObjects), Box<dyn Error>> {
    Err("game loading not available in the web version".into())
}

/// Serialize and store GameState and Objects into a JSON file.
#[cfg(not(target_arch = "wasm32"))]
fn save_game(state: &GameState, objects: &GameObjects) -> Result<(), Box<dyn Error>> {
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

/// Dummy file for saving the game state.
/// Attempted use will do nothing when built with WebAssembly.
#[cfg(target_arch = "wasm32")]
fn save_game(_state: &GameState, _objects: &GameObjects) -> Result<(), Box<dyn Error>> {
    Err("game saving not available in the web version".into())
}

impl Rltk_GameState for Game {
    /// Central function of the game.
    /// - process player input
    /// - render game world
    /// - let NPCs take their turn
    fn tick(&mut self, ctx: &mut Rltk) {
        let mut timer = Timer::new("game loop");
        // mouse workaround
        if ctx.left_click {
            if self.mouse_workaround {
                ctx.left_click = false;
            }
            self.mouse_workaround = !self.mouse_workaround;
        }

        // Render world and world only if there is any new information, otherwise save the
        // computation.
        if self.re_render || self.hud.require_refresh || self.state.log.is_changed {
            // println!(
            //     "{}, {}, {}",
            //     self.re_render, self.hud.require_refresh, self.state.log.is_changed
            // );
            ctx.set_active_console(HUD_CON);
            ctx.cls();

            // if self.re_render || self.hud.require_refresh {
            ctx.set_active_console(WORLD_CON);
            ctx.cls();
            render_world(&mut self.objects, ctx);
            // }

            ctx.set_active_console(HUD_CON);
            if let Some(player) = self.objects.extract_by_index(self.state.player_idx) {
                render_gui(&self.state, &mut self.hud, ctx, &player);
                self.objects.replace(self.state.player_idx, player);
            }

            // switch off any triggers
            self.re_render = false;
            self.state.log.is_changed = false;
            self.hud.require_refresh = false
        }

        // The particles need to be queried each cycle to activate and cull them in time.
        trace!("updating particles");
        ctx.set_active_console(PAR_CON);
        ctx.cls();
        let mut draw_batch = DrawBatch::new();
        for particle in &particles().particles {
            if particle.start_delay <= 0.0 {
                draw_batch.set_fancy(
                    particle.pos,
                    1,
                    Degrees::new(0.0),
                    particle.scale.into(),
                    ColorPair::new(particle.col_fg, particle.col_bg),
                    to_cp437(particle.glyph),
                );
            }
        }
        draw_batch.submit(PAR_CON_Z).unwrap();
        self.re_render = particles().update(ctx);

        let mut new_run_state = self.run_state.take().unwrap();
        new_run_state = match new_run_state {
            RunState::MainMenu(ref mut instance) => {
                self.state.log.is_changed = false;
                self.hud.require_refresh = false;
                self.re_render = false;
                particles().particles.clear();
                ctx.set_active_console(WORLD_CON);
                ctx.cls();
                ctx.render_xp_sprite(&self.rex_assets.menu, 0, 0);
                match instance.display(ctx) {
                    Some(option) => {
                        MainMenuItem::process(&mut self.state, &mut self.objects, instance, &option)
                    }
                    None => RunState::MainMenu(instance.clone()),
                }
            }
            RunState::GameOver(ref mut instance) => {
                self.state.log.is_changed = false;
                self.hud.require_refresh = false;
                self.re_render = false;
                particles().particles.clear();
                ctx.set_active_console(WORLD_CON);
                ctx.cls();
                ctx.render_xp_sprite(&self.rex_assets.menu, 0, 0);
                let fg = palette().hud_fg_dna_sensor;
                let bg = palette().hud_bg;
                ctx.print_color_centered_at(SCREEN_WIDTH / 2, 1, fg, bg, "GAME OVER");
                match instance.display(ctx) {
                    Some(option) => GameOverMenuItem::process(
                        &mut self.state,
                        &mut self.objects,
                        instance,
                        &option,
                    ),
                    None => RunState::GameOver(instance.clone()),
                }
            }
            RunState::ChooseActionMenu(ref mut instance) => match instance.display(ctx) {
                Some(option) => {
                    self.re_render = true;
                    ActionItem::process(&mut self.state, &mut self.objects, instance, &option)
                }
                None => RunState::ChooseActionMenu(instance.clone()),
            },
            RunState::Ticking => {
                trace!("enter RunState::Ticking {}", self.state.log.is_changed);
                let mut feedback;
                // Let the game engine process objects until we have to re-render the world or UI.
                // Re-rendering is necessary either because the world changed or messages need to
                // be printed to the log.
                'processing: loop {
                    feedback = self.state.process_object(&mut self.objects);
                    if feedback != ObjectFeedback::NoFeedback || self.state.log.is_changed {
                        break 'processing;
                    }
                }

                trace!("process feedback in RunState::Ticking: {:#?}", feedback);
                match feedback {
                    ObjectFeedback::GameOver => RunState::GameOver(game_over_menu()),
                    ObjectFeedback::Render => {
                        // if innit_env().is_spectating {
                        //     RunState::CheckInput
                        // } else {
                        self.re_render = true;
                        RunState::Ticking
                        // }
                    }
                    ObjectFeedback::GenomeManipulator => {
                        if let Some(genome_editor) =
                            create_genome_manipulator(&mut self.state, &mut self.objects)
                        {
                            RunState::GenomeEditing(genome_editor)
                        } else {
                            RunState::CheckInput
                        }
                    }
                    ObjectFeedback::UpdateHud => {
                        self.hud.require_refresh = true;
                        RunState::Ticking
                    }
                    // if there is no reason to re-render, check whether we're waiting on user input
                    _ => {
                        if self.state.is_players_turn()
                            && (self.state.player_energy_full(&self.objects)
                                || innit_env().is_spectating)
                        {
                            RunState::CheckInput
                        } else {
                            self.re_render = false;
                            RunState::Ticking
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
                            let a: Option<Box<dyn Action>> = match in_game_action {
                                PrimaryAction(dir) => Some(player.get_primary_action(dir)),
                                SecondaryAction(dir) => Some(player.get_secondary_action(dir)),
                                Quick1Action => Some(player.get_quick1_action()),
                                Quick2Action => Some(player.get_quick2_action()),
                                UseInventoryItem(idx) => {
                                    trace!("PlayInput USE_ITEM");
                                    let inventory_object = &player.inventory.items.remove(idx);
                                    player.inventory.inv_actions.retain(|a| {
                                        a.get_identifier() != "drop item"
                                            || a.get_level() == idx as i32
                                    });
                                    if let Some(item) = &inventory_object.item {
                                        item.use_action.clone()
                                    } else {
                                        None
                                    }
                                }
                                DropItem(idx) => {
                                    trace!("PlayInput DROP_ITEM");
                                    if player.inventory.items.len() > idx {
                                        Some(Box::new(ActDropItem::new(idx as i32)))
                                    } else {
                                        None
                                    }
                                }
                                PassTurn => Some(Box::new(ActPass::default())),
                            };
                            player.set_next_action(a);
                            RunState::Ticking
                        } else {
                            RunState::Ticking
                        }
                    }
                    PlayerInput::Undefined => {
                        // if we're only spectating then go back to ticking, otherwise keep
                        // checking for input
                        if innit_env().is_spectating {
                            RunState::Ticking
                        } else {
                            RunState::CheckInput
                        }
                    }
                }
            }
            RunState::GenomeEditing(genome_editor) => match genome_editor.state {
                GenomeEditingState::Done => {
                    if let Some(ref mut player) = self.objects[self.state.player_idx] {
                        player.set_dna(genome_editor.player_dna);
                    }
                    self.re_render = true;
                    RunState::CheckInput
                }

                _ => genome_editor.display(&mut self.state, ctx),
            },
            RunState::InfoBox(infobox) => match infobox.display(ctx) {
                Some(infobox) => RunState::InfoBox(infobox),
                None => {
                    self.re_render = true;
                    RunState::Ticking
                }
            },
            RunState::NewGame => {
                // start new game
                let (new_state, new_objects) = Game::new_game();
                self.reset(new_state, new_objects);
                self.world_generator = OrganicsWorldGenerator::new();
                RunState::WorldGen
            }
            RunState::LoadGame => {
                // load game from file
                match load_game() {
                    Ok((state, objects)) => {
                        self.reset(state, objects);
                        self.re_render = true;
                        RunState::Ticking
                    }
                    Err(_e) => RunState::MainMenu(main_menu()),
                }
            }
            RunState::WorldGen => {
                if innit_env().is_debug_mode {
                    self.re_render = true;
                }
                self.world_gen()
            }
        };
        self.run_state.replace(new_run_state);

        ctx.set_active_console(HUD_CON);
        ctx.print_color(
            1,
            1,
            (255, 255, 255),
            (0, 0, 0),
            &format!("FPS: {}", ctx.fps),
        );

        // keep time and emit warning if a tick takes longer than half a second
        let tick_elapsed = timer.stop_silent();
        if tick_elapsed > 500_000_000 {
            warn!("game loop took {}", time_from(tick_elapsed));
        }
        self.slowest_tick = self.slowest_tick.max(tick_elapsed);

        rltk::render_draw_buffer(ctx).unwrap()
    }
}

pub fn handle_meta_actions(
    state: &mut GameState,
    objects: &mut GameObjects,
    ctx: &mut Rltk,
    action: UiAction,
) -> RunState {
    debug!("received action {:?}", action);
    match action {
        UiAction::ExitGameLoop => {
            match save_game(&state, &objects) {
                Ok(()) => {}
                Err(_e) => {} // TODO: Create some message visible in the main menu
            }
            RunState::MainMenu(main_menu())
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
                    RunState::Ticking
                }
            } else {
                RunState::Ticking
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
                    RunState::Ticking
                }
            } else {
                RunState::Ticking
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
                    RunState::Ticking
                }
            } else {
                RunState::Ticking
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
                    RunState::Ticking
                }
            } else {
                RunState::Ticking
            }
        }
        UiAction::GenomeEditor => {
            if let Some(genome_editor) = create_genome_manipulator(state, objects) {
                RunState::GenomeEditing(genome_editor)
            } else {
                RunState::CheckInput
            }
        }
        UiAction::Help => RunState::InfoBox(controls_screen()),
        UiAction::SetFont(x) => {
            ctx.set_active_console(WORLD_CON);
            ctx.set_active_font(x, false);
            // ctx.cls();
            ctx.set_active_console(HUD_CON);
            ctx.set_active_font(x, false);
            // ctx.cls();
            ctx.set_active_console(PAR_CON);
            ctx.set_active_font(x, false);
            // ctx.cls();
            RunState::CheckInput
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

fn create_genome_manipulator(
    state: &mut GameState,
    objects: &mut GameObjects,
) -> Option<GenomeEditor> {
    if let Some(ref mut player) = objects[state.player_idx] {
        // NOTE: In the future editor features could be read from the plasmid.
        let genome_editor = GenomeEditor::new(player.dna.clone(), 1);
        Some(genome_editor)
    } else {
        None
    }
}
