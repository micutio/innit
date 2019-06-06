//! Module GUI
//!
//! This module contains all structures and methods pertaining to the user interface.

use crate::{
    drop_item, level_up, next_level, pick_item_up, player_move_or_attack, use_item, DeathCallback,
    Equipment, Fighter, FovMap, GameState, Item, PlayerAction, Slot, LEVEL_UP_BASE,
    LEVEL_UP_FACTOR, PLAYER, TORCH_RADIUS,
};

use ai::ai_take_turn;
use object::Object;
use world::{make_world, World, WORLD_HEIGHT, WORLD_WIDTH};

use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use tcod::colors::{self, Color};
use tcod::console::*;
use tcod::input::{self, Event, Key, Mouse};
use tcod::map::FovAlgorithm;

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

const INVENTORY_WIDTH: i32 = 50;
const CHARACTER_SCREEN_WIDTH: i32 = 30;

const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_LIGHT_WALL: Color = Color {
    r: 130,
    g: 110,
    b: 50,
};
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};
const COLOR_LIGHT_GROUND: Color = Color {
    r: 200,
    g: 180,
    b: 50,
};

pub type Messages = Vec<(String, Color)>;

pub struct Tcod {
    pub root: Root,
    pub con: Offscreen,
    pub panel: Offscreen,
    pub fov: FovMap,
    pub mouse: Mouse,
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

pub trait MessageLog {
    fn add<T: Into<String>>(&mut self, message: T, color: Color);
}

impl MessageLog for Vec<(String, Color)> {
    fn add<T: Into<String>>(&mut self, message: T, color: Color) {
        self.push((message.into(), color));
    }
}

/// Render all objects and tiles.
fn render_all(
    tcod: &mut Tcod,
    game_state: &mut GameState,
    objects: &[Object],
    fov_recompute: bool,
) {
    if fov_recompute {
        // recompute fov if needed (the player moved or something)
        let player = &objects[PLAYER];
        tcod.fov
            .compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALG);
    }

    // go through all tiles and set their background color
    for y in 0..WORLD_HEIGHT {
        for x in 0..WORLD_WIDTH {
            let visible = tcod.fov.is_in_fov(x, y);
            let wall = game_state.world[x as usize][y as usize].block_sight;
            let tile_color = match (visible, wall) {
                // outside field of view:
                (false, true) => COLOR_DARK_WALL,
                (false, false) => COLOR_DARK_GROUND,
                // inside fov:
                // (true, true) => COLOR_LIGHT_WALL,
                (true, true) => colors::lerp(
                    COLOR_LIGHT_WALL,
                    COLOR_DARK_WALL,
                    objects[PLAYER].distance(x, y) / TORCH_RADIUS as f32,
                ),
                // (true, false) => COLOR_LIGHT_GROUND,
                (true, false) => colors::lerp(
                    COLOR_LIGHT_GROUND,
                    COLOR_DARK_GROUND,
                    objects[PLAYER].distance(x, y) / TORCH_RADIUS as f32,
                ),
            };

            let explored = &mut game_state.world[x as usize][y as usize].explored;
            if visible {
                *explored = true;
            }
            if *explored {
                // show explored tiles only (any visible tile is explored already)
                tcod.con
                    .set_char_background(x, y, tile_color, BackgroundFlag::Set);
            }
        }
    }

    let mut to_draw: Vec<_> = objects
        .iter()
        .filter(|o| {
            tcod.fov.is_in_fov(o.x, o.y)
                || (o.always_visible && game_state.world[o.x as usize][o.y as usize].explored)
        })
        .collect();
    // sort, so that non-blocking objects com first
    to_draw.sort_by(|o1, o2| o1.blocks.cmp(&o2.blocks));
    // draw the objects in the list
    for object in &to_draw {
        object.draw(&mut tcod.con);
    }

    // prepare to render the GUI panel
    tcod.panel.set_default_background(colors::BLACK);
    tcod.panel.clear();

    // show player's stats
    let hp = objects[PLAYER].fighter.map_or(0, |f| f.hp);
    let max_hp = objects[PLAYER].fighter.map_or(0, |f| f.base_max_hp);
    render_bar(
        &mut tcod.panel,
        1,
        1,
        BAR_WIDTH,
        "HP",
        hp,
        max_hp,
        colors::LIGHT_RED,
        colors::DARKER_RED,
    );
    tcod.panel.print_ex(
        1,
        2,
        BackgroundFlag::None,
        TextAlignment::Left,
        format!("Dungeon level: {}", game_state.dungeon_level),
    );

