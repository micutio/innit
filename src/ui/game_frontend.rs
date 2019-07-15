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

use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use tcod::chars;
use tcod::colors::{self, Color};
use tcod::console::*;
use tcod::map::FovAlgorithm;

use core::game_objects::GameObjects;
use core::game_state::{ GameState, ObjectProcResult, LEVEL_UP_BASE, LEVEL_UP_FACTOR, PLAYER, TORCH_RADIUS, };
use core::world::is_explored;
use entity::object::Object;
use game::{WORLD_HEIGHT, WORLD_WIDTH, new_game, game_loop, load_game, save_game};
use ui::color_palette::*;
use ui::game_input::{
    get_names_under_mouse, get_player_action_instance, start_input_proc_thread, GameInput,
    PlayerAction, UiAction,
};

// game window properties
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 20; // target fps

// field of view algorithm parameters
const FOV_ALG: FovAlgorithm = FovAlgorithm::Shadow;
const FOV_LIGHT_WALLS: bool = true;

// ui and menu constraints
const BAR_WIDTH: i32 = 20;
const PANEL_HEIGHT: i32 = 7;
const PANEL_Y: i32 = SCREEN_HEIGHT - PANEL_HEIGHT;
// message box measurements
const MSG_X: i32 = BAR_WIDTH + 2;
const MSG_WIDTH: i32 = SCREEN_WIDTH - BAR_WIDTH - 2;
const MSG_HEIGHT: usize = PANEL_HEIGHT as usize - 1;
// width of the character info screen.
pub const CHARACTER_SCREEN_WIDTH: i32 = 30;

/// Field of view mapping
pub use tcod::map::Map as FovMap;

/// GameIO holds the core components for game's input and output processing.
pub struct GameFrontend {
    pub root: Root,
    pub con: Offscreen,
    pub panel: Offscreen,
    pub fov: FovMap,
    pub coloring: ColorPalette,
    pub input: Option<InputHandler>,
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

pub struct MousePosition {
    x: i32,
    y: i32,
}

pub struct ConcurrentInput {
    // game_input: Arc<Mutex<GameInput>>,
    game_input_ref: Arc<Mutex<GameInput>>,
    input_thread_tx: Sender<bool>,
    input_thread: JoinHandle<()>,
}

impl ConcurrentInput {
    fn join_thread(self) {
        match self.input_thread.join() {
            Ok(_) => println!("[concurrent input] successfully joined user input thread"),
            Err(e) => println!(
                "[concurrent input] error while trying to join user input thread: {:?}",
                e
            ),
        }
    }
}

pub struct InputHandler {
    pub current_mouse_pos: MousePosition,
    pub names_under_mouse: String,
    concurrent_input: Option<ConcurrentInput>,
    next_action: Option<PlayerAction>,
}

impl InputHandler {
    fn new() -> Self {
        InputHandler {
            current_mouse_pos: MousePosition { x: 0, y: 0 },
            names_under_mouse: Default::default(),
            concurrent_input: None,
            next_action: None,
        }
    }

    pub fn start_concurrent_input(&mut self) {
        let (tx, rx) = mpsc::channel();
        let game_input = Arc::new(Mutex::new(GameInput::new()));
        let mut game_input_ref = Arc::clone(&game_input);
        let input_thread = start_input_proc_thread(&mut game_input_ref, rx);

        self.concurrent_input = Some(ConcurrentInput {
            // game_input,
            game_input_ref,
            input_thread_tx: tx,
            input_thread,
        })
    }

    pub fn stop_concurrent_input(&mut self) {
        // clean up any existing threads before creating a new one
        self.terminate_concurrent_input();
        match self.concurrent_input.take() {
            Some(concurrent) => {
                concurrent.join_thread();
            }
            None => panic!("[InputHandler] ERROR: failed to reset concurrent thread!"),
        }
    }

    // pub fn get_names_under_mouse(&self) -> &str {
    //     match &self.input {
    //         Some(input) => {
    //             &input.names_under_mouse
    //         }
    //         None => {
    //             Default::default()
    //         }
    //     }
    // }

