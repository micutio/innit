/// Module GUI
///
/// This module contains all structures and methods pertaining to the user interface.
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use tcod::colors::{self, Color};
use tcod::console::*;
use tcod::map::FovAlgorithm;

// internal modules
use entity::ai::ai_take_turn;
use entity::object::Object;
use game_state::{level_up, new_game, GameState, PLAYER, TORCH_RADIUS};
use world::{World, WORLD_HEIGHT, WORLD_WIDTH};

use ui::color_palette::*;
use ui::game_input::{handle_keys, GameInput, PlayerAction};

// GUI constraints
// window size
pub const SCREEN_WIDTH: i32 = 80;
pub const SCREEN_HEIGHT: i32 = 50;
// target fps
pub const LIMIT_FPS: i32 = 20;
// constraints for field of view computing and rendering
const FOV_ALG: FovAlgorithm = FovAlgorithm::Shadow;
const FOV_LIGHT_WALLS: bool = true;

// Menu constraints
const BAR_WIDTH: i32 = 20;
pub const PANEL_HEIGHT: i32 = 7;
const PANEL_Y: i32 = SCREEN_HEIGHT - PANEL_HEIGHT;
const MSG_X: i32 = BAR_WIDTH + 2;
const MSG_WIDTH: i32 = SCREEN_WIDTH - BAR_WIDTH - 2;
const MSG_HEIGHT: usize = PANEL_HEIGHT as usize - 1;

pub const CHARACTER_SCREEN_WIDTH: i32 = 30;

/// Field of view mapping
pub use tcod::map::Map as FovMap;

/// GameIO holds he core components for game's input and output processing.
pub struct GameFrontend {
    pub root: Root,
    pub con: Offscreen,
    pub panel: Offscreen,
    pub fov: FovMap,
    // pub mouse: Mouse,
}

pub type Messages = Vec<(String, Color)>;

pub trait MessageLog {
    fn add<T: Into<String>>(&mut self, message: T, color: Color);
}

impl MessageLog for Vec<(String, Color)> {
    fn add<T: Into<String>>(&mut self, message: T, color: Color) {
        self.push((message.into(), color));
    }
}

pub fn initialize_io() -> GameFrontend {
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
        // mouse: Default::default(),
    }
}

pub fn initialize_fov(world: &World, game_frontend: &mut GameFrontend) {
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
    game_frontend.con.clear(); // unexplored areas start black (which is the default background color)
}

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

