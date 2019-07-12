/// Module GUI
///
/// This module contains all structures and methods pertaining to the user interface.
///
/// Game frontend, input, state, engine etc are very common parameters
/// for function calls in innit, therefore utmost consistency should be
/// kept in the order of handing them over to functions.
///
/// Function parameter precedence:
/// game_frontend, game_input, game_engine, game_state, objects, anything else.
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use tcod::chars;
use tcod::colors::{self, Color};
use tcod::console::*;
use tcod::map::FovAlgorithm;

use core::game_state::{
    level_up, new_game, GameState, ObjectProcResult, LEVEL_UP_BASE, LEVEL_UP_FACTOR, PLAYER,
    TORCH_RADIUS,
};
use core::world::{World, WORLD_HEIGHT, WORLD_WIDTH};
use entity::object::{GameObjects, Object};
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

    pub fn init_input(&mut self) {
        // clean up any existing threads before creating a new one
        self.terminate_input_thread();
        self.join_input_thread();
        self.input = Some(InputHandler::new());
    }

    pub fn get_names_under_mouse(&self) -> &str {
        match &self.input {
            Some(input) => {
                &input.names_under_mouse
            }
            None => {
                Default::default()
            }
        }
    }

    pub fn reset_names_under_mouse(&mut self){
        match &self.input {
            Some(input) => {
                input.names_under_mouse = Default::default();
            }
            None => {}
        }
    }

    pub fn reset_next_action(&mut self) {
        match &mut self.input {
            Some(input) => {
                input.next_action = None
            }
            None => {}
        }
    }

    pub fn get_next_action(&mut self) -> Option<PlayerAction> {
        match &mut self.input {
            Some(input) => {
                input.next_action.take()
            }
            None => None
        }
    }

    pub fn check_for_next_action(&mut self, game_state: &mut GameState, objects: &GameObjects) {
        let need_re_render = false;
        let _input = self.input.take();
        match _input {
            Some(mut input_handler) => {
                // use separate scope to get mutex to unlock again, could also use `Mutex::drop()`
                let data = input_handler.game_input_ref.lock().unwrap();
                // If the mouse moved, update the names and trigger re-render
                need_re_render = check_set_mouse_position(&mut input_handler, data.mouse_x, data.mouse_y);
                
            }
            None => {

            }
        }



        self.input.replace(_input.unwrap());


        if need_re_render {
            input.names_under_mouse = get_names_under_mouse(
                objects,
                &self.fov,
                input.current_mouse_pos.x,
                input.current_mouse_pos.y,
            );
            re_render(self, game_state, objects);
            self.reset_names_under_mouse();
        }

        if let Some(ref player) = objects[PLAYER] {
            // ... but only if the previous user action is used up
            if player.next_action.is_none() {
                if let Some(new_action) = data.next_player_actions.pop_front() {
                    input.next_action = Some(new_action);
                    println!("[frontend] player next action changed to {:?}", input.next_action);
                }
            }
        }
    }

    pub fn terminate_input_thread(&mut self) {
        match &self.input {
            Some(input) => {
                match input.input_thread_channel.0.send(true) {
                    Ok(_) => {
                        println!("[frontend] terminating input proc thread");
                    }
                    Err(e) => {
                        println!("[frontend] error terminating input proc thread: {}", e);
                    }
                }
        }
            None => {}
        }
    }

    pub fn join_input_thread(&mut self) {
        match &self.input {
            Some(input) => {
                let join_result = input.input_thread.join();
                match join_result {
                    Ok(_) => println!("[join] successfully joined user input thread"),
                    Err(e) => println!(
                        "[join] error while trying to join user input thread: {:?}",
                        e
                    ),
                }
            }
            None => {

            }
        }
        
    }
}

struct InputHandler {
    current_mouse_pos: MousePosition,
    names_under_mouse: String,
    game_input: Arc<Mutex<GameInput>>,
    game_input_ref: Arc<Mutex<GameInput>>,
    input_thread_channel: (Sender<bool>, Receiver<bool>),
    input_thread: JoinHandle<()>,
    next_action: Option<PlayerAction>,
}

struct MousePosition {
    x: i32,
    y: i32,
}