    // show names of objects under the mouse
    tcod.panel.set_default_foreground(colors::LIGHT_GREY);
    tcod.panel.print_ex(
        1,
        0,
        BackgroundFlag::None,
        TextAlignment::Left,
        get_names_under_mouse(tcod.mouse, objects, &tcod.fov),
    );

    // print game messages, one line at a time
    let mut y = MSG_HEIGHT as i32;
    for &(ref msg, color) in &mut game_state.log.iter().rev() {
        let msg_height = tcod.panel.get_height_rect(MSG_X, y, MSG_WIDTH, 0, msg);
        y -= msg_height;
        if y < 0 {
            break;
        }
        tcod.panel.set_default_foreground(color);
        tcod.panel.print_rect(MSG_X, y, MSG_WIDTH, 0, msg);
    }

    // blit contents of `tcod.panel` to the root console
    blit(
        &tcod.panel,
        (0, 0),
        (SCREEN_WIDTH, SCREEN_HEIGHT),
        &mut tcod.root,
        (0, PANEL_Y),
        1.0,
        1.0,
    );

    // blit contents of offscreen console to root console and present it
    blit(
        &tcod.con,
        (0, 0),
        (WORLD_WIDTH, WORLD_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0,
    );
}

/// Handle user input
fn handle_keys(
    tcod: &mut Tcod,
    game_state: &mut GameState,
    objects: &mut Vec<Object>,
    key: Key,
) -> PlayerAction {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;
    use PlayerAction::*;

    let player_alive = objects[PLAYER].alive;
    match (key, player_alive) {
        // toggle fullscreen
        (
            Key {
                code: Enter,
                alt: true,
                ..
            },
            _,
        ) => {
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
            DidntTakeTurn
        }

        // exit game
        (Key { code: Escape, .. }, _) => Exit,

        // handle movement
        (Key { code: Up, .. }, true) | (Key { printable: 'w', .. }, true) => {
            player_move_or_attack(game_state, objects, 0, -1);
            TookTurn
        }
        (Key { code: Down, .. }, true) | (Key { printable: 's', .. }, true) => {
            player_move_or_attack(game_state, objects, 0, 1);
            TookTurn
        }
        (Key { code: Left, .. }, true) | (Key { printable: 'a', .. }, true) => {
            player_move_or_attack(game_state, objects, -1, 0);
            TookTurn
        }
        (Key { code: Right, .. }, true) | (Key { printable: 'd', .. }, true) => {
            player_move_or_attack(game_state, objects, 1, 0);
            TookTurn
        }
        (Key { printable: 'x', .. }, true) => {
            // do nothing, i.e. wait for the monster to come to you
            TookTurn
        }
        (Key { printable: 'g', .. }, true) => {
            // pick up an item
            let item_id = objects
                .iter()
                .position(|object| object.pos() == objects[PLAYER].pos() && object.item.is_some());
            if let Some(item_id) = item_id {
                pick_item_up(game_state, objects, item_id);
            }
            DidntTakeTurn
        }
        (Key { printable: 'i', .. }, true) => {
            // show the inventory: if an item is selected, use it
            let inventory_index = inventory_menu(
                &mut tcod.root,
                &game_state.inventory,
                "Press the key next to an item to use it, or any other to cancel.\n",
            );
            if let Some(inventory_index) = inventory_index {
                use_item(tcod, game_state, objects, inventory_index);
            }
            DidntTakeTurn
        }
        (Key { printable: 'd', .. }, true) => {
            // show_inventory; if an item is selected, drop it
            let inventory_index = inventory_menu(
                &mut tcod.root,
                &game_state.inventory,
                "Press the key enxt to an item to drop it, or any other to cancel.\n",
            );
            if let Some(inventory_index) = inventory_index {
                drop_item(game_state, objects, inventory_index);
            }
            DidntTakeTurn
        }
        (Key { printable: 'e', .. }, true) => {
            // go down the stairs, if the player is on them
            println!("trying to go down stairs");
            let player_on_stairs = objects
                .iter()
                .any(|object| object.pos() == objects[PLAYER].pos() && object.name == "stairs");
            if player_on_stairs {
                next_level(tcod, objects, game_state);
            }
            DidntTakeTurn
        }
        (Key { printable: 'c', .. }, true) => {
            // show character information
            let player = &objects[PLAYER];
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
                msgbox(&msg, CHARACTER_SCREEN_WIDTH, &mut tcod.root);
            }

            DidntTakeTurn
        }

        _ => DidntTakeTurn,
    }
}

