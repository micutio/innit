/// Module Game_State
///
/// This module contains the struct that encompasses all parts of the game state:
///
/// TODO: Try to move as many dependecies to game_io as possible out of here.

// external imports
use tcod::colors;

// internal imports
use entity::action::*;
use entity::fighter::{DeathCallback, Fighter};
use entity::object::{Object, ObjectVec};
use ui::game_frontend::{initialize_fov, menu, GameFrontend, MessageLog, Messages, FovMap, AnimationType};
use util::mut_two;
use world::{is_blocked, make_world, World};

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

pub fn new_game() -> (GameEngine, GameState, ObjectVec) {
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
    player.attack_action = Some(AttackAction::new(2, 0));

    // create array holding all objectVec
    let mut objects = ObjectVec::new();
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

    // a warm welcoming message
    game_state.log.add(
        "Welcome microbe! You're innit now. Beware of bacteria and viruses",
        colors::RED,
    );

    let mut game_engine = GameEngine::new();

    (game_engine, game_state, objects)
}

#[derive(Serialize, Deserialize)]
pub struct GameEngine {
    current_obj_index: usize
}

pub enum ProcessResult {
    Nil,
    UpdateFOV,
    UpdateRender,
    Animate {
        // TODO: Create animation type enum.
        anim_type: AnimationType,
    }
}

impl GameEngine {
    pub fn new() -> Self {
        GameEngine {
            current_obj_index: 0
        }
    }

    // TODO: Implement energy costs for actions.
    pub fn process_object(&mut self, fov_map: &FovMap, game_state: &mut GameState, objects: &mut ObjectVec) -> ProcessResult {
        if let Some((active_index, active_object)) = objects.extract(self.current_obj_index) {
            let action_option = active_object.get_next_action();
            let process_result = process_action(&mut active_object, fov_map, game_state, objects, action_option);
            return process_result;
        }
        ProcessResult::Nil
    }
}

/// Process an action of a given object,
/// TODO: Use fov_map to check whether something moved within the player's FOV.
fn process_action(actor: &mut Object, fov_map: &FovMap, game_state: &mut GameState, objects: &mut ObjectVec, action_option: &Option<Box<Action>>) -> ProcessResult {
    // first execute action
    let action_result = match action_option {
        Some(action) => {
            action.perform(&mut actor, objects, game_state)
        }
        None => {
            ActionResult::Failure
        }
    };

    // then process result and return
    match action_result {
        ActionResult::Success => {
            // what do?
            // check if we need to play an animation, update fov or render stuff
            ProcessResult::Nil
        }
        ActionResult::Failure => {
            // how to handle fails?
            ProcessResult::Nil
        }
        ActionResult::Consequence{ action } => {
            // if we have a side effect, process it first and then the `main` action
            let consequence_result = process_action(actor, fov_map, game_state, objects, &action);
            // TODO: Think of a way to handle both results of action and consequence.
            ProcessResult::Nil
        }
    }
}

pub fn move_by(world: &World, objects: &mut ObjectVec, id: usize, dx: i32, dy: i32) {
    // move by the given amount
    match objects[id] {
        Some(obj) => {
            let (x, y) = obj.pos();
            if !is_blocked(world, objects, x + dx, y + dy) {
                obj.set_pos(x + dx, y + dy);
            }
        }
    }
    
}

pub fn move_towards(
    world: &World,
    objects: &mut ObjectVec,
    id: usize,
    target_x: i32,
    target_y: i32,
) {
    // vector from this object to the target, and distance
    match objects[id] {
        Some(obj) => {
            let dx = target_x - obj.x;
            let dy = target_y - obj.y;
            let distance = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();

            // normalize it to length 1 (preserving direction), then round it and
            // convert to integer so the movement is restricted to the map grid
            let dx = (dx as f32 / distance).round() as i32;
            let dy = (dy as f32 / distance).round() as i32;
            move_by(world, objects, id, dx, dy);
        }
        None => {

        }
    }
    
}

/// Advance to the next level
pub fn next_level(
    game_io: &mut GameFrontend,
    objects: &mut ObjectVec,
    game_state: &mut GameState,
) {
    game_state.log.add(
        "You take a moment to rest, and recover your strength.",
        colors::VIOLET,
    );
    // let heal_hp = objects[PLAYER].max_hp(game_state) / 2;
    // objects[PLAYER].heal(game_state, heal_hp);

    game_state.log.add(
        "After a rare moment of peace, you descend deeper into the heart of the dungeon...",
        colors::RED,
    );
    game_state.dungeon_level += 1;
    game_state.world = make_world(objects, game_state.dungeon_level);
    initialize_fov(game_io, &game_state.world);
}

pub struct Transition {
    pub level: u32,
    pub value: u32,
}

/// Return a value that depends on dnugeonlevel.
/// The table specifies what value occurs after each level, default is 0.
pub fn from_dungeon_level(table: &[Transition], level: u32) -> u32 {
    table
        .iter()
        .rev()
        .find(|transition| level >= transition.level)
        .map_or(0, |transition| transition.value)
}

pub fn level_up(objects: &mut ObjectVec, game_state: &mut GameState, game_io: &mut GameFrontend) {
    let player = &mut objects[PLAYER].unwrap();
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