impl InputHandler {
    fn new() -> Self {
        // user input data
        let mut current_mouse_pos: (i32, i32) = Default::default();
        let mut next_action: Option<PlayerAction>;
        let mut names_under_mouse: String = Default::default();

        // concurrent input processing
        let (tx, rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();
        let mut game_input = Arc::new(Mutex::new(GameInput::new()));
        let game_input_ref = Arc::clone(&game_input);
        // let input_thread = ;

        InputHandler {
            current_mouse_pos: MousePosition{x: 0, y: 0},
            names_under_mouse: Default::default(),
            game_input,
            game_input_ref,
            input_thread_channel: (tx, rx),
            input_thread: start_input_proc_thread(&mut game_input, rx),
            next_action,
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
        let choice = menu(game_frontend, "main menu", choices, 24);

        match choice {
            Some(0) => {
                // start new game
                let (mut game_state, mut objects) = new_game();
                initialize_fov(game_frontend, &game_state.world);
                game_loop(game_frontend, &mut game_state, &mut objects);
            }
            Some(1) => {
                // load game from file
                match load_game() {
                    Ok((mut game_state, mut objects)) => {
                        initialize_fov(game_frontend, &game_state.world);
                        game_loop(game_frontend, &mut game_state, &mut objects);
                    }
                    Err(_e) => {
                        msgbox(game_frontend, "\nNo saved game to load\n", 24);
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
pub fn initialize_fov(game_frontend: &mut GameFrontend, world: &World) {
    // init fov map
    for y in 0..WORLD_HEIGHT {
        for x in 0..WORLD_WIDTH {
            game_frontend.fov.set(
                x,
                y,
                !world[x as usize][y as usize].block_sight,
                !world[x as usize][y as usize].blocked,
            );
        }
    }
    // unexplored areas start black (which is the default background color)
    game_frontend.con.clear();
}

/// Central function of the game.
/// - process player input
/// - render game world
/// - let NPCs take their turn
pub fn game_loop(
    game_frontend: &mut GameFrontend,
    game_state: &mut GameState,
    objects: &mut GameObjects,
) {
    // step 1/3: pre-processing ///////////////////////////////////////////////

    recompute_fov(game_frontend, objects);
    re_render(game_frontend, game_state, objects);

    // step 2/2: the actual loop //////////////////////////////////////////////
    while !game_frontend.root.window_closed() {
        game_frontend.reset_next_action();
        // let the game engine process an object
        let proc_result = game_state.process_object(&game_frontend.fov, objects);
        match proc_result {
            // no action has been performed, repeat the turn and try again
            ObjectProcResult::NoAction => {}

            // action has been completed, but nothing needs to be done about it
            ObjectProcResult::NoFeedback => {}

            // the player's FOV has been updated, thus we also need to re-render
            ObjectProcResult::UpdateFOV => {
                recompute_fov(game_frontend, objects);
                re_render(game_frontend, game_state, objects);
            }

            // the player hasn't moved but something happened within fov
            ObjectProcResult::ReRender => {
                re_render(game_frontend, game_state, objects);
            }

            ObjectProcResult::Animate { anim_type } => {
                // TODO: Play animation.
                println!("animation");
            }

            _ => {}
        }

        // once processing is done, check whether we have a new user input
        game_frontend.check_for_next_action(game_state, objects);

        // distinguish between in-game action and ui (=meta) actions
        match game_frontend.get_next_action() {
            Some(PlayerAction::MetaAction(actual_action)) => {
                println!("[frontend] process UI action: {:?}", actual_action);
                let is_exit_game =
                    handle_ui_actions(game_frontend, game_state, objects, actual_action);
                if is_exit_game {
                    
                    break;
                }
            }
            Some(ingame_action) => {
                // let mut player = objects.mut_obj(PLAYER);
                // *player.set_next_action(Some(get_player_action_instance(next_action)));
                // objects.mut_obj(PLAYER).unwrap().set_next_action(Some(get_player_action_instance(next_action)));
                println!(
                    "[frontend] inject ingame action {:?} to player",
                    ingame_action
                );
                if let Some(ref mut player) = objects[PLAYER] {
                    let player_next_action = Some(get_player_action_instance(ingame_action));
                    println!("[frontend] player action object: {:?}", player_next_action);
                    player.set_next_action(player_next_action);
                };
            }
            None => {}
        }

        // level up if needed
        // level_up(objects, game_state, game_frontend);
    }

    // step 3/3: cleanup before returning to main menu
    game_frontend.join_input_thread();
}

/// Load an existing savegame and instantiates GameState & Objects
/// from which the game is resumed in the game loop.
fn load_game() -> Result<(GameState, GameObjects), Box<Error>> {
    let mut json_save_state = String::new();
    let mut file = File::open("savegame")?;
    file.read_to_string(&mut json_save_state)?;
    let result = serde_json::from_str::<(GameState, GameObjects)>(&json_save_state)?;
    Ok(result)
}

/// Serialize and store GameState and Objects into a JSON file.
fn save_game(game_state: &GameState, objects: &GameObjects) -> Result<(), Box<Error>> {
    let save_data = serde_json::to_string(&(game_state, objects))?;
    let mut file = File::create("savegame")?;
    file.write_all(save_data.as_bytes())?;
    Ok(())
}

pub fn check_set_mouse_position(input: &mut InputHandler, new_x: i32, new_y: i32) -> bool {
    if input.current_mouse_pos.x != new_x || input.current_mouse_pos.y != new_y {
        input.current_mouse_pos.x = new_x;
        input.current_mouse_pos.y = new_y;
        return true;
    }
    false
}

fn handle_ui_actions(
    game_frontend: &mut GameFrontend,
    game_state: &mut GameState,
    objects: &GameObjects,
    action: UiAction,
) -> bool {
    match action {
        UiAction::ExitGameLoop => {
            save_game(game_state, objects).unwrap();
            println!("RETURN TRUE");
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
                    msgbox(game_frontend, &msg, CHARACTER_SCREEN_WIDTH);
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

fn recompute_fov(game_frontend: &mut GameFrontend, objects: &GameObjects) {
    if let Some(ref player) = objects[PLAYER] {
        game_frontend
            .fov
            .compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALG);
    }
}

fn re_render(
    game_frontend: &mut GameFrontend,
    game_state: &mut GameState,
    objects: &GameObjects,
) {
    // clear the screen of the previous frame
    game_frontend.con.clear();
    // render objects and map
    // step 1/2: update visibility of objects and world tiles
    update_visibility(game_frontend, game_state, objects);
    // step 2/2: render everything
    render_all(game_frontend, game_state, objects);

    // draw everything on the window at once
    game_frontend.root.flush();
}

fn update_visibility(
    game_frontend: &mut GameFrontend,
    game_state: &mut GameState,
    objects: &GameObjects,
) {
    // go through all tiles and set their background color
    if let Some(ref player) = objects[PLAYER] {
        let col_dark_wall = game_frontend.coloring.get_col_dark_wall();
        let col_light_wall = game_frontend.coloring.get_col_light_wall();
        let col_dark_ground = game_frontend.coloring.get_col_dark_ground();
        let col_light_ground = game_frontend.coloring.get_col_light_ground();
        for y in 0..WORLD_HEIGHT {
            for x in 0..WORLD_WIDTH {
                let visible = game_frontend.fov.is_in_fov(x, y);
                let wall = game_state.world[x as usize][y as usize].block_sight;
                let tile_color = match (visible, wall) {
                    // outside field of view:
                    (false, true) => col_dark_wall,
                    (false, false) => col_dark_ground,
                    // inside fov:
                    // (true, true) => COLOR_LIGHT_WALL,
                    (true, true) => colors::lerp(
                        col_light_wall,
                        col_dark_wall,
                        player.distance(x, y) / TORCH_RADIUS as f32,
                    ),
                    // (true, false) => COLOR_LIGHT_GROUND,
                    (true, false) => colors::lerp(
                        col_light_ground,
                        col_dark_ground,
                        player.distance(x, y) / TORCH_RADIUS as f32,
                    ),
                };

                let explored = &mut game_state.world[x as usize][y as usize].explored;
                if visible {
                    *explored = true;
                }
                if *explored {
                    // show explored tiles only (any visible tile is explored already)
                    game_frontend
                        .con
                        .set_char_background(x, y, tile_color, BackgroundFlag::Set);
                }
            }
        }
    }
}

/// Render all objects and tiles.
/// Right now this happens because we are updating explored tiles here too.
/// Is there a way to auto-update explored and visible tiles/objects when the player moves?
/// But visibility can also be influenced by other things.
pub fn render_all(
    game_frontend: &mut GameFrontend,
    game_state: &mut GameState,
    objects: &GameObjects,
) {
    let mut to_draw: Vec<&Object> = objects
        .get_vector()
        .iter()
        .flatten()
        .filter(|o| {
            game_frontend.fov.is_in_fov(o.x, o.y)
                || (o.always_visible && game_state.world[o.x as usize][o.y as usize].explored)
        })
        .collect();
    // sort, so that non-blocking objects come first
    to_draw.sort_by(|o1, o2| o1.blocks.cmp(&o2.blocks));
    // draw the objects in the list
    for object in &to_draw {
        object.draw(&mut game_frontend.con);
    }

    render_ui(game_frontend, game_state, objects);
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
            game_frontend.get_names_under_mouse(),
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

/// Display a generic menu with multiple options to choose from.
/// Returns the number of the menu item that has been chosen.
pub fn menu<T: AsRef<str>>(
    game_frontend: &mut GameFrontend,
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
    let key = game_frontend.root.wait_for_keypress(true);

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
pub fn msgbox(game_frontend: &mut GameFrontend, text: &str, width: i32) {
    let options: &[&str] = &[];
    menu(game_frontend, text, options, width);
}