/// Render all objects and tiles.
pub fn render_all(
    game_frontend: &mut GameFrontend,
    game_state: &mut GameState,
    objects: &[Object],
    fov_recompute: bool,
    names_under_mouse: &str,
) {
    if fov_recompute {
        // recompute fov if needed (the player moved or something)
        let player = &objects[PLAYER];
        game_frontend
            .fov
            .compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALG);
    }

    // go through all tiles and set their background color
    for y in 0..WORLD_HEIGHT {
        for x in 0..WORLD_WIDTH {
            let visible = game_frontend.fov.is_in_fov(x, y);
            let wall = game_state.world[x as usize][y as usize].block_sight;
            let tile_color = match (visible, wall) {
                // outside field of view:
                (false, true) => get_col_dark_wall(),
                (false, false) => get_col_dark_ground(),
                // inside fov:
                // (true, true) => COLOR_LIGHT_WALL,
                (true, true) => colors::lerp(
                    get_col_light_wall(),
                    get_col_dark_wall(),
                    objects[PLAYER].distance(x, y) / TORCH_RADIUS as f32,
                ),
                // (true, false) => COLOR_LIGHT_GROUND,
                (true, false) => colors::lerp(
                    get_col_light_ground(),
                    get_col_dark_ground(),
                    objects[PLAYER].distance(x, y) / TORCH_RADIUS as f32,
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

    let mut to_draw: Vec<&Object> = objects
        .iter()
        .filter(|o| {
            game_frontend.fov.is_in_fov(o.x, o.y)
                || (o.always_visible && game_state.world[o.x as usize][o.y as usize].explored)
        })
        .collect();
    // sort, so that non-blocking objects com first
    to_draw.sort_by(|o1, o2| o1.blocks.cmp(&o2.blocks));
    // draw the objects in the list
    for object in &to_draw {
        object.draw(&mut game_frontend.con);
    }

    // prepare to render the GUI panel
    game_frontend.panel.set_default_background(colors::BLACK);
    game_frontend.panel.clear();

    // show player's stats
    let hp = objects[PLAYER].fighter.map_or(0, |f| f.hp);
    let max_hp = objects[PLAYER].fighter.map_or(0, |f| f.base_max_hp);
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

pub fn menu<T: AsRef<str>>(
    header: &str,
    options: &[T],
    width: i32,
    root: &mut Root,
) -> Option<usize> {
    assert!(
        options.len() <= 26,
        "Cannot have a mnu with more than 26 options."
    );

    // calculate total height for the header (after auto-wrap) and one line per option
    let header_height = if header.is_empty() {
        0
    } else {
        root.get_height_rect(0, 0, width, SCREEN_HEIGHT, header)
    };

    let height = options.len() as i32 + header_height;

    // create an off-screen console that represents the menu's window
    let mut window = Offscreen::new(width, height);

    // print the header, with auto-wrap
    window.set_default_foreground(colors::WHITE);
    window.print_rect_ex(
        0,
        0,
        width,
        height,
        BackgroundFlag::None,
        TextAlignment::Left,
        header,
    );

    // print all the options
    for (index, option_text) in options.iter().enumerate() {
        let menu_letter = (b'a' + index as u8) as char;
        let text = format!("({}) {}", menu_letter, option_text.as_ref());
        window.print_ex(
            0,
            header_height + index as i32,
            BackgroundFlag::None,
            TextAlignment::Left,
            text,
        );
    }

    // blit contents of "window" to the root console
    let x = SCREEN_WIDTH / 2 - width / 2;
    let y = SCREEN_HEIGHT / 2 - height / 2;
    tcod::console::blit(&window, (0, 0), (width, height), root, (x, y), 1.0, 0.7);

    // present the root console to the player and wait for a key-press
    root.flush();
    let key = root.wait_for_keypress(true);

    // convert the ASCII code to and index; if if corresponds to an option, return it
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

pub fn msgbox(text: &str, width: i32, root: &mut Root) {
    let options: &[&str] = &[];
    menu(text, options, width, root);
}

pub fn main_menu(game_frontend: &mut GameFrontend, game_input: &mut GameInput) {
    let img = tcod::image::Image::from_file("assets/menu_background.png")
        .expect("Background image not found");

    while !game_frontend.root.window_closed() {
        // show the background image, at twice the regular console resolution
        tcod::image::blit_2x(&img, (0, 0), (-1, -1), &mut game_frontend.root, (0, 0));

        game_frontend
            .root
            .set_default_foreground(colors::LIGHT_YELLOW);
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
        let choice = menu("", choices, 24, &mut game_frontend.root);

        match choice {
            Some(0) => {
                // start new game
                let (mut objects, mut game_state) = new_game();
                initialize_fov(&game_state.world, game_frontend);
                game_loop(&mut objects, &mut game_state, game_frontend, game_input);
            }
            Some(1) => {
                // load game from file
                match load_game() {
                    Ok((mut objects, mut game_state)) => {
                        initialize_fov(&game_state.world, game_frontend);
                        game_loop(&mut objects, &mut game_state, game_frontend, game_input);
                    }
                    Err(_e) => {
                        msgbox("\nNo saved game to load\n", 24, &mut game_frontend.root);
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

pub fn save_game(objects: &[Object], game_state: &GameState) -> Result<(), Box<Error>> {
    let save_data = serde_json::to_string(&(objects, game_state))?;
    let mut file = File::create("savegame")?;
    file.write_all(save_data.as_bytes())?;
    Ok(())
}

pub fn load_game() -> Result<(Vec<Object>, GameState), Box<Error>> {
    let mut json_save_state = String::new();
    let mut file = File::open("savegame")?;
    file.read_to_string(&mut json_save_state)?;
    let result = serde_json::from_str::<(Vec<Object>, GameState)>(&json_save_state)?;
    Ok(result)
}

/// Central function of the game.
/// - process player input
/// - render game world
/// - let NPCs take their turn
pub fn game_loop(
    objects: &mut Vec<Object>,
    game_state: &mut GameState,
    game_frontend: &mut GameFrontend,
    game_input: &mut GameInput,
) {
    // force FOV "recompute" first time through the game loop
    let mut previous_player_position = (-1, -1);

    // input processing
    // let mut key: Key = Default::default();

    while !game_frontend.root.window_closed() {
        // clear the screen of the previous frame
        game_frontend.con.clear();

        // check for input events
        game_input.check_for_input_events(objects, &game_frontend.fov);

        // render objects and map
        let fov_recompute = previous_player_position != (objects[PLAYER].x, objects[PLAYER].y);
        render_all(
            game_frontend,
            game_state,
            objects,
            fov_recompute,
            &game_input.names_under_mouse,
        );

        // draw everything on the window at once
        game_frontend.root.flush();

        // level up if needed
        level_up(objects, game_state, game_frontend);

        // handle keys and exit game if needed
        // TODO: Generate and `action` from the player input and set the player object to execute it.
        previous_player_position = objects[PLAYER].pos();
        let player_action = handle_keys(game_frontend, game_input, game_state, objects);
        if player_action == PlayerAction::Exit {
            save_game(objects, game_state).unwrap();
            break;
        }

        // let monsters take their turn
        if objects[PLAYER].alive && player_action != PlayerAction::DidntTakeTurn {
            for id in 0..objects.len() {
                if objects[id].ai.is_some() {
                    ai_take_turn(game_state, objects, &game_frontend.fov, id);
                }
            }
        }
    }
}
