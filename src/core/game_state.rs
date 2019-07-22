/// Module Game_State
///
/// This module contains the struct that encompasses all parts of the game state:
// external imports
use tcod::colors::Color;

// internal imports
use core::game_objects::GameObjects;
use core::world::world_gen::WorldGen;
use core::world::world_gen_rogue::RogueWorldGenerator;
use entity::action::*;
use entity::object::Object;
// use ui::dialog::menu;
use ui::game_frontend::{AnimationType, FovMap};
// use ui::game_input::GameInput;

// player object reference, index of the object vector
pub const PLAYER: usize = 0;
// TODO: Replace with something like object -> perception -> range.
pub const TORCH_RADIUS: i32 = 10;
// experience and level-ups
pub const LEVEL_UP_BASE: i32 = 200;
pub const LEVEL_UP_FACTOR: i32 = 150;
// pub const LEVEL_SCREEN_WIDTH: i32 = 40;

// Structures and functions for message output

/// Messages are expressed as colored text.
pub type Messages = Vec<(String, Color)>;

pub trait MessageLog {
    fn add<T: Into<String>>(&mut self, message: T, color: Color);
}

impl MessageLog for Vec<(String, Color)> {
    fn add<T: Into<String>>(&mut self, message: T, color: Color) {
        self.push((message.into(), color));
    }
}

/// Results from porcessing an objects action for that turn, in ascending rank.
#[derive(PartialEq, Debug)]
pub enum ObjectProcResult {
    NoAction,
    NoFeedback,
    CheckEnterFOV,
    Animate { anim_type: AnimationType },
    UpdateFOV,
    ReRender,
}

/// The game state struct contains all information necessary to represent
/// the current state of the game, EXCEPT the object vector.
#[derive(Serialize, Deserialize)]
pub struct GameState {
    pub log: Messages,
    pub dungeon_level: u32,
    current_obj_index: usize,
}

impl GameState {
    pub fn new(game_objects: &mut GameObjects, level: u32) -> Self {
        let mut world_generator = RogueWorldGenerator::new();
        world_generator.make_world(game_objects, level);

        GameState {
            // create the list of game messages and their colors, starts empty
            log: vec![],
            dungeon_level: 1,
            current_obj_index: 0,
        }
    }

    // TODO: Implement energy costs for actions.
    pub fn process_object(
        &mut self,
        objects: &mut GameObjects,
        fov_map: &FovMap,
    ) -> ObjectProcResult {
        let mut process_result = ObjectProcResult::NoAction;
        // unpack object to process its next action
        if let Some(mut active_object) = objects.extract(self.current_obj_index) {
            if let Some(next_action) = active_object.get_next_action() {
                // perform action
                process_result =
                    self.process_action(fov_map, objects, &mut active_object, next_action);
            }
            // return object back to objects vector
            objects[self.current_obj_index].replace(active_object);
        }

        // only increase counter if the object has made a move
        if process_result != ObjectProcResult::NoAction {
            self.current_obj_index = (self.current_obj_index + 1) % objects.get_num_objects();
        }
        process_result
    }

    /// Process an action of a given object.
    fn process_action(
        &mut self,
        fov_map: &FovMap,
        objects: &mut GameObjects,
        actor: &mut Object,
        action: Box<Action>,
    ) -> ObjectProcResult {
        // first execute action

        // then process result and return
        match action.perform(self, objects, actor) {
            ActionResult::Success { callback } => {
                match callback {
                    ObjectProcResult::CheckEnterFOV => {
                        if self.current_obj_index == PLAYER {
                            // if we have the player, then it will surely be in it's own fov
                            ObjectProcResult::UpdateFOV
                        } else if fov_map.is_in_fov(actor.x, actor.y) {
                            // if the acting object is inside the FOV now, trigger a re-render
                            ObjectProcResult::ReRender
                        } else {
                            ObjectProcResult::NoFeedback
                        }
                    }
                    // only play animations if the object is visible to our hero
                    ObjectProcResult::Animate { anim_type } => {
                        if fov_map.is_in_fov(actor.x, actor.y) {
                            ObjectProcResult::Animate { anim_type }
                        } else {
                            ObjectProcResult::NoFeedback
                        }
                    }
                    _ => callback,
                }
            }
            ActionResult::Failure => {
                // how to handle fails?
                ObjectProcResult::NoAction
            }
            ActionResult::Consequence { action } => {
                // if we have a side effect, process it first and then the `main` action
                let _consequence_result =
                    self.process_action(fov_map, objects, actor, action.unwrap());
                // TODO: Think of a way to handle both results of action and consequence.
                // TODO: extract into function, recursively bubble up results and return the highest priority
                ObjectProcResult::ReRender // use highest priority for now as a dummy
            }
        }
    }
}