    // pub fn reset_names_under_mouse(&mut self) {
    //     self.names_under_mouse = Default::default();
    // }

    pub fn reset_next_action(&mut self) {
        match &self.next_action {
            Some(action) => panic!(
                "Why are we trying to reset an existing action? {:?}",
                action
            ),
            None => {}
        }
    }

    pub fn get_next_action(&mut self) -> Option<PlayerAction> {
        self.next_action.take()
    }

    pub fn check_for_next_action(
        &mut self,
        game_frontend: &mut GameFrontend,
        game_state: &mut GameState,
        objects: &mut GameObjects,
    ) {
        // use separate scope to get mutex to unlock again, could also use `Mutex::drop()`
        let concurrent_option = self.concurrent_input.take();
        match &concurrent_option {
            Some(concurrent) => {
                let mut data = concurrent.game_input_ref.lock().unwrap();
                // If the mouse moved, update the names and trigger re-render
                if self.check_set_mouse_position(data.mouse_x, data.mouse_y) {
                    self.names_under_mouse = get_names_under_mouse(
                        objects,
                        &game_frontend.fov,
                        self.current_mouse_pos.x,
                        self.current_mouse_pos.y,
                    );
                    re_render(game_frontend, game_state, objects, &self.names_under_mouse);
                    self.names_under_mouse = Default::default();
                }

                if let Some(ref player) = objects[PLAYER] {
                    // ... but only if the previous user action is used up
                    if player.next_action.is_none() {
                        if let Some(new_action) = data.next_player_actions.pop_front() {
                            self.next_action = Some(new_action);
                            println!(
                                "[frontend] player next action changed to {:?}",
                                self.next_action
                            );
                        }
                    }
                }
            }
            None => {
                panic!("[InputHandler] concurrent is not there");
            }
        }

        match concurrent_option {
            Some(concurrent) => {
                self.concurrent_input.replace(concurrent);
            }
            None => {
                panic!("[InputHandler] concurrent is still not there");
            }
        }
    }

    fn check_set_mouse_position(&mut self, new_x: i32, new_y: i32) -> bool {
        if self.current_mouse_pos.x != new_x || self.current_mouse_pos.y != new_y {
            self.current_mouse_pos.x = new_x;
            self.current_mouse_pos.y = new_y;
            return true;
        }
        false
    }

    fn terminate_concurrent_input(&self) {
        if let Some(concurrent) = &self.concurrent_input {
            match concurrent.input_thread_tx.send(true) {
                Ok(_) => {
                    println!("[frontend] terminating input proc thread");
                }
                Err(e) => {
                    println!("[frontend] error terminating input proc thread: {}", e);
                }
            }
        }
    }

    // pub fn take_thread(&mut self) -> Option<ConcurrentInput> {
    //     self.concurrent_input.take()
    // }
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
    let img = tcod::image::Image::from_file("assets/menu_background.png")
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
        game_frontend.root.print_ex(
            SCREEN_WIDTH / 2,
            SCREEN_HEIGHT / 2 - 4,
            BackgroundFlag::None,
            TextAlignment::Center,
            "I N N I T",
        );
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
                let (mut game_state, mut objects) = new_game();
                initialize_fov(game_frontend, &mut objects);
                let mut input_handler = InputHandler::new();
                input_handler.start_concurrent_input();
                game_loop(&mut game_state, game_frontend, &mut input_handler, &mut objects);
            }
            Some(1) => {
                // load game from file
                match load_game() {
                    Ok((mut game_state, mut objects)) => {
                        initialize_fov(game_frontend, &mut objects);
                        let mut input_handler = InputHandler::new();
                        input_handler.start_concurrent_input();
                        game_loop(&mut game_state, game_frontend, &mut input_handler, &mut objects);
                    }
                    Err(_e) => {
                        msgbox(game_frontend, &mut None, "\nNo saved game to load\n", 24);
                        continue;
                    }
                }
            }
            Some(2) => {
                //quit
                break;
            }
            _ => {}
        }
    }
}

/// Initialize the field of view map with a given instance of **World**
pub fn initialize_fov(game_frontend: &mut GameFrontend, objects: &mut GameObjects) {
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
}

