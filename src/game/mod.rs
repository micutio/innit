//! The top level representation of the game. Here the major game components are constructed and
//! the game loop is executed.

pub mod consts;
pub mod env;
pub mod msg;
pub mod objects;
pub mod position;
mod state;

pub use env::env;
pub use objects::ObjectStore;
pub use position::Position;
pub use state::State;

use crate::entity::act;
use crate::entity::control;
use crate::entity::genetics;
use crate::entity::object;
use crate::game::msg::MessageLog;
use crate::raws;
use crate::ui::custom::genome_editor;
use crate::ui::dialog;
use crate::ui::frontend;
use crate::ui::hud;
use crate::ui::input;
use crate::ui::menu::{self, MenuItem};
use crate::ui::palette;
use crate::ui::particles;
use crate::ui::rex_assets;
use crate::util;
use crate::util::timer;
use crate::world_gen;
use crate::world_gen::WorldGen;

use core::fmt;
use rltk;
use std::error::Error;
use std::fmt::{Display, Formatter};
#[cfg(not(target_arch = "wasm32"))]
use std::fs::{self, File};
#[cfg(not(target_arch = "wasm32"))]
use std::io::{Read, Write};

#[derive(Debug)]
pub enum RunState {
    MainMenu(menu::Menu<menu::main::MainMenuItem>),
    NewGame,
    LoadGame,
    ChooseActionMenu(menu::Menu<menu::choose_action::ActionItem>),
    GameOver(menu::Menu<menu::game_over::GameOverMenuItem>),
    WinScreen(menu::Menu<menu::game_won::GameWonMenuItem>),
    InfoBox(dialog::InfoBox),
    GenomeEditing(genome_editor::GenomeEditor),
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
            RunState::WinScreen(_) => write!(f, "WinScreen"),
            RunState::InfoBox(_) => write!(f, "InfoBox"),
            RunState::GenomeEditing(_) => write!(f, "GenomeEditing"),
            RunState::Ticking => write!(f, "Ticking"),
            RunState::CheckInput => write!(f, "CheckInput"),
            RunState::WorldGen => write!(f, "WorldGen"),
        }
    }
}

pub struct Game {
    tick_timer: util::Timer,
    turn_timer: util::Timer,
    state: State,
    objects: ObjectStore,
    run_state: Option<RunState>,
    // world generation state start
    spawns: Vec<raws::spawn::Spawn>,
    object_templates: Vec<raws::template::ObjectTemplate>,
    world_generator: world_gen::ca::CaBased,
    // world generation state end
    hud: hud::Hud,
    require_render: bool,
    rex_assets: rex_assets::RexAssets,
    /// This workaround is required because each mouse click is registered twice (press & release),
    /// Without it each mouse event is fired twice in a row and toggles are useless.
    mouse_workaround: bool,
    /// Keep track of the time to warn if the game runs too slow.
    slowest_tick: u128,
}

impl Game {
    pub fn new() -> Self {
        let state = State::new(1);
        let objects = ObjectStore::new();
        let run_state = if env().is_spectating {
            RunState::NewGame
        } else {
            RunState::MainMenu(menu::main::new())
        };

        Game {
            tick_timer: util::Timer::new("tick timer"),
            turn_timer: util::Timer::new("turn timer"),
            state,
            objects,
            // spawns: load_spawns(),
            // object_templates: load_object_templates(),
            run_state: Some(run_state),
            spawns: raws::load_spawns(),
            object_templates: raws::load_object_templates(),

            // let mut world_generator : RogueWorldGenerator::new(),
            world_generator: world_gen::ca::CaBased::new(),
            hud: hud::Hud::new(),
            require_render: false,
            rex_assets: rex_assets::RexAssets::new(),
            mouse_workaround: false,
            slowest_tick: 0,
        }
    }

    fn reset(&mut self, state: State, objects: ObjectStore) {
        self.state = state;
        self.objects = objects;

        if let Some(player) = &self.objects[self.state.player_idx] {
            self.hud.update_ui_items(player);
        };
    }

