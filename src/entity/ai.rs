//! Module Ai
//!
//! Structures and methods for constructing the game ai.
// external imports
// use rand::Rng;
// use tcod::colors;

// internal imports
// use entity::object::ObjectVec;
// use game_state::{GameState, MessageLog};
// use ui::game_frontend::FovMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Ai {
    Basic,
    Confused {
        previous_ai: Box<Ai>,
        num_turns: i32,
    },
}

// /// Main NPC control function
// /// TODO:
// ///  - sense
// ///  - reason
// ///  - actuate
// ///     |- either move
// ///     |- or attack
// pub fn ai_take_turn(
//     game_state: &mut GameState,
//     objects: &mut ObjectVec,
//     fov_map: &FovMap,
//     monster_id: usize,
// ) {
//     use self::Ai::*;
//     // match objects[monster_id] {
//     //     Some(obj) => {
//     //         if let Some(ai) = obj.ai.take() {
//     //             let new_ai = match ai {
//     //                 Basic => ai_basic(game_state, objects, fov_map, monster_id),
//     //                 Confused {
//     //                     previous_ai,
//     //                     num_turns,
//     //                 } => ai_confused(game_state, objects, monster_id, previous_ai, num_turns),
//     //             };
//     //             obj.ai = Some(new_ai);
//     //         }
//     //     }
//     //     None => {}
//     // }
// }

// fn ai_basic(
//     game_state: &mut GameState,
//     objects: &mut ObjectVec,
//     fov_map: &FovMap,
//     monster_id: usize,
// ) -> Ai {
//     // A basic monster takes its turn. If you can see it, it can see you.
//     // if let Some(ref mut monster) = objects[monster_id] {
//     //     let (monster_x, monster_y) = obj.pos();
//     //     if fov_map.is_in_fov(monster_x, monster_y) {
//     //         if obj.distance_to(&objects[PLAYER].unwrap()) >= 2.0 {
//     //             // move towards player if far away
//     //             let (player_x, player_y) = objects[PLAYER].unwrap().pos();
//     //             // TODO: Re-implement AI decision making!
//     //             // move_towards(&game_state.world, objects, monster_id, player_x, player_y);
//     //         } else if objects[PLAYER].unwrap().fighter.map_or(false, |f| f.hp > 0) {
//     //             // Close enough, attack! (if player is still alive)
//     //             // FIXME: This ain't working no more.
//     //             // let (monster, player) = mut_two(objects, monster_id, PLAYER);
//     //             // monster.attack(player, game_state);
//     //         }
//     //     }
//     Ai::Basic
//     // }
// }

// fn ai_confused(
//     game_state: &mut GameState,
//     objects: &mut ObjectVec,
//     monster_id: usize,
//     previous_ai: Box<Ai>,
//     num_turns: i32,
// ) -> Ai {
//     if num_turns >= 0 {
//         // still confused...
//         // move in a random direction, and decrease the number of tuns confused
//         // TODO: Re-implement AI decision making.
//         // move_by(
//         //     &game_state.world,
//         //     objects,
//         //     monster_id,
//         //     rand::thread_rng().gen_range(-1, 2),
//         //     rand::thread_rng().gen_range(-1, 2),
//         // );
//         Ai::Confused {
//             previous_ai,
//             num_turns: num_turns - 1,
//         }
//     } else {
//         // restore the previous AI (this one will be deleted)
//         if let Some(ref monster) = objects[monster_id] {
//             game_state.log.add(
//                 format!("The {} is no longer confused!", monster.name),
//                 colors::RED,
//             );
//         }
//         *previous_ai
//     }
// }
