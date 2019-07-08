/// Module Game Input
///
/// User input processing
/// Handle user input

// internal imports
use entity::action::*;
use entity::object::{ObjectVec};
use game_state::{
    next_level, player_move_or_attack, GameState, LEVEL_UP_BASE, LEVEL_UP_FACTOR, PLAYER,
};
use ui::game_frontend::{msgbox, FovMap, GameFrontend, CHARACTER_SCREEN_WIDTH};

// external imports
use std::collections::HashMap;
use std::sync::{Mutex, Arc};
use std::thread::{self, JoinHandle};
use tcod::input::{self, Event, Key, Mouse};

#[derive(PartialEq, Eq, Hash)]
pub enum KeyCode {
    A, B, C, D, E, F, G, H,
    I, J, K, L, M, N, O, P,
    Q, R, S, T, U, V, W, X,
    Y, Z,
    Up, Down, Left, Right,
}

pub enum PlayerAction {
    Pending,
    DoNothing,
    WalkNorth,
    WalkSouth,
    WalkEast,
    WalkWest,
}

pub struct GameInput {
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub next_player_action: Option<PlayerAction>,
}

impl GameInput {
    pub fn new() -> Self {
        GameInput {
            mouse_x: 0,
            mouse_y: 0,
            next_player_action: None,
        }
    }
}

fn get_names_under_mouse(object_vec: &ObjectVec, fov_map: &FovMap, mouse_x: i32, mouse_y: i32) -> String {
    // create a list with the names of all objects at the mouse's coordinates and in FOV
    let names = object_vec
        .get_vector()
        .iter()
        .filter(|Some(obj)| obj.pos() == (mouse_x, mouse_y) && fov_map.is_in_fov(obj.x, obj.y))
        .map(|Some(obj)| obj.name.clone())
        .collect::<Vec<_>>();

    names.join(", ") // return names separated by commas
}

pub fn start_input_proc_thread(input_buffer: &mut Arc<Mutex<GameInput>>) -> JoinHandle<()> {
    let input_buf = Arc::clone(&input_buffer);

    thread::spawn(move|| {
        // lock our mutex and get to work
        let mut input = input_buf.lock().unwrap();

        let _mouse: Mouse = Default::default();
        let _key: Key = Default::default();
        match input::check_for_event(input::MOUSE | input::KEY_PRESS) {
            Some((_, Event::Mouse(_m))) => {
                // record mouse position for later use
                input.mouse_x = _m.cx as i32;
                input.mouse_y = _m.cy as i32;
            }

            Some((_, Event::Key(k))) => _key = k,
            _ => _key = Default::default(),
        }

    })
}

pub fn create_key_mapping() -> HashMap<KeyCode, PlayerAction> {
    use self::KeyCode::*;
    use self::PlayerAction::*;

    let key_map: HashMap<KeyCode, PlayerAction> = HashMap::new();

    // TODO: Fill mapping from json file.
    key_map.insert(Up, WalkNorth);
    key_map.insert(Down, WalkSouth);
    key_map.insert(Left, WalkWest);
    key_map.insert(Right, WalkEast);

    key_map
}

// #[derive(Clone, Copy, Debug, PartialEq)]
// pub enum PlayerAction {
//     TookTurn,
//     DidntTakeTurn,
//     Exit,
// }

// pub fn handle_keys(
//     game_io: &mut GameFrontend,
//     game_input: &mut GameInput,
//     game_state: &mut GameState,
//     objects: &mut Vec<Object>,
// ) -> PlayerAction {
//     use self::PlayerAction::*;
//     use tcod::input::KeyCode::*;

//     let player_alive = objects[PLAYER].alive;
//     match (game_input.key, player_alive) {
//         // toggle fullscreen
//         (
//             Key {
//                 code: Enter,
//                 alt: true,
//                 ..
//             },
//             _,
//         ) => {
//             let fullscreen = game_io.root.is_fullscreen();
//             game_io.root.set_fullscreen(!fullscreen);
//             DidntTakeTurn
//         }

//         // exit game
//         (Key { code: Escape, .. }, _) => Exit,