    /// Create a new game by instantiating the game engine, game state and object vector.
    fn new_game() -> (State, ObjectStore) {
        // create game state holding game-relevant information
        let state = State::new(1);

        // initialise game object vector
        let mut objects = ObjectStore::new();
        objects.blank_circle_world();

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
                        "Receptor".to_string(),
                        "Cell Membrane".to_string(),
                        "Cell Membrane".to_string(),
                        "Energy Store".to_string(),
                        "Energy Store".to_string(),
                        "Binary Fission".to_string(),
                        "Kill Switch".to_string(),
                        "Life Expectancy".to_string(),
                        "Life Expectancy".to_string(),
                        "Life Expectancy".to_string(),
                    ],
                    &self.state.gene_library,
                );

                if !env().is_spectating {
                    // create object representing the player
                    let (new_x, new_y) = self.world_generator.get_player_start_pos();
                    // let dna = self.state.gene_library.dna_from_distribution(
                    //     &mut self.state.rng,
                    //     &[3, 2, 5],
                    //     &[
                    //         TraitFamily::Sensing,
                    //         TraitFamily::Processing,
                    //         TraitFamily::Actuating,
                    //     ],
                    //     false,
                    //     GENOME_LEN,
                    // );
                    let dna = self.state.gene_library.dna_from_trait_strs(
                        &mut self.state.rng,
                        &[
                            "Move".to_string(),
                            "Receptor".to_string(),
                            "Optical Sensor".to_string(),
                            "Optical Sensor".to_string(),
                            "Optical Sensor".to_string(),
                            "Energy Store".to_string(),
                            "Energy Store".to_string(),
                            "Energy Store".to_string(),
                        ],
                    );
                    let player = object::Object::new()
                        .position_xy(new_x, new_y)
                        .living(true)
                        .visualize("You", '@', (255, 255, 255, 255))
                        .physical(true, false, true)
                        .control(control::Controller::Player(control::Player::new()))
                        .genome(
                            0.99,
                            self.state
                                .gene_library
                                .dna_to_traits(genetics::DnaType::Nucleus, &dna),
                        );

                    trace!("created player object {}", player);
                    trace!("player sensors: {:?}", player.sensors);
                    trace!("player processors: {:?}", player.processors);
                    trace!("player actuators: {:?}", player.actuators);
                    trace!("player dna: {:?}", player.dna);
                    trace!(
                        "player default action: {:?}",
                        player.get_primary_action(act::Target::Center).to_text()
                    );

                    self.objects.set_player(player);

                    // a warm welcoming message
                    self.state.log.add(
                        "Welcome microbe! You're innit now. Beware of bacteria and viruses",
                        msg::MsgClass::Story,
                    );
                }
            }
        }

        new_runstate
    }

    /// Check whether there is a turn limit and if so, whether it has been reached.
    fn check_turn_limit(&self) {
        if let Some(turn_limit) = env().turn_limit {
            if self.state.turn > turn_limit {
                warn!(
                    "turn limit of {} has been reached, closing game",
                    turn_limit
                );
                std::process::exit(0);
            }
        }
    }

    fn check_win_condition(&self) -> bool {
        let has_game_started = self.state.turn > 0;
        let has_no_pathogens_left = !self
            .objects
            .get_non_tiles()
            .iter()
            .flatten()
            .any(|o| o.visual.name.to_lowercase().contains("virus"));
        has_game_started && has_no_pathogens_left
    }

    /// Use the `input::PlayerAction` enum to determine where to retrieve the player's next action.
    fn set_player_action(&mut self, in_game_action: input::PlayerAction) -> RunState {
        if let Some(ref mut player) = self.objects[self.state.player_idx] {
            use crate::ui::input::PlayerAction::*;
            let a: Option<Box<dyn act::Action>> = match in_game_action {
                Primary(dir) => Some(player.get_primary_action(dir)),
                Secondary(dir) => Some(player.get_secondary_action(dir)),
                Quick1 => Some(player.get_quick1_action()),
                Quick2 => Some(player.get_quick2_action()),
                UseInventoryItem(idx) => {
                    trace!("PlayInput USE_ITEM");
                    let inventory_object = &player.inventory.items.remove(idx);
                    player.inventory.inv_actions.retain(|a| {
                        a.get_identifier() != "drop item" || a.get_level() == idx as i32
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
                        Some(Box::new(act::DropItem::new(idx as i32)))
                    } else {
                        None
                    }
                }
                PassTurn => Some(Box::new(act::Pass)),
            };
            player.set_next_action(a);
            RunState::Ticking
        } else {
            RunState::Ticking
        }
    }

    /// Perform a meta action, i.e.: the player using the game UI without directly affecting the
    /// game state.
    fn run_meta_action(&mut self, ctx: &mut rltk::Rltk, action: input::UiAction) -> RunState {
        debug!("received action {:?}", action);
        use input::UiAction;
        match action {
            UiAction::ExitGameLoop => {
                match save_game(&self.state, &self.objects) {
                    Ok(()) => {}
                    Err(_e) => {} // TODO: Create some message visible in the main menu
                }
                RunState::MainMenu(menu::main::new())
            }
            UiAction::CharacterScreen => RunState::InfoBox(dialog::character::character_screen(
                &self.state,
                &self.objects,
            )),
            UiAction::ChoosePrimary => {
                if let Some(ref mut player) = self.objects[self.state.player_idx] {
                    let action_items = player.get_available_actions(&[
                        act::TargetCategory::Any,
                        act::TargetCategory::EmptyObject,
                        act::TargetCategory::BlockingObject,
                    ]);
                    if !action_items.is_empty() {
                        RunState::ChooseActionMenu(menu::choose_action::new(
                            action_items,
                            menu::choose_action::ActionCategory::Primary,
                        ))
                    } else {
                        self.state.log.add(
                            "You have no actions available! Try modifying your genome.",
                            msg::MsgClass::Alert,
                        );
                        RunState::Ticking
                    }
                } else {
                    RunState::Ticking
                }
            }
            UiAction::ChooseSecondary => {
                if let Some(ref mut player) = self.objects[self.state.player_idx] {
                    let action_items = player.get_available_actions(&[
                        act::TargetCategory::Any,
                        act::TargetCategory::EmptyObject,
                        act::TargetCategory::BlockingObject,
                    ]);
                    if !action_items.is_empty() {
                        RunState::ChooseActionMenu(menu::choose_action::new(
                            action_items,
                            menu::choose_action::ActionCategory::Secondary,
                        ))
                    } else {
                        self.state.log.add(
                            "You have no actions available! Try modifying your genome.",
                            msg::MsgClass::Alert,
                        );
                        RunState::Ticking
                    }
                } else {
                    RunState::Ticking
                }
            }
            UiAction::ChooseQuick1 => {
                if let Some(ref mut player) = self.objects[self.state.player_idx] {
                    let action_items = player.get_available_actions(&[act::TargetCategory::None]);
                    if !action_items.is_empty() {
                        RunState::ChooseActionMenu(menu::choose_action::new(
                            action_items,
                            menu::choose_action::ActionCategory::Quick1,
                        ))
                    } else {
                        self.state.log.add(
                            "You have no actions available! Try modifying your genome.",
                            msg::MsgClass::Alert,
                        );
                        RunState::Ticking
                    }
                } else {
                    RunState::Ticking
                }
            }
            UiAction::ChooseQuick2 => {
                if let Some(ref mut player) = self.objects[self.state.player_idx] {
                    let action_items = player.get_available_actions(&[act::TargetCategory::None]);
                    if !action_items.is_empty() {
                        RunState::ChooseActionMenu(menu::choose_action::new(
                            action_items,
                            menu::choose_action::ActionCategory::Quick2,
                        ))
                    } else {
                        self.state.log.add(
                            "You have no actions available! Try modifying your genome.",
                            msg::MsgClass::Alert,
                        );
                        RunState::Ticking
                    }
                } else {
                    RunState::Ticking
                }
            }
            UiAction::GenomeEditor => {
                if let Some(genome_editor) =
                    genome_editor::try_create(&mut self.state, &mut self.objects)
                {
                    RunState::GenomeEditing(genome_editor)
                } else {
                    RunState::CheckInput
                }
            }
            UiAction::Help => RunState::InfoBox(dialog::controls::controls_screen()),
            UiAction::SetFont(x) => {
                ctx.set_active_console(consts::WORLD_CON);
                ctx.set_active_font(x, false);
                // ctx.cls();
                ctx.set_active_console(consts::HUD_CON);
                ctx.set_active_font(x, false);
                // ctx.cls();
                ctx.set_active_console(consts::PAR_CON);
                ctx.set_active_font(x, false);
                // ctx.cls();
                RunState::CheckInput
            }
        }
    }
}

