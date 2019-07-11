/// Module Game Input
///
/// User input processing
/// Handle user input
// internal imports
use entity::action::*;
use entity::object::ObjectVec;
use ui::game_frontend::FovMap;

// external imports
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use tcod::input::{self, Event, Key, Mouse};

/// As tcod's key codes don't implement Eq and Hash they cannot be used
/// as keys in a hash table. So to still be able to hash keys, we define our own.
#[derive(PartialEq, Eq, Hash)]
pub enum KeyCode {
    // A,
    // B,
    C,
    // D,
    // E,
    // F,
    // G,
    // H,
    // I,
    // J,
    // K,
    // L,
    // M,
    // N,
    // O,
    // P,
    // Q,
    // R,
    // S,
    // T,
    // U,
    // V,
    // W,
    // X,
    // Y,
    // Z,
    UndefinedKey,
    Up,
    Down,
    Left,
    Right,
    Esc,
    // F1,
    // F2,
    // F3,
    F4,
}

#[derive(Clone, Debug)]
pub enum PlayerAction {
    MetaAction(UiAction),
    // Pending,
    // DoNothing,
    WalkNorth,
    WalkSouth,
    WalkEast,
    WalkWest,
}

#[derive(Clone, Debug)]
pub enum UiAction {
    // UndefinedUi,
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

pub fn get_names_under_mouse(
    objects: &ObjectVec,
    fov_map: &FovMap,
    mouse_x: i32,
    mouse_y: i32,
) -> String {
    // create a list with the names of all objects at the mouse's coordinates and in FOV
    objects
        .get_vector()
        .iter()
        .flatten()
        .filter(|o| o.pos() == (mouse_x, mouse_y) && fov_map.is_in_fov(o.x, o.y))
        .map(|o| o.name.clone())
        .collect::<Vec<_>>()
        .join(", ")

    //names//.join(", ") // return names separated by commas
}

pub fn start_input_proc_thread(
    game_input: &mut Arc<Mutex<GameInput>>,
    rx: Receiver<bool>,
) -> JoinHandle<()> {
    let game_input_buf = Arc::clone(&game_input);
    let key_to_action_mapping = create_key_mapping();

    thread::spawn(move || {
        loop {
            let mut mouse_x: i32 = 0;
            let mut mouse_y: i32 = 0;
            let _mouse: Mouse = Default::default(); // this is not really used right now
            let mut _key: Key = Default::default();
            match input::check_for_event(input::MOUSE | input::KEY_PRESS) {
                // record mouse position for later use
                Some((_, Event::Mouse(_m))) => {
                    mouse_x = _m.cx as i32;
                    mouse_y = _m.cy as i32;
                    println!("[input thread] mouse moved {},{}", _m.cx, _m.cy);
                }
                // get used key to create next user action
                Some((_, Event::Key(k))) => {
                    _key = k;
                    println!("[input thread] key input {:?}", k.code);
                }
                _ => {}
            }

            // lock our mutex and get to work
            let mut input = game_input_buf.lock().unwrap();
            // let player_action: PlayerAction =
            if let Some(key) = key_to_action_mapping.get(&tcod_to_key_code(_key)) {
                // println!("[input thread] push back {:?}", key);
                input.next_player_actions.push_back(key.clone());
            };
            input.mouse_x = mouse_x;
            input.mouse_y = mouse_y;

            match rx.try_recv() {
                Ok(true) | Err(TryRecvError::Disconnected) => {
                    println!("[input thread] terminating");
                    break;
                }
                _ => {}
            }

            // println!("[input thread] key {:?}", _key);
            // println!("[input thread] mouse {:?}", _mouse);
        }
    })
}

/// Translate between tcod's keys and our own key codes.
fn tcod_to_key_code(tcod_key: tcod::input::Key) -> self::KeyCode {
    use tcod::input::KeyCode::*;

    match tcod_key {
        // in-game actions
        Key { code: Up, .. } => self::KeyCode::Up,
        Key { code: Down, .. } => self::KeyCode::Down,
        Key { code: Right, .. } => self::KeyCode::Right,
        Key { code: Left, .. } => self::KeyCode::Left,
        // non-in-game actions
        Key { code: Escape, .. } => self::KeyCode::Esc,
        Key { code: F4, .. } => self::KeyCode::F4,
        _ => self::KeyCode::UndefinedKey,
    }
}

fn create_key_mapping() -> HashMap<KeyCode, PlayerAction> {
    use self::KeyCode::*;
    use self::PlayerAction::*;
    use self::UiAction::*;

    let mut key_map: HashMap<KeyCode, PlayerAction> = HashMap::new();

    // TODO: Fill mapping from json file.
    // set up all in-game actions
    key_map.insert(Up, WalkNorth);
    key_map.insert(Down, WalkSouth);
    key_map.insert(Left, WalkWest);
    key_map.insert(Right, WalkEast);
    // set up all non-in-game actions.
    key_map.insert(Esc, MetaAction(ExitGameLoop));
    key_map.insert(F4, MetaAction(Fullscreen));
    key_map.insert(C, MetaAction(CharacterScreen));

    key_map
}

pub fn get_player_action_instance(player_action: PlayerAction) -> Box<dyn Action> {
    // TODO: Use actual costs.
    // No need to map `Esc` since we filter out exiting before instantiating
    // any player actions.
    // println!("player action: {:?}", player_action);
    match player_action {
        PlayerAction::WalkNorth => Box::new(MoveAction::new(Direction::North, 0)),
        PlayerAction::WalkSouth => Box::new(MoveAction::new(Direction::South, 0)),
        PlayerAction::WalkEast => Box::new(MoveAction::new(Direction::East, 0)),
        PlayerAction::WalkWest => Box::new(MoveAction::new(Direction::West, 0)),
        _ => Box::new(PassAction),
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
