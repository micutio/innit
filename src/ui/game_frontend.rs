use tcod::chars;
/// Module GUI
///
/// This module contains all structures and methods pertaining to the user interface.
///
/// Game frontend, input, state, engine etc are very common parameters
/// for function calls in innit, therefore utmost consistency should be
/// kept in the order of handing them over to functions.
///
/// Function parameter precedence:
/// game_state, game_frontend, game_input, objects, anything else.
use tcod::colors::{self, Color};
use tcod::console::*;
use tcod::map::FovAlgorithm;

use core::game_objects::GameObjects;
use core::game_state::{GameState, ObjectProcResult, PLAYER, TORCH_RADIUS};
use core::world::world_gen::is_explored;
use entity::object::Object;
use game::{game_loop, load_game, new_game, save_game, WORLD_HEIGHT, WORLD_WIDTH};
use ui::color_palette::*;
use ui::dialog::*;
use ui::game_input::{GameInput, UiAction};

// game window properties
pub const SCREEN_WIDTH: i32 = 80;
pub const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 20; // target fps

// field of view algorithm parameters
const FOV_ALG: FovAlgorithm = FovAlgorithm::Shadow;
const FOV_LIGHT_WALLS: bool = true;

// ui and menu constraints
pub const BAR_WIDTH: i32 = 20;
pub const PANEL_HEIGHT: i32 = 7;
const PANEL_Y: i32 = SCREEN_HEIGHT - PANEL_HEIGHT;

/// Field of view mapping
pub use tcod::map::Map as FovMap;

/// GameIO holds the core components for game's input and output processing.
pub struct GameFrontend {
    pub root: Root,
    pub con: Offscreen,
    pub panel: Offscreen,
    pub fov: FovMap,
    pub coloring: ColorPalette,
    pub input: Option<GameInput>,
}

impl GameFrontend {
    /// Initialize the game frontend:
    ///     - load assets, like fonts etc.
    ///     - set ui window size
    ///     - set ui window title
    ///     - set fps
    ///     - init permanent ui components
    pub fn new() -> Self {
        let root = Root::initializer()
            .font("assets/terminal16x16_gs_ro.png", FontLayout::AsciiInRow)
            .font_type(FontType::Greyscale)
            .size(SCREEN_WIDTH, SCREEN_HEIGHT)
            .title("innit alpha v0.0.1")
            .init();

        tcod::system::set_fps(LIMIT_FPS);

        GameFrontend {
            root,
            con: Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT),
            panel: Offscreen::new(SCREEN_WIDTH, PANEL_HEIGHT),
            fov: FovMap::new(WORLD_WIDTH, WORLD_HEIGHT),
            coloring: ColorPalette::new(),
            input: None,
        }
    }
}

/// Specification of animations and their parameters.
/// TODO: Outsource (heh) this to its own module.
#[derive(PartialEq, Debug)]
pub enum AnimationType {
    /// Gradual transition of the world hue and or brightness
    ColorTransition,
    /// A cell flashes with a specific character.
    /// Example: flash a red 'x' over an object to indicate a hit.
    FlashEffect,
}

/// Main menu of the game.
/// Display a background image and three options for the player to choose
///     - starting a new game
///     - loading an existing game
///     - quitting the game
pub fn main_menu(game_frontend: &mut GameFrontend) {
    let img = tcod::image::Image::from_file("assets/menu_background_pixelized_title.png")
        .expect("Background image not found");

    while !game_frontend.root.window_closed() {
        // show the background image, at twice the regular console resolution
        tcod::image::blit_2x(&img, (0, 0), (-1, -1), &mut game_frontend.root, (0, 0));

        game_frontend
            .root
            .set_default_foreground(game_frontend.coloring.get_col_menu_bg());
        game_frontend
            .root
            .set_default_background(game_frontend.coloring.get_col_menu_bg());
        game_frontend
            .root
            .set_default_foreground(game_frontend.coloring.get_col_acc_warm());
        game_frontend.root.print_ex(
            SCREEN_WIDTH / 2,
            SCREEN_HEIGHT - 2,
            BackgroundFlag::None,
            TextAlignment::Center,
            "By Michael Wagner",
        );

        // show options and wait for the player's choice
        let choices = &["Play a new game", "Continue last game", "Quit"];
        let choice = menu(game_frontend, &mut None, "main menu", choices, 24);

        match choice {
            Some(0) => {
                // start new game
                let (mut game_state, mut game_objects) = new_game();
                // initialize_fov(game_frontend, &mut objects);
                let mut game_input = GameInput::new();
                init_object_visuals(
                    &mut game_state,
                    game_frontend,
                    &game_input,
                    &mut game_objects,
                );
                game_input.start_concurrent_input();
                game_loop(
                    &mut game_state,
                    game_frontend,
                    &mut game_input,
                    &mut game_objects,
                );
            }
            Some(1) => {
                // load game from file
                match load_game() {
                    Ok((mut game_state, mut game_objects)) => {
                        // initialize_fov(game_frontend, &mut objects);
                        let mut game_input = GameInput::new();
                        init_object_visuals(
                            &mut game_state,
                            game_frontend,
                            &game_input,
                            &mut game_objects,
                        );
                        game_input.start_concurrent_input();
                        game_loop(
                            &mut game_state,
                            game_frontend,
                            &mut game_input,
                            &mut game_objects,
                        );
                    }
                    Err(_e) => {
                        msgbox(
                            game_frontend,
                            &mut None,
                            "",
                            "\nNo saved game to load\n",
                            24,
                        );
                        continue;
                    }
                }
            }
            Some(2) => {
                // quit
                break;
            }
            _ => {}
        }

        // let mut x: u8 = 0;
        // for i in 0..16 {
        //     for j in 0..16 {
        //         game_frontend
        //             .root
        //             .put_char_ex(i, j, x as u8 as char, colors::WHITE, colors::BLUE);
        //         if x < 255 {
        //             x += 1;
        //         }
        //     }
        // }
    }
}