// NOTE: All functions below are hot candidates for a rewrite because they might not fit into the new command pattern system.

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

// pub fn level_up(
//     game_io: &mut GameFrontend,
//     game_state: &mut GameState,
//     objects: &mut GameObjects,
//     game_input: &mut Option<&mut GameInput>,
// ) {
//     if let Some(ref mut player) = objects[PLAYER] {
//         let level_up_xp = LEVEL_UP_BASE + player.level * LEVEL_UP_FACTOR;
//         // see if the player's experience is enough to level up
//         if player.fighter.as_ref().map_or(0, |f| f.xp) >= level_up_xp {
//             // exp is enough, lvl up
//             player.level += 1;
//             game_state.log.add(
//                 format!(
//                     "Your battle skills grow stringer! You reached level {}!",
//                     player.level
//                 ),
//                 colors::YELLOW,
//             );

//             let fighter = player.fighter.as_mut().unwrap();
//             let mut choice = None;
//             while choice.is_none() {
//                 // keep asking until a choice is made
//                 choice = menu(
//                     game_io,
//                     game_input,
//                     "Level up! Chose a stat to raise:\n",
//                     &[
//                         format!("Constitution (+20 HP, from {})", fighter.base_max_hp),
//                         format!("Strength (+1 attack, from {})", fighter.base_power),
//                         format!("Agility (+1 defense, from {})", fighter.base_defense),
//                     ],
//                     LEVEL_SCREEN_WIDTH,
//                 );
//             }
//             fighter.xp -= level_up_xp;
//             match choice.unwrap() {
//                 0 => {
//                     fighter.base_max_hp += 20;
//                     fighter.hp += 20;
//                 }
//                 1 => {
//                     fighter.base_power += 1;
//                 }
//                 2 => {
//                     fighter.base_defense += 1;
//                 }
//                 _ => unreachable!(),
//             }
//         }
//     }
// }

// /// Advance to the next level
// pub fn next_level(
//     game_io: &mut GameFrontend,
//     objects: &mut GameObjects,
//     game_state: &mut GameState,
// ) {
//     game_state.log.add(
//         "You take a moment to rest, and recover your strength.",
//         colors::VIOLET,
//     );
//     // let heal_hp = objects[PLAYER].max_hp(game_state) / 2;
//     // objects[PLAYER].heal(game_state, heal_hp);

//     game_state.log.add(
//         "After a rare moment of peace, you descend deeper into the heart of the dungeon...",
//         colors::RED,
//     );
//     game_state.dungeon_level += 1;
//     game_state.world = make_world(objects, game_state.dungeon_level);
//     initialize_fov(game_io, &game_state.world);
// }

// /// Move the object with given id to the given position.
// pub fn move_by(world: &World, objects: &mut GameObjects, id: usize, dx: i32, dy: i32) {
//     // move by the given amount
//     if let Some(ref mut object) = objects[id] {
//         let (x, y) = object.pos();
//             if !is_blocked(world, objects, x + dx, y + dy) {
//                 object.set_pos(x + dx, y + dy);
//             }
//     }
// }

// Move the object with given id towards a target.
// pub fn move_towards(
//     world: &World,
//     objects: &mut GameObjects,
//     id: usize,
//     target_x: i32,
//     target_y: i32,
// ) {
//     // vector from this object to the target, and distance
//     match objects[id] {
//         Some(obj) => {
//             let dx = target_x - obj.x;
//             let dy = target_y - obj.y;
//             let distance = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();

//             // normalize it to length 1 (preserving direction), then round it and
//             // convert to integer so the movement is restricted to the map grid
//             let dx = (dx as f32 / distance).round() as i32;
//             let dy = (dy as f32 / distance).round() as i32;
//             move_by(world, objects, id, dx, dy);
//         }
//         None => {

//         }
//     }
// }