/// Load an existing savegame and instantiates GameState & Objects
/// from which the game is resumed in the game loop.
#[cfg(not(target_arch = "wasm32"))]
fn load_game() -> Result<(State, ObjectStore), Box<dyn Error>> {
    // TODO: Add proper UI error output if any of this fails!
    if let Some(mut save_file) = dirs::data_local_dir() {
        save_file.push("innit");
        save_file.push("savegame");
        let mut file = File::open(save_file)?;
        let mut json_save_state = String::new();
        file.read_to_string(&mut json_save_state)?;
        let result = serde_json::from_str::<(State, ObjectStore)>(&json_save_state)?;
        Ok(result)
    } else {
        error!("CANNOT ACCESS SYSTEM DATA DIR");
        panic!("CANNOT ACCESS SYSTEM DATA DIR");
    }
}

/// Dummy game loading function for building innit with WebAssembly.
/// In this case loading is disabled and attempted use will simply redirect to the main menu.
#[cfg(target_arch = "wasm32")]
fn load_game() -> Result<(State, ObjectStore), Box<dyn Error>> {
    Err("game loading not available in the web version".into())
}

/// Serialize and store GameState and Objects into a JSON file.
#[cfg(not(target_arch = "wasm32"))]
fn save_game(state: &State, objects: &ObjectStore) -> Result<(), Box<dyn Error>> {
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
fn save_game(_state: &State, _objects: &ObjectStore) -> Result<(), Box<dyn Error>> {
    Err("game saving not available in the web version".into())
}

impl rltk::GameState for Game {
    /// Central function of the game.
    /// - process player input
    /// - render game world
    /// - let NPCs take their turn
    fn tick(&mut self, ctx: &mut rltk::Rltk) {
        // mouse workaround
        if ctx.left_click {
            if self.mouse_workaround {
                ctx.left_click = false;
            }
            self.mouse_workaround = !self.mouse_workaround;
        }

        // Render world and world only if there is any new information, otherwise save the
        // computation.
        if self.require_render || self.hud.require_refresh || self.state.log.is_changed {
            trace!(
                "render trigger: world {}, hud {}, log {}",
                self.require_render,
                self.hud.require_refresh,
                self.state.log.is_changed
            );

            let require_update_visibility =
                !(self.hud.require_refresh || self.state.log.is_changed);
            self.state.log.is_changed = false;
            self.hud.require_refresh = false;

            ctx.set_active_console(consts::WORLD_CON);
            ctx.cls();
            ctx.set_active_console(consts::HUD_CON);
            ctx.cls();
            ctx.set_active_console(consts::PAR_CON);
            ctx.cls();

            frontend::render_world(&mut self.objects, ctx, require_update_visibility);

            if let Some(player) = self.objects.extract_by_index(self.state.player_idx) {
                hud::render_gui(&self.state, &mut self.hud, ctx, &player);
                self.objects.replace(self.state.player_idx, player);
            }
        }

        // The particles need to be queried each cycle to activate and cull them in time.
        particles().render(ctx);
        self.require_render = particles().update(ctx);

        let mut new_run_state = self.run_state.take().unwrap();
        let current_turn = self.state.turn;
        new_run_state = match new_run_state {
            RunState::MainMenu(ref mut instance) => {
                self.state.log.is_changed = false;
                self.hud.require_refresh = false;
                self.require_render = false;
                particles().particles.clear();
                ctx.set_active_console(consts::WORLD_CON);
                ctx.cls();
                ctx.render_xp_sprite(&self.rex_assets.menu, 0, 0);
                match instance.display(ctx) {
                    Some(option) => menu::main::MainMenuItem::process(
                        &mut self.state,
                        &mut self.objects,
                        instance,
                        &option,
                    ),
                    None => RunState::MainMenu(instance.clone()),
                }
            }
            RunState::GameOver(ref mut instance) => {
                self.state.log.is_changed = false;
                self.hud.require_refresh = false;
                self.require_render = false;
                particles().particles.clear();
                frontend::render_world(&mut self.objects, ctx, true);
                match instance.display(ctx) {
                    Some(option) => menu::game_over::GameOverMenuItem::process(
                        &mut self.state,
                        &mut self.objects,
                        instance,
                        &option,
                    ),
                    None => RunState::GameOver(instance.clone()),
                }
            }
            RunState::WinScreen(ref mut instance) => {
                self.state.log.is_changed = false;
                self.hud.require_refresh = false;
                self.require_render = false;
                particles().particles.clear();
                ctx.set_active_console(consts::WORLD_CON);
                frontend::render_world(&mut self.objects, ctx, true);
                match instance.display(ctx) {
                    Some(option) => menu::game_won::GameWonMenuItem::process(
                        &mut self.state,
                        &mut self.objects,
                        instance,
                        &option,
                    ),
                    None => RunState::WinScreen(instance.clone()),
                }
            }
            RunState::ChooseActionMenu(ref mut instance) => match instance.display(ctx) {
                Some(option) => {
                    self.require_render = true;
                    menu::choose_action::ActionItem::process(
                        &mut self.state,
                        &mut self.objects,
                        instance,
                        &option,
                    )
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
                    if feedback != act::ObjectFeedback::NoFeedback || self.state.log.is_changed {
                        break 'processing;
                    }
                }

                trace!("process feedback in RunState::Ticking: {:#?}", feedback);
                match feedback {
                    act::ObjectFeedback::GameOver => RunState::GameOver(menu::game_over::new()),
                    act::ObjectFeedback::Render => {
                        self.require_render = true;
                        RunState::Ticking
                    }
                    act::ObjectFeedback::GenomeManipulator => {
                        if let Some(genome_editor) =
                            genome_editor::try_create(&mut self.state, &mut self.objects)
                        {
                            RunState::GenomeEditing(genome_editor)
                        } else {
                            RunState::CheckInput
                        }
                    }
                    act::ObjectFeedback::UpdateHud => {
                        self.hud.require_refresh = true;
                        RunState::Ticking
                    }
                    // if there is no reason to re-render, check whether we're waiting on user input
                    _ => {
                        if self.state.is_players_turn()
                            && (self.state.player_energy_full(&self.objects) || env().is_spectating)
                        {
                            RunState::CheckInput
                        } else {
                            self.require_render = false;
                            RunState::Ticking
                        }
                    }
                }
            }
            RunState::CheckInput => {
                use input::PlayerInput;
                match input::read(&mut self.state, &mut self.objects, &mut self.hud, ctx) {
                    PlayerInput::Meta(meta_action) => {
                        trace!("process meta action: {:#?}", meta_action);
                        self.run_meta_action(ctx, meta_action)
                    }
                    PlayerInput::Game(game_action) => {
                        trace!("inject in-game action {:#?} to player", game_action);
                        self.set_player_action(game_action)
                    }
                    PlayerInput::Undefined => {
                        // if we're only spectating then go back to ticking, otherwise keep
                        // checking for input
                        if env().is_spectating {
                            RunState::Ticking
                        } else {
                            RunState::CheckInput
                        }
                    }
                }
            }
            RunState::GenomeEditing(genome_editor) => match genome_editor.state {
                genome_editor::GenomeEditingState::Done => {
                    if let Some(ref mut player) = self.objects[self.state.player_idx] {
                        player.set_dna(genome_editor.player_dna);
                    }
                    self.require_render = true;
                    RunState::CheckInput
                }

                _ => genome_editor.display(&mut self.state, ctx),
            },
            RunState::InfoBox(infobox) => match infobox.display(ctx) {
                Some(infobox) => RunState::InfoBox(infobox),
                None => {
                    self.require_render = true;
                    RunState::Ticking
                }
            },
            RunState::NewGame => {
                // start new game
                ctx.set_active_console(consts::WORLD_CON);
                ctx.cls_bg(rltk::RGBA::new());
                let (new_state, new_objects) = Game::new_game();
                self.reset(new_state, new_objects);
                self.world_generator = world_gen::ca::CaBased::new();
                self.tick_timer = util::Timer::new("tick timer");
                self.turn_timer = util::Timer::new("turn timer");
                RunState::WorldGen
            }
            RunState::LoadGame => {
                // load game from file
                match load_game() {
                    Ok((state, objects)) => {
                        self.reset(state, objects);
                        self.require_render = true;
                        RunState::Ticking
                    }
                    Err(_e) => RunState::MainMenu(menu::main::new()),
                }
            }
            RunState::WorldGen => {
                if env().is_debug_mode {
                    self.require_render = true;
                }
                self.world_gen()
            }
        };
        self.run_state.replace(new_run_state);

        ctx.set_active_console(consts::HUD_CON);
        let fps_fg = palette().hud_fg;
        let fps_bg = palette().hud_bg;
        ctx.print_color(
            0,
            consts::SCREEN_HEIGHT - 1,
            fps_fg,
            fps_bg,
            &format!("FPS: {}", ctx.fps),
        );

        self.check_turn_limit();

        // keep time and emit warning if a tick takes longer than half a second
        let tick_elapsed = self.tick_timer.lap_silent();
        if tick_elapsed > 50_000_000 {
            warn!("game loop running slow: {}", timer::format(tick_elapsed));
        }
        self.slowest_tick = self.slowest_tick.max(tick_elapsed);

        if current_turn != self.state.turn {
            self.turn_timer.lap();
            info!("new turn {}", self.state.turn);

            // Check whether the win condition is fulfilled and if so, override the run state to
            // skip to the win screen.
            let has_won = self.check_win_condition();
            if has_won {
                self.run_state
                    .replace(RunState::WinScreen(menu::game_won::new()));
            }
        }

        rltk::render_draw_buffer(ctx).unwrap()
    }
}