pub fn pre_game_loop(game_state: &mut GameState, game_frontend: &mut GameFrontend, input_handler: &InputHandler, game_objects: &mut GameObjects) {
    recompute_fov(game_frontend, game_objects);
    re_render(
        game_frontend,
        game_state,
        game_objects,
        &input_handler.names_under_mouse,
    );
}

pub fn handle_ui_actions(
    game_frontend: &mut GameFrontend,
    game_state: &mut GameState,
    objects: &GameObjects,
    input_handler: &mut Option<&mut InputHandler>,
    action: UiAction,
) -> bool {
    match action {
        UiAction::ExitGameLoop => {
            save_game(game_state, objects).unwrap();
            return true;
        }
        UiAction::CharacterScreen => {
            // TODO: move this to separate function
            // show character information
            if let Some(ref player) = objects[PLAYER] {
                let level = player.level;
                let level_up_xp = LEVEL_UP_BASE + player.level * LEVEL_UP_FACTOR;
                if let Some(fighter) = player.fighter.as_ref() {
                    let msg = format!(
                        "Character information

Level: {}
Experience: {}
Experience to level up: {}

Maximum HP: {}
Attack: {}
Defense: {}",
                        level,
                        fighter.xp,
                        level_up_xp,
                        player.max_hp(game_state),
                        player.power(game_state),
                        player.defense(game_state),
                    );
                    msgbox(game_frontend, input_handler, &msg, CHARACTER_SCREEN_WIDTH);
                }
            };
        }

        UiAction::Fullscreen => {
            let fullscreen = game_frontend.root.is_fullscreen();
            game_frontend.root.set_fullscreen(!fullscreen);
        }
    }
    false
}

pub fn recompute_fov(game_frontend: &mut GameFrontend, objects: &GameObjects) {
    if let Some(ref player) = objects[PLAYER] {
        game_frontend
            .fov
            .compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALG);
    }
}

pub fn re_render(
    game_frontend: &mut GameFrontend,
    game_state: &mut GameState,
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

fn update_visibility(game_frontend: &mut GameFrontend, objects: &mut GameObjects) {
    // go through all tiles and set their background color
    let mut player_pos: (i32, i32) = (0, 0);
    if let Some(ref player) = objects[PLAYER] {
        player_pos = (player.x, player.y);
    }

    let col_dark_wall = game_frontend.coloring.get_col_dark_wall();
    let col_light_wall = game_frontend.coloring.get_col_light_wall();
    let col_dark_ground = game_frontend.coloring.get_col_dark_ground();
    let col_light_ground = game_frontend.coloring.get_col_light_ground();
    // TODO: Can this be done is a more functional programming way?
    for y in 0..WORLD_HEIGHT {
        for x in 0..WORLD_WIDTH {
            let visible = game_frontend.fov.is_in_fov(x, y);
            if let Some(ref mut tile_object) = objects.get_tile_at(x as usize, y as usize) {
                let wall = tile_object.physics.is_blocking_sight;
                let tile_color = match (visible, wall) {
                    // outside field of view:
                    (false, true) => col_dark_wall,
                    (false, false) => col_dark_ground,
                    // inside fov:
                    // (true, true) => COLOR_LIGHT_WALL,
                    (true, true) => colors::lerp(
                        col_light_wall,
                        col_dark_wall,
                        tile_object.distance(player_pos.0, player_pos.1) / TORCH_RADIUS as f32,
                    ),
                    // (true, false) => COLOR_LIGHT_GROUND,
                    (true, false) => colors::lerp(
                        col_light_ground,
                        col_dark_ground,
                        tile_object.distance(player_pos.0, player_pos.1) / TORCH_RADIUS as f32,
                    ),
                };

                if let Some(tile) = &mut tile_object.tile {
                    if visible {
                        tile.explored = true;
                    }
                    if tile.explored {
                        // show explored tiles only (any visible tile is explored already)
                        game_frontend.con.set_char_background(
                            x,
                            y,
                            tile_color,
                            BackgroundFlag::Set,
                        );
                    }
                }
            }
        }
    }
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
                || (o.physics.is_always_visible && *o.tile.as_ref().and_then(is_explored).unwrap())
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
            colors::LIGHT_RED,
            colors::DARKER_RED,
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
            if y < 0 {
                break;
            }
            game_frontend.panel.set_default_foreground(color);
            game_frontend.panel.print_rect(MSG_X, y, MSG_WIDTH, 0, msg);
        }
    }
}