//         // handle movement
//         (Key { code: Up, .. }, true) => {
//             player_move_or_attack(game_state, objects, 0, -1);
//             TookTurn
//         }
//         (Key { code: Down, .. }, true) => {
//             player_move_or_attack(game_state, objects, 0, 1);
//             TookTurn
//         }
//         (Key { code: Left, .. }, true) => {
//             player_move_or_attack(game_state, objects, -1, 0);
//             TookTurn
//         }
//         (Key { code: Right, .. }, true) => {
//             player_move_or_attack(game_state, objects, 1, 0);
//             TookTurn
//         }
//         (Key { printable: 'x', .. }, true) => {
//             // do nothing, i.e. wait for the monster to come to you
//             TookTurn
//         }
//         (Key { printable: 'e', .. }, true) => {
//             // go down the stairs, if the player is on them
//             println!("trying to go down stairs");
//             let player_on_stairs = objects
//                 .iter()
//                 .any(|object| object.pos() == objects[PLAYER].pos() && object.name == "stairs");
//             if player_on_stairs {
//                 next_level(game_io, objects, game_state);
//             }
//             DidntTakeTurn
//         }
//         (Key { printable: 'c', .. }, true) => {
//             // show character information
//             let player = &objects[PLAYER];
//             let level = player.level;
//             let level_up_xp = LEVEL_UP_BASE + player.level * LEVEL_UP_FACTOR;
//             if let Some(fighter) = player.fighter.as_ref() {
//                 let msg = format!(
//                     "Character information

//                 Level: {}
//                 Experience: {}
//                 Experience to level up: {}

//                 Maximum HP: {}
//                 Attack: {}
//                 Defense: {}",
//                     level,
//                     fighter.xp,
//                     level_up_xp,
//                     player.max_hp(game_state),
//                     player.power(game_state),
//                     player.defense(game_state),
//                 );
//                 msgbox(&msg, CHARACTER_SCREEN_WIDTH, &mut game_io.root);
//             }

//             DidntTakeTurn
//         }

//         _ => DidntTakeTurn,
//     }
// }



// /// return the position of a tile left-clicked in player's FOV (optionally in a range),
// /// or (None, None) if right-clicked.
// pub fn target_tile(
//     game_io: &mut GameFrontend,
//     game_state: &mut GameState,
//     objects: &[Object],
//     max_range: Option<f32>,
// ) -> Option<(i32, i32)> {
//     use tcod::input::KeyCode::Escape;
//     loop {
//         // render the screen. this erases the inventory and shows the names of objects under the mouse
//         game_io.root.flush();
//         let event = input::check_for_event(input::KEY_PRESS | input::MOUSE).map(|e| e.1);
//         let mut key = None;
//         match event {
//             Some(Event::Mouse(m)) => game_io.mouse = m,
//             Some(Event::Key(k)) => key = Some(k),
//             None => {}
//         }
//         render_all(game_io, game_state, objects, false);

//         let (x, y) = (game_io.mouse.cx as i32, game_io.mouse.cy as i32);

//         // accept the target if the player clicked in FOV, and in case a range is specified, if it's in that range
//         let in_fov = (x < WORLD_WIDTH) && (y < WORLD_HEIGHT) && game_io.fov.is_in_fov(x, y);
//         let in_range = max_range.map_or(true, |range| objects[PLAYER].distance(x, y) <= range);

//         if game_io.mouse.lbutton_pressed && in_fov && in_range {
//             return Some((x, y));
//         }

//         let escape = key.map_or(false, |k| k.code == Escape);
//         if game_io.mouse.rbutton_pressed || escape {
//             return None; // cancel if the player right-clicked or pressed Escape
//         }
//     }
// }

// pub fn target_monster(
//     game_io: &mut GameFrontend,
//     game_state: &mut GameState,
//     objects: &[Object],
//     max_range: Option<f32>,
// ) -> Option<usize> {
//     loop {
//         match target_tile(game_io, game_state, objects, max_range) {
//             Some((x, y)) => {
//                 // return the first clicked monster, otherwise continue looping
//                 for (id, obj) in objects.iter().enumerate() {
//                     if obj.pos() == (x, y) && obj.fighter.is_some() && id != PLAYER {
//                         return Some(id);
//                     }
//                 }
//             }
//             None => return None,
//         }
//     }
// }