/// Initialize the field of view map with a given instance of **World**
fn initialize_fov(game_frontend: &mut GameFrontend, objects: &mut GameObjects) {
    // init fov map
    for y in 0..WORLD_HEIGHT {
        for x in 0..WORLD_WIDTH {
            match objects.get_tile_at(x as usize, y as usize) {
                Some(object) => {
                    game_frontend.fov.set(
                        x as i32,
                        y as i32,
                        !object.physics.is_blocking_sight,
                        !object.physics.is_blocking,
                    );
                }
                None => {
                    panic!("[game_frontend] Error initializing fov");
                }
            }
        }
    }
    // unexplored areas start black (which is the default background color)
    game_frontend.con.clear();
    game_frontend
        .con
        .set_default_background(game_frontend.coloring.get_col_world_bg());
}

pub fn recompute_fov(game_frontend: &mut GameFrontend, objects: &GameObjects) {
    if let Some(ref player) = objects[PLAYER] {
        game_frontend
            .fov
            .compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALG);
    }
}

/// Initialize the player's field of view and render objects + ui for the start of the game.
fn init_object_visuals(
    game_state: &mut GameState,
    game_frontend: &mut GameFrontend,
    game_input: &GameInput,
    game_objects: &mut GameObjects,
) {
    initialize_fov(game_frontend, game_objects);
    recompute_fov(game_frontend, game_objects);
    re_render(
        game_state,
        game_frontend,
        game_objects,
        &game_input.names_under_mouse,
    );
}

/// Update the player's field of view and updated which tiles are visible/explored.
fn update_visibility(game_frontend: &mut GameFrontend, objects: &mut GameObjects) {
    // go through all tiles and set their background color
    let mut player_pos: (i32, i32) = (0, 0);
    if let Some(ref player) = objects[PLAYER] {
        player_pos = (player.x, player.y);
    }

    let col_wall_out_fov = game_frontend.coloring.get_col_wall_out_fov();
    let col_wall_in_fov = game_frontend.coloring.get_col_wall_in_fov();
    let col_ground_out_fov = game_frontend.coloring.get_col_ground_out_fov();
    let col_ground_in_fov = game_frontend.coloring.get_col_ground_in_fov();

    for y in 0..WORLD_HEIGHT {
        for x in 0..WORLD_WIDTH {
            let visible = game_frontend.fov.is_in_fov(x, y);
            if let Some(ref mut tile_object) = objects.get_tile_at(x as usize, y as usize) {
                let wall = tile_object.physics.is_blocking_sight;

                // set tile background colors
                let tile_color = match (visible, wall) {
                    // outside field of view:
                    (false, true) => col_wall_out_fov,
                    (false, false) => col_ground_out_fov,
                    // inside fov:
                    // (true, true) => COLOR_LIGHT_WALL,
                    (true, true) => colors::lerp(
                        col_wall_in_fov,
                        col_wall_out_fov,
                        tile_object.distance(player_pos.0, player_pos.1) / TORCH_RADIUS as f32,
                    ),
                    // (true, false) => COLOR_ground_in_fov,
                    (true, false) => colors::lerp(
                        col_ground_in_fov,
                        col_ground_out_fov,
                        tile_object.distance(player_pos.0, player_pos.1) / TORCH_RADIUS as f32,
                    ),
                };

                if let Some(tile) = &mut tile_object.tile {
                    if visible {
                        tile.explored = true;
                    }
                    if tile.explored {
                        // show explored tiles only (any visible tile is explored already)
                        tile_object.visual.color = tile_color;
                        // game_frontend.con.set_char_background(
                        //     x,
                        //     y,
                        //     tile_color,
                        //     BackgroundFlag::Set,
                        // );
                    }
                }
            }
        }
    }
}

