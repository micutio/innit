/// Module Game Input
///
/// User input processing
/// Handle user input

// internal imports
use entity::action::*;
use entity::object::{ObjectVec};
use ui::game_frontend::FovMap;

// external imports
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::{Mutex, Arc};
use std::thread::{self, JoinHandle};
use tcod::input::{self, Event, Key, Mouse};

/// As tcod's key codes don't implement Eq and Hash they cannot be used
/// as keys in a hash table. So to still be able to hash keys, we define our own.
#[derive(PartialEq, Eq, Hash)]
pub enum KeyCode {
    A, B, C, D, E, F, G, H,
    I, J, K, L, M, N, O, P,
    Q, R, S, T, U, V, W, X,
    Y, Z, Undefined,
    Up, Down, Left, Right,
    Esc, F1, F2, F3, F4,
}

pub enum PlayerAction {
    MetaAction(UiAction),
    Undefined,
    Pending,
    DoNothing,
    WalkNorth,
    WalkSouth,
    WalkEast,
    WalkWest,
}

pub enum UiAction {
    ExitGameLoop,
    Fullscreen,
    CharacterScreen,
}

pub struct GameInput {
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub next_player_actions: VecDeque<PlayerAction>,
}

impl GameInput {
    pub fn new() -> Self {
        GameInput {
            mouse_x: 0,
            mouse_y: 0,
            next_player_actions: VecDeque::new(),
        }
    }
}

fn get_names_under_mouse(objects: &ObjectVec, fov_map: &FovMap, mouse_x: i32, mouse_y: i32) -> String {
    // create a list with the names of all objects at the mouse's coordinates and in FOV
    let names = objects
        .get_vector()
        .iter()
        .filter(|Some(obj)| obj.pos() == (mouse_x, mouse_y) && fov_map.is_in_fov(obj.x, obj.y))
        .map(|Some(obj)| obj.name.clone())
        .collect::<Vec<_>>();

    names.join(", ") // return names separated by commas
}

pub fn start_input_proc_thread(game_input: &mut Arc<Mutex<GameInput>>) -> JoinHandle<()> {
    let game_input_buf = Arc::clone(&game_input);
    let key_to_action_mapping = create_key_mapping();

    thread::spawn(move|| {
        loop {
            let mouse_x: i32 = 0;
            let mouse_y: i32 = 0;
            let _mouse: Mouse = Default::default(); // this is not really used right now
            let _key: Key = Default::default();
            match input::check_for_event(input::MOUSE | input::KEY_PRESS) {
                // record mouse position for later use
                Some((_, Event::Mouse(_m))) => {
                    mouse_x = _m.cx as i32;
                    mouse_y = _m.cy as i32;
                }
                // get used key to create next user action
                Some((_, Event::Key(k))) => {
                    _key = k;
                }
                _ => {
                    mouse_x = Default::default();
                    mouse_y = Default::default();
                }
            }

            // lock our mutex and get to work
            let mut input = game_input_buf.lock().unwrap();
            let player_action = match key_to_action_mapping.get(&tcod_to_key_code(_key)) {
                Some(key) => key,
                None => &PlayerAction::Undefined,
            };
            input.next_player_actions.push_back(*player_action);
            input.mouse_x = mouse_x;
            input.mouse_y = mouse_y;
        }
    })
}

/// Translate between tcod's keys and our own key codes.
fn tcod_to_key_code(tcod_key: tcod::input::Key) -> self::KeyCode {
    match tcod_key {
        // in-game actinos
        Key { code: Up, .. } => self::KeyCode::Up,
        Key { code: Down, .. } => self::KeyCode::Down,
        Key { code: Right, .. } => self::KeyCode::Right,
        Key { code: Left, .. } => self::KeyCode::Left,
        // non-in-game actions
        Key { code: Escape, .. } => self::KeyCode::Esc,
        Key { code: F4, ..} => self::KeyCode::F4,
        _ => self::KeyCode::Undefined,
    }
}

pub fn create_key_mapping() -> HashMap<KeyCode, PlayerAction> {
    use self::KeyCode::*;
    use self::PlayerAction::*;
    use self::UiAction::*;

    let key_map: HashMap<KeyCode, PlayerAction> = HashMap::new();

    // TODO: Fill mapping from json file.
    // set up all in-game actions
    key_map.insert(Up, WalkNorth);
    key_map.insert(Down, WalkSouth);
    key_map.insert(Left, WalkWest);
    key_map.insert(Right, WalkEast);
    // set up all non-in-game actions.
    key_map.insert(Esc, MetaAction(ExitGameLoop));
    key_map.insert(F4, MetaAction(Fullscreen));

    key_map
}

pub fn get_player_action_instance(player_action: PlayerAction) -> Box<dyn Action> {
    use entity::action::Direction::*;
    
    // TODO: Use actual costs.
    // No need to map `Esc` since we filter out exiting before instantiating
    // any player actions.
    match player_action {
        Up => Box::new(MoveAction::new(North, 0)),
        Down => Box::new(MoveAction::new(South, 0)),
        Right => Box::new(MoveAction::new(East, 0)),
        Left => Box::new(MoveAction::new(West, 0)),
    }
}


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
