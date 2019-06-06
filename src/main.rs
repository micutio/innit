//! Module Main
//!
//! This module contains all structures and methods pertaining to the user interface.

extern crate rand;
extern crate serde;
extern crate tcod;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod ai;
mod gui;
mod item;
mod object;
mod world;

use gui::{
    initialize_fov, main_menu, MessageLog, Messages, Tcod, LIMIT_FPS, PANEL_HEIGHT, SCREEN_HEIGHT,
    SCREEN_WIDTH,
};
use object::Object;
use world::{is_blocked, make_world, World, WORLD_HEIGHT, WORLD_WIDTH};

use std::cmp;

use tcod::colors;
use tcod::console::*;
use tcod::map::Map as FovMap;

const TORCH_RADIUS: i32 = 10;
// player object
pub const PLAYER: usize = 0;

/// Mutably borrow two *separate* elements from the given slice.
/// Panics when the indexes are equal or out of bounds.
fn mut_two<T>(items: &mut [T], first_index: usize, second_index: usize) -> (&mut T, &mut T) {
    assert!(first_index != second_index);
    let split_at_index = cmp::max(first_index, second_index);
    let (first_slice, second_slice) = items.split_at_mut(split_at_index);
    if first_index < second_index {
        (&mut first_slice[first_index], &mut second_slice[0])
    } else {
        (&mut second_slice[0], &mut first_slice[second_index])
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameState {
    world: World,
    log: Messages,
    inventory: Vec<Object>,
    dungeon_level: u32,
}

// objects and player

#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit,
}

// combat related poperties and methods (monster, player, NPC)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fighter {
    hp: i32,
    base_max_hp: i32,
    base_defense: i32,
    base_power: i32,
    on_death: DeathCallback,
    xp: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
enum DeathCallback {
    Player,
    Monster,
}

impl DeathCallback {
    fn callback(self, object: &mut Object, messages: &mut Messages) {
        use DeathCallback::*;
        let callback: fn(&mut Object, &mut Messages) = match self {
            Player => player_death,
            Monster => monster_death,
        };
        callback(object, messages);
    }
}

fn player_death(player: &mut Object, messages: &mut Messages) {
    // the game ended!
    messages.add("You died!", colors::RED);

    // for added effect, transform the player into a corpse
    player.chr = '%';
    player.color = colors::DARK_RED;
}

fn monster_death(monster: &mut Object, messages: &mut Messages) {
    messages.add(
        format!(
            "{} is dead! You gain {} XP",
            monster.name,
            monster.fighter.unwrap().xp
        ),
        colors::ORANGE,
    );
    monster.chr = '%';
    monster.color = colors::DARK_RED;
    monster.blocks = false;
    monster.fighter = None;
    monster.ai = None;
    monster.name = format!("remains of {}", monster.name);
}

fn move_by(world: &World, objects: &mut [Object], id: usize, dx: i32, dy: i32) {
    // move by the given amount
    let (x, y) = objects[id].pos();
    if !is_blocked(world, objects, x + dx, y + dy) {
        objects[id].set_pos(x + dx, y + dy);
    }
}

fn player_move_or_attack(game_state: &mut GameState, objects: &mut [Object], dx: i32, dy: i32) {
    // the coordinate the player is moving to/attacking
    let x = objects[PLAYER].x + dx;
    let y = objects[PLAYER].y + dy;

    // try to find an attackable object there
    let target_id = objects
        .iter()
        .position(|object| object.fighter.is_some() && object.pos() == (x, y));

    // attack if target found, move otherwise
    match target_id {
        Some(target_id) => {
            let (player, target) = mut_two(objects, PLAYER, target_id);
            player.attack(target, game_state);
        }
        None => {
            move_by(&game_state.world, objects, PLAYER, dx, dy);
        }
    }
}

fn move_towards(world: &World, objects: &mut [Object], id: usize, target_x: i32, target_y: i32) {
    // vector from this object to the target, and distance
    let dx = target_x - objects[id].x;
    let dy = target_y - objects[id].y;
    let distance = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();

    // normalize it to length 1 (preserving direction), then round it and
    // convert to integer so the movement is restricted to the map grid
    let dx = (dx as f32 / distance).round() as i32;
    let dy = (dy as f32 / distance).round() as i32;
    move_by(world, objects, id, dx, dy);
}

/// Advance to the next level
fn next_level(tcod: &mut Tcod, objects: &mut Vec<Object>, game_state: &mut GameState) {
    game_state.log.add(
        "You take a moment to rest, and recover your strength.",
        colors::VIOLET,
    );
    let heal_hp = objects[PLAYER].max_hp(game_state) / 2;
    objects[PLAYER].heal(game_state, heal_hp);

    game_state.log.add(
        "After a rare moment of peace, you descend deeper into the heart of the dungeon...",
        colors::RED,
    );
    game_state.dungeon_level += 1;
    game_state.world = make_world(objects, game_state.dungeon_level);
    initialize_fov(&game_state.world, tcod);
}

pub struct Transition {
    level: u32,
    value: u32,
}

/// Return a value that depends on level.
/// The table specifies what value occurs after each level, default is 0.
pub fn from_dungeon_level(table: &[Transition], level: u32) -> u32 {
    table
        .iter()
        .rev()
        .find(|transition| level >= transition.level)
        .map_or(0, |transition| transition.value)
}

fn main() {
    let root = Root::initializer()
        .font("assets/terminal16x16_gs_ro.png", FontLayout::AsciiInRow)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("roguelike")
        .init();

    tcod::system::set_fps(LIMIT_FPS);

    let mut tcod = Tcod {
        root: root,
        con: Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT),
        panel: Offscreen::new(SCREEN_WIDTH, PANEL_HEIGHT),
        fov: FovMap::new(WORLD_WIDTH, WORLD_HEIGHT),
        mouse: Default::default(),
    };

    main_menu(&mut tcod);
}
