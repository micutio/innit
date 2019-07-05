/// Module Game_State
///
/// This module contains the struct that encompasses all parts of the game state:
///
/// TODO: Try to move as many dependecies to game_io as possible out of here.
// internal modules
use entity::action::*;
use entity::fighter::{DeathCallback, Fighter};
use entity::object::{Object, ObjectVec};
use ui::game_frontend::{initialize_fov, menu, GameFrontend, MessageLog, Messages};
use util::mut_two;
use world::{is_blocked, make_world, World};

use tcod::colors;

// TODO: reorganize objectVec vector
//      - first n = WORLD_WIDTH*WORLD_HEIGHT objects are world tile objectVec
//      - n+1 object is PLAYER
//      - then everything else
// player object reference, index of the object vector
pub const PLAYER: usize = 0;
pub const TORCH_RADIUS: i32 = 10;
// experience and level-ups
pub const LEVEL_UP_BASE: i32 = 200;
pub const LEVEL_UP_FACTOR: i32 = 150;
pub const LEVEL_SCREEN_WIDTH: i32 = 40;

#[derive(Serialize, Deserialize)]
pub struct GameState {
    pub world: World,
    pub log: Messages,
    pub inventory: Vec<Object>,
    pub dungeon_level: u32,
}

pub fn new_game() -> (ObjectVec, GameState, GameEngine) {
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
    player.attack_action = Some(AttackAction::new(2));

    // create array holding all objectVec
    let mut object_vec = ObjectVec::new();
    let level = 1;

    // create game state holding most game-relevant information
    //  - also creates map and player starting position
    let mut game_state = GameState {
        // generate map (at this point it's not drawn on screen)
        world: make_world(&mut object_vec, level),
        // create the list of game messages and their colors, starts empty
        log: vec![],
        inventory: vec![],
        dungeon_level: 1,
    };

    // a warm welcoming message
    game_state.log.add(
        "Welcome microbe! You're innit now. Beware of bacteria and viruses",
        colors::RED,
    );

    let mut game_engine = GameEngine::new();

    (object_vec, game_state, game_engine)
}

#[derive(Serialize, Deserialize)]
pub struct GameEngine {
    current_obj_index: usize
}

pub enum ProcessResult {
    Nil,
    UpdateVisibility,
    Animate {
        x: u32,
        y: u32,
    }
}

impl GameEngine {
    pub fn new() -> Self {
        GameEngine {
            current_obj_index: 0
        }
    }

    pub fn process(&mut self, game_state: &mut GameState, object_vec: &mut ObjectVec) -> ProcessResult {
        if let Some((active_index, active_object)) = object_vec.extract(self.current_obj_index) {
            // execute objects current action
            dummy_mut_borrow(object_vec);
            // return result of action
            return ProcessResult::Nil;
        }
        ProcessResult::Nil
    }
}

fn dummy_mut_borrow(object_vec: &mut ObjectVec) {
    if let Some(object) = &mut object_vec[0] {
        object.set_pos(0, 0);
    }
}

pub fn move_by(world: &World, object_vec: &mut ObjectVec, id: usize, dx: i32, dy: i32) {
    // move by the given amount
    let (x, y) = object_vec[id].pos();
    if !is_blocked(world, object_vec, x + dx, y + dy) {
        object_vec[id].set_pos(x + dx, y + dy);
    }
}

pub fn player_move_or_attack(game_state: &mut GameState, object_vec: &mut ObjectVec, dx: i32, dy: i32) {
    // the coordinate the player is moving to/attacking
    let x = object_vec[PLAYER].x + dx;
    let y = object_vec[PLAYER].y + dy;

    // try to find an attackable object there
    let target_id = object_vec
        .iter()
        .position(|object| object.fighter.is_some() && object.pos() == (x, y));

    // attack if target found, move otherwise
    match target_id {
        Some(target_id) => {
            let (player, target) = mut_two(object_vec, PLAYER, target_id);
            player.attack(target, game_state);

            // TODO: Solve double mutable borrow of `object_vec` here!
            // match player.attack_action {
            //     Some(ref mut attack_action) => {
            //         // attack_action.acquire_target(target_id);
            //         attack_action.perform(object_vec, game_state);
            //     }
            //     None => {}
            // }
        }
        None => {
            move_by(&game_state.world, object_vec, PLAYER, dx, dy);
        }
    }
}

pub fn move_towards(
    world: &World,
    object_vec: &mut ObjectVec,
    id: usize,
    target_x: i32,
    target_y: i32,
) {
    // vector from this object to the target, and distance
    let dx = target_x - object_vec[id].x;
    let dy = target_y - object_vec[id].y;
    let distance = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();

    // normalize it to length 1 (preserving direction), then round it and
    // convert to integer so the movement is restricted to the map grid
    let dx = (dx as f32 / distance).round() as i32;
    let dy = (dy as f32 / distance).round() as i32;
    move_by(world, object_vec, id, dx, dy);
}

/// Advance to the next level
pub fn next_level(
    game_io: &mut GameFrontend,
    objectVec: &mut Vec<Object>,
    game_state: &mut GameState,
) {
    game_state.log.add(
        "You take a moment to rest, and recover your strength.",
        colors::VIOLET,
    );
    let heal_hp = objectVec[PLAYER].max_hp(game_state) / 2;
    objectVec[PLAYER].heal(game_state, heal_hp);

    game_state.log.add(
        "After a rare moment of peace, you descend deeper into the heart of the dungeon...",
        colors::RED,
    );
    game_state.dungeon_level += 1;
    game_state.world = make_world(objectVec, game_state.dungeon_level);
    initialize_fov(&game_state.world, game_io);
}

pub struct Transition {
    pub level: u32,
    pub value: u32,
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

pub fn level_up(object_vec: &mut ObjectVec, game_state: &mut GameState, game_io: &mut GameFrontend) {
    let player = &mut object_vec[PLAYER];
    let level_up_xp = LEVEL_UP_BASE + player.level * LEVEL_UP_FACTOR;
    // see if the player's experience is enough to level up
    if player.fighter.as_ref().map_or(0, |f| f.xp) >= level_up_xp {
        // exp is enough, lvl up
        player.level += 1;
        game_state.log.add(
            format!(
                "Your battle skills grow stringer! You reached level {}!",
                player.level
            ),
            colors::YELLOW,
        );
        // TODO: increase player's stats
        let fighter = player.fighter.as_mut().unwrap();
        let mut choice = None;
        while choice.is_none() {
            // keep asking until a choice is made
            choice = menu(
                "Level up! Chose a stat to raise:\n",
                &[
                    format!("Constitution (+20 HP, from {})", fighter.base_max_hp),
                    format!("Strength (+1 attack, from {})", fighter.base_power),
                    format!("Agility (+1 defense, from {})", fighter.base_defense),
                ],
                LEVEL_SCREEN_WIDTH,
                &mut game_io.root,
            );
        }
        fighter.xp -= level_up_xp;
        match choice.unwrap() {
            0 => {
                fighter.base_max_hp += 20;
                fighter.hp += 20;
            }
            1 => {
                fighter.base_power += 1;
            }
            2 => {
                fighter.base_defense += 1;
            }
            _ => unreachable!(),
        }
    }
}