fn get_names_under_mouse(mouse: Mouse, objects: &[Object], fov_map: &FovMap) -> String {
    let (x, y) = (mouse.cx as i32, mouse.cy as i32);

    // create a list with the names of all objects at the mouse's coordinates and in FOV
    let names = objects
        .iter()
        .filter(|obj| obj.pos() == (x, y) && fov_map.is_in_fov(obj.x, obj.y))
        .map(|obj| obj.name.clone())
        .collect::<Vec<_>>();

    names.join(", ") // return names separated by commas
}

/// return the position of a tile left-clicked in player's FOV (optionally in a range),
/// or (None, None) if right-clicked.
pub fn target_tile(
    tcod: &mut Tcod,
    game_state: &mut GameState,
    objects: &[Object],
    max_range: Option<f32>,
) -> Option<(i32, i32)> {
    use tcod::input::KeyCode::Escape;
    loop {
        // render the screen. this erases the inventory and shows the names of objects under the mouse
        tcod.root.flush();
        let event = input::check_for_event(input::KEY_PRESS | input::MOUSE).map(|e| e.1);
        let mut key = None;
        match event {
            Some(Event::Mouse(m)) => tcod.mouse = m,
            Some(Event::Key(k)) => key = Some(k),
            None => {}
        }
        render_all(tcod, game_state, objects, false);

        let (x, y) = (tcod.mouse.cx as i32, tcod.mouse.cy as i32);

        // accept the target if the player clicked in FOV, and in case a range is specified, if it's in that range
        let in_fov = (x < WORLD_WIDTH) && (y < WORLD_HEIGHT) && tcod.fov.is_in_fov(x, y);
        let in_range = max_range.map_or(true, |range| objects[PLAYER].distance(x, y) <= range);

        if tcod.mouse.lbutton_pressed && in_fov && in_range {
            return Some((x, y));
        }

        let escape = key.map_or(false, |k| k.code == Escape);
        if tcod.mouse.rbutton_pressed || escape {
            return None; // cancel if the player right-clicked or pressed Escape
        }
    }
}