pub fn process_visual_feedback(
    game_state: &mut GameState,
    game_frontend: &mut GameFrontend,
    game_input: &GameInput,
    game_objects: &mut GameObjects,
    proc_result: ObjectProcResult,
) {
    match proc_result {
        // no action has been performed, repeat the turn and try again
        ObjectProcResult::NoAction => {}

        // action has been completed, but nothing needs to be done about it
        ObjectProcResult::NoFeedback => {}

        // the player's FOV has been updated, thus we also need to re-render
        ObjectProcResult::UpdateFOV => {
            recompute_fov(game_frontend, game_objects);
            re_render(
                game_state,
                game_frontend,
                game_objects,
                &game_input.names_under_mouse,
            );
        }

        // the player hasn't moved but something happened within fov
        ObjectProcResult::ReRender => {
            re_render(
                game_state,
                game_frontend,
                game_objects,
                &game_input.names_under_mouse,
            );
        }

        ObjectProcResult::Animate { anim_type } => {
            // TODO: Play animation.
            info!("animation");
        }

        _ => {}
    }
}

/// Render all objects, the menu
pub fn re_render(
    game_state: &mut GameState,
    game_frontend: &mut GameFrontend,
    objects: &mut GameObjects,
    names_under_mouse: &str,
) {
    // clear the screen of the previous frame
    game_frontend.con.clear();
    // render objects and map
    // step 1/2: update visibility of objects and world tiles
    update_visibility(game_frontend, objects);
    // step 2/2: render everything
    render_all(game_frontend, game_state, objects, names_under_mouse);

    // draw everything on the window at once
    game_frontend.root.flush();
}

/// Render all objects and tiles.
/// Right now this happens because we are updating explored tiles here too.
/// Is there a way to auto-update explored and visible tiles/objects when the player moves?
/// But visibility can also be influenced by other things.
fn render_all(
    game_frontend: &mut GameFrontend,
    game_state: &mut GameState,
    objects: &GameObjects,
    names_under_mouse: &str,
) {
    let mut to_draw: Vec<&Object> = objects
        .get_vector()
        .iter()
        .flatten()
        .filter(|o| {
            // FIXME: there must be a better way than using `and_then`.
            game_frontend.fov.is_in_fov(o.x, o.y)
                || o.physics.is_always_visible
                || (o.tile.is_some() && *o.tile.as_ref().and_then(is_explored).unwrap())
        })
        .collect();
    // sort, so that non-blocking objects come first
    to_draw.sort_by(|o1, o2| o1.physics.is_blocking.cmp(&o2.physics.is_blocking));
    // draw the objects in the list
    for object in &to_draw {
        object.draw(&mut game_frontend.con);
    }

    render_ui(game_frontend, game_state, objects, names_under_mouse);
    // blit contents of `game_frontend.panel` to the root console
    blit(
        &game_frontend.panel,
        (0, 0),
        (SCREEN_WIDTH, SCREEN_HEIGHT),
        &mut game_frontend.root,
        (0, PANEL_Y),
        1.0,
        1.0,
    );

    // blit contents of offscreen console to root console and present it
    blit(
        &game_frontend.con,
        (0, 0),
        (WORLD_WIDTH, WORLD_HEIGHT),
        &mut game_frontend.root,
        (0, 0),
        1.0,
        1.0,
    );
}