/// Render a generic progress or status bar in the UI.
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

/// Display a generic menu with multiple options to choose from.
/// Returns the number of the menu item that has been chosen.
/// TODO: Make this private
pub fn menu<T: AsRef<str>>(
    game_frontend: &mut GameFrontend,
    input_handler: &mut Option<&mut InputHandler>,
    header: &str,
    options: &[T],
    width: i32,
) -> Option<usize> {
    assert!(
        options.len() <= 26,
        "Cannot have a mnu with more than 26 options."
    );

    // calculate total height for the header (after auto-wrap) and one line per option
    let header_height = if header.is_empty() {
        1
    } else {
        game_frontend
            .root
            .get_height_rect(0, 0, width, SCREEN_HEIGHT, header)
    };

    let height = options.len() as i32 + header_height + 1;

    // create an off-screen console that represents the menu's window
    let mut window = Offscreen::new(width, height);

    for x in 0..width {
        for y in 0..height {
            window.set_char_background(
                x,
                y,
                game_frontend.coloring.get_col_menu_bg(),
                BackgroundFlag::Set,
            );
            window.set_char(x, y, ' ');
        }
    }

    // render border
    for x in 0..width {
        window.set_char(x, 0, chars::BLOCK1);
        window.set_char(x, height - 1, chars::BLOCK1);
    }
    for y in 0..height - 1 {
        window.set_char(0, y, chars::BLOCK1);
        window.set_char(width - 1, y, chars::BLOCK1);
    }

    // print the header, with auto-wrap
    window.set_default_background(game_frontend.coloring.get_col_menu_bg());
    window.set_default_foreground(game_frontend.coloring.get_col_menu_fg());

    window.print_rect_ex(
        width / 2 as i32,
        0,
        width,
        height,
        BackgroundFlag::None,
        TextAlignment::Center,
        header,
    );

    // print all the options
    for (index, option_text) in options.iter().enumerate() {
        let menu_letter = (b'a' + index as u8) as char;
        let text = format!("({}) {}", menu_letter, option_text.as_ref());
        window.print_ex(
            1,
            header_height + index as i32,
            BackgroundFlag::None,
            TextAlignment::Left,
            text,
        );
    }

    // blit contents of "window" to the root console
    let x = SCREEN_WIDTH / 2 - width / 2;
    let y = SCREEN_HEIGHT / 2 - height / 2;
    tcod::console::blit(
        &window,
        (0, 0),
        (width, height),
        &mut game_frontend.root,
        (x, y),
        1.0,
        0.7,
    );

    // present the root console to the player and wait for a key-press
    game_frontend.root.flush();

    // if we have an instance of InputHandler, finish the input listener thread first
    // so that we can receive events here

    match input_handler {
        Some(handle) => {
            handle.stop_concurrent_input();
        }
        None => {}
    }

    let key = game_frontend.root.wait_for_keypress(true);
    // after we got he key, restart input listener thread

    match input_handler {
        Some(ref mut handle) => {
            handle.start_concurrent_input();
        }
        None => {}
    }

    // convert the ASCII code to an index
    // if it corresponds to an option, return it
    if key.printable.is_alphabetic() {
        let index = key.printable.to_ascii_lowercase() as usize - 'a' as usize;
        if index < options.len() {
            Some(index)
        } else {
            None
        }
    } else {
        None
    }
}

/// Display a box with a message to the user.
/// This works like a menu, but without any choices.
fn msgbox(
    game_frontend: &mut GameFrontend,
    input_handler: &mut Option<&mut InputHandler>,
    text: &str,
    width: i32,
) {
    let options: &[&str] = &[];
    menu(game_frontend, input_handler, text, options, width);
}