pub fn target_monster(
    tcod: &mut Tcod,
    game_state: &mut GameState,
    objects: &[Object],
    max_range: Option<f32>,
) -> Option<usize> {
    loop {
        match target_tile(tcod, game_state, objects, max_range) {
            Some((x, y)) => {
                // return the first clicked monster, otherwise continue looping
                for (id, obj) in objects.iter().enumerate() {
                    if obj.pos() == (x, y) && obj.fighter.is_some() && id != PLAYER {
                        return Some(id);
                    }
                }
            }
            None => return None,
        }
    }
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

fn msgbox(text: &str, width: i32, root: &mut Root) {
    let options: &[&str] = &[];
    menu(text, options, width, root);
}

fn inventory_menu(root: &mut Root, inventory: &[Object], header: &str) -> Option<usize> {
    // how a menu with each item of the inventory as an option
    let options = if inventory.is_empty() {
        vec!["Inventory is empty.".into()]
    } else {
        // inventory.iter().map(|item| item.name.clone()).collect()
        inventory
            .iter()
            .map(|item| {
                // show additional information, in case it's equipped
                match item.equipment {
                    Some(equipment) if equipment.equipped => {
                        format!("{} (on {})", item.name, equipment.slot)
                    }
                    _ => item.name.clone(),
                }
            })
            .collect()
    };

    let inventory_index = menu(header, &options, INVENTORY_WIDTH, root);

    // if an item was chosen, return if
    if !inventory.is_empty() {
        inventory_index
    } else {
        None
    }
}

fn new_game(tcod: &mut Tcod) -> (Vec<Object>, GameState) {
    // create object representing the player
    let mut player = Object::new(0, 0, "player", true, '@', colors::WHITE);
    player.alive = true;
    player.fighter = Some(Fighter {
        base_max_hp: 100,
        hp: 100,
        base_defense: 1,
        base_power: 2,
        on_death: DeathCallback::Player,
        xp: 0,
    });

    // create array holding all objects
    let mut objects = vec![player];
    let level = 1;

    // create game state holding most game-relevant information
    //  - also creates map and player starting position
    let mut game_state = GameState {
        // generate map (at this point it's not drawn on screen)
        world: make_world(&mut objects, level),
        // create the list of game messages and their colors, starts empty
        log: vec![],
        inventory: vec![],
        dungeon_level: 1,
    };

    // initial equipment: a dagger
    let mut dagger = Object::new(0, 0, "dagger", false, '-', colors::SKY);
    dagger.item = Some(Item::Sword);
    dagger.equipment = Some(Equipment {
        equipped: true,
        slot: Slot::LeftHand,
        max_hp_bonus: 0,
        defense_bonus: 0,
        power_bonus: 2,
    });
    game_state.inventory.push(dagger);

    initialize_fov(&game_state.world, tcod);

    // a warm welcoming message
    game_state.log.add(
        "Welcome stranger! prepare to perish in the Tombs of the Ancient Kings.",
        colors::RED,
    );

    (objects, game_state)
}

pub fn initialize_fov(world: &World, tcod: &mut Tcod) {
    // init fov map
    for y in 0..WORLD_HEIGHT {
        for x in 0..WORLD_WIDTH {
            tcod.fov.set(
                x,
                y,
                !world[x as usize][y as usize].block_sight,
                !world[x as usize][y as usize].blocked,
            );
        }
    }
    tcod.con.clear(); // unexplored areas start black (which is the default background color)
}

fn game_loop(objects: &mut Vec<Object>, game_state: &mut GameState, tcod: &mut Tcod) {
    // force FOV "recompute" first time through the game loop
    let mut previous_player_position = (-1, -1);

    // input processing
    let mut key: Key = Default::default();

    while !tcod.root.window_closed() {
        // clear the screen of the previous frame
        tcod.con.clear();

        // check for input events
        match input::check_for_event(input::MOUSE | input::KEY_PRESS) {
            Some((_, Event::Mouse(m))) => tcod.mouse = m,
            Some((_, Event::Key(k))) => key = k,
            _ => key = Default::default(),
        }

        // render objects and map
        let fov_recompute = previous_player_position != (objects[PLAYER].x, objects[PLAYER].y);
        render_all(tcod, game_state, &objects, fov_recompute);

        // draw everything on the window at once
        tcod.root.flush();

        // level up if needed
        level_up(objects, game_state, tcod);

        // handle keys and exit game if needed
        previous_player_position = objects[PLAYER].pos();
        let player_action = handle_keys(tcod, game_state, objects, key);
        if player_action == PlayerAction::Exit {
            save_game(objects, game_state).unwrap();
            break;
        }

        // let monsters take their turn
        if objects[PLAYER].alive && player_action != PlayerAction::DidntTakeTurn {
            for id in 0..objects.len() {
                if objects[id].ai.is_some() {
                    ai_take_turn(game_state, objects, &tcod.fov, id);
                }
            }
        }
    }
}

pub fn main_menu(tcod: &mut Tcod) {
    let img = tcod::image::Image::from_file("menu_background.png")
        .ok()
        .expect("Background image not found");

    while !tcod.root.window_closed() {
        // show the background image, at twice the regular console resolution
        tcod::image::blit_2x(&img, (0, 0), (-1, -1), &mut tcod.root, (0, 0));

        tcod.root.set_default_foreground(colors::LIGHT_YELLOW);
        tcod.root.print_ex(
            SCREEN_WIDTH / 2,
            SCREEN_HEIGHT / 2 - 4,
            BackgroundFlag::None,
            TextAlignment::Center,
            "TOMBS OF THE ANCIENT KINGS",
        );
        tcod.root.print_ex(
            SCREEN_WIDTH / 2,
            SCREEN_HEIGHT - 2,
            BackgroundFlag::None,
            TextAlignment::Center,
            "By Michael Wagner",
        );

        // show options and wait for the player's choice
        let choices = &["Play a new game", "Continue last game", "Quit"];
        let choice = menu("", choices, 24, &mut tcod.root);

        match choice {
            Some(0) => {
                // start new game
                let (mut objects, mut game_state) = new_game(tcod);
                game_loop(&mut objects, &mut game_state, tcod);
            }
            Some(1) => {
                // load game from file
                match load_game() {
                    Ok((mut objects, mut game_state)) => {
                        initialize_fov(&game_state.world, tcod);
                        game_loop(&mut objects, &mut game_state, tcod);
                    }
                    Err(_e) => {
                        msgbox("\nNo saved game to load\n", 24, &mut tcod.root);
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

fn save_game(objects: &[Object], game_state: &GameState) -> Result<(), Box<Error>> {
    let save_data = serde_json::to_string(&(objects, game_state))?;
    let mut file = File::create("savegame")?;
    file.write_all(save_data.as_bytes())?;
    Ok(())
}

fn load_game() -> Result<(Vec<Object>, GameState), Box<Error>> {
    let mut json_save_state = String::new();
    let mut file = File::open("savegame")?;
    file.read_to_string(&mut json_save_state)?;
    let result = serde_json::from_str::<(Vec<Object>, GameState)>(&json_save_state)?;
    Ok(result)
}