/// Render the user interface, consisting of:
///     - health bar
///     - player stats
///     - message log
///     - objects names under mouse cursor
/// Add all ui elements to the panel component of the frontend.
fn render_ui(
    game_frontend: &mut GameFrontend,
    game_state: &mut GameState,
    objects: &GameObjects,
    names_under_mouse: &str,
) {
    // prepare to render the GUI panel
    game_frontend
        .panel
        .set_default_background(game_frontend.coloring.get_col_menu_bg());
    game_frontend.panel.clear();

    // set panel borders
    // set background and foreground colors
    for x in 0..SCREEN_WIDTH {
        for y in 0..PANEL_HEIGHT {
            game_frontend.panel.set_char_background(
                x,
                y,
                game_frontend.coloring.get_col_menu_bg(),
                BackgroundFlag::Set,
            );
            game_frontend
                .panel
                .set_char_foreground(x, y, game_frontend.coloring.get_col_menu_fg());
            game_frontend.panel.set_char(x, y, ' ');
        }
    }

    // render horizontal borders
    for x in 0..SCREEN_WIDTH - 1 {
        game_frontend.panel.set_char(x, 0, chars::DHLINE);
        game_frontend
            .panel
            .set_char(x, PANEL_HEIGHT - 1, chars::HLINE);
    }
    // render vertical borders
    for y in 0..PANEL_HEIGHT - 1 {
        game_frontend.panel.set_char(0, y, chars::VLINE);
        game_frontend
            .panel
            .set_char(SCREEN_WIDTH - 1, y, chars::VLINE);
    }

    // render corners
    game_frontend.panel.set_char(0, 0, '\u{d5}');
    game_frontend.panel.set_char(SCREEN_WIDTH - 1, 0, '\u{b8}');
    game_frontend.panel.set_char(0, PANEL_HEIGHT - 1, chars::SW);
    game_frontend
        .panel
        .set_char(SCREEN_WIDTH - 1, PANEL_HEIGHT - 1, chars::SE);

    // show player's stats
    if let Some(ref player) = objects[PLAYER] {
        let hp = player.fighter.map_or(0, |f| f.hp);
        let max_hp = player.fighter.map_or(0, |f| f.base_max_hp);
        render_bar(
            &mut game_frontend.panel,
            1,
            1,
            BAR_WIDTH,
            "HP",
            hp,
            max_hp,
            colors::DARK_RED,
            colors::DARKEST_RED,
        );
        game_frontend.panel.print_ex(
            1,
            2,
            BackgroundFlag::None,
            TextAlignment::Left,
            format!("Dungeon level: {}", game_state.dungeon_level),
        );

        // show names of objects under the mouse
        game_frontend
            .panel
            .set_default_foreground(colors::LIGHT_GREY);
        game_frontend.panel.print_ex(
            1,
            0,
            BackgroundFlag::None,
            TextAlignment::Left,
            names_under_mouse,
        );

        // print game messages, one line at a time
        let mut y = MSG_HEIGHT as i32;
        for &(ref msg, color) in &mut game_state.log.iter().rev() {
            let msg_height = game_frontend
                .panel
                .get_height_rect(MSG_X, y, MSG_WIDTH, 0, msg);
            y -= msg_height;
            if y < 1 {
                break;
            }
            game_frontend.panel.set_default_foreground(color);
            game_frontend.panel.print_rect(MSG_X, y, MSG_WIDTH, 0, msg);
        }
    }
}

/// Render a generic progress or status bar in the UI.
#[allow(clippy::too_many_arguments)]
fn render_bar(
    panel: &mut Offscreen,
    x: i32,
    y: i32,
    total_width: i32,
    name: &str,
    value: i32,
    maximum: i32,
    bar_color: Color,
    back_color: Color,
) {
    // render a bar (HP, EXP, etc)
    let bar_width = (value as f32 / maximum as f32 * total_width as f32) as i32;

    // render background first
    panel.set_default_background(back_color);
    panel.rect(x, y, total_width, 1, false, BackgroundFlag::Screen);

    // now render bar on top
    panel.set_default_background(bar_color);
    if bar_width > 0 {
        panel.rect(x, y, bar_width, 1, false, BackgroundFlag::Screen);
    }

    // finally some centered text with the values
    panel.set_default_foreground(colors::WHITE);
    panel.print_ex(
        x + total_width / 2,
        y,
        BackgroundFlag::None,
        TextAlignment::Center,
        &format!("{}: {}/{}", name, value, maximum),
    );
}

pub fn handle_ui_actions(
    game_frontend: &mut GameFrontend,
    game_state: &mut GameState,
    game_objects: &mut GameObjects,
    game_input: &mut Option<&mut GameInput>,
    action: UiAction,
) -> bool {
    // TODO: Screens for key mapping, primary and secondary action selection, dna operations.
    match action {
        UiAction::ExitGameLoop => {
            save_game(game_state, game_objects).unwrap();
            return true;
        }
        UiAction::ToggleDarkLightMode => {
            game_frontend.coloring.toggle_dark_light_mode();
            recompute_fov(game_frontend, game_objects);
            initialize_fov(game_frontend, game_objects);
            re_render(game_state, game_frontend, game_objects, "");
        }
        UiAction::CharacterScreen => {
            // TODO: move this to separate function
            // show character information
            show_character_screen(game_state, game_frontend, game_input, game_objects);
        }

        UiAction::Fullscreen => {
            let fullscreen = game_frontend.root.is_fullscreen();
            game_frontend.root.set_fullscreen(!fullscreen);
            initialize_fov(game_frontend, game_objects);
        }
    }
    re_render(game_state, game_frontend, game_objects, "");
    false
}
