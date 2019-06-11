/// Module Ai
///
/// Structures and methods for constructing the game ai.
use rand::Rng;
use tcod::colors;

use game_state::{move_by, move_towards, GameState, PLAYER};
use game_io::{FovMap, MessageLog};
use object::Object;
use util::mut_two;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Ai {
    Basic,
    Confused {
        previous_ai: Box<Ai>,
        num_turns: i32,
    },
}

pub fn ai_take_turn(
    game_state: &mut GameState,
    objects: &mut [Object],
    fov_map: &FovMap,
    monster_id: usize,
) {
    use self::Ai::*;
    if let Some(ai) = objects[monster_id].ai.take() {
        let new_ai = match ai {
            Basic => ai_basic(game_state, objects, fov_map, monster_id),
            Confused {
                previous_ai,
                num_turns,
            } => ai_confused(game_state, objects, monster_id, previous_ai, num_turns),
        };
        objects[monster_id].ai = Some(new_ai);
    }
}

fn ai_basic(
    game_state: &mut GameState,
    objects: &mut [Object],
    fov_map: &FovMap,
    monster_id: usize,
) -> Ai {
    // A basic monster takes its turn. If you can see it, it can see you.
    let (monster_x, monster_y) = objects[monster_id].pos();
    if fov_map.is_in_fov(monster_x, monster_y) {
        if objects[monster_id].distance_to(&objects[PLAYER]) >= 2.0 {
            // move towards player if far away
            let (player_x, player_y) = objects[PLAYER].pos();
            move_towards(&game_state.world, objects, monster_id, player_x, player_y);
        } else if objects[PLAYER].fighter.map_or(false, |f| f.hp > 0) {
            // Close enough, attack! (if player is still alive)
            let (monster, player) = mut_two(objects, monster_id, PLAYER);
            monster.attack(player, game_state);
        }
    }
    Ai::Basic
}

fn ai_confused(
    game_state: &mut GameState,
    objects: &mut [Object],
    monster_id: usize,
    previous_ai: Box<Ai>,
    num_turns: i32,
) -> Ai {
    if num_turns >= 0 {
        // still confused...
        // move in a random direction, and decrease the number of tuns confused
        move_by(
            &game_state.world,
            objects,
            monster_id,
            rand::thread_rng().gen_range(-1, 2),
            rand::thread_rng().gen_range(-1, 2),
        );
        Ai::Confused {
            previous_ai: previous_ai,
            num_turns: num_turns - 1,
        }
    } else {
        // restore the previous AI (this one will be deleted)
        game_state.log.add(
            format!("The {} is no longer confused!", objects[monster_id].name),
            colors::RED,
        );
        *previous_ai
    }
}
