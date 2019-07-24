use std::collections::{HashMap, VecDeque};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use tcod::input::{self, Event, Key, Mouse};

use crate::core::game_objects::GameObjects;
use crate::core::game_state::{GameState, PLAYER};
use crate::entity::action::*;
use crate::ui::game_frontend::{re_render, FovMap, GameFrontend};

/// Enum for thread control.
#[derive(Debug, Clone, Copy)]
enum InputThreadCommand {
    Resume,
    Pause,
    Stop,
}

pub struct MousePosition {
    x: i32,
    y: i32,
}

/// Concurrent input contains
/// - the action queue that is shared between game and concurrent input listener thread
/// - the cannel that allows the ui to communicate with the running thread
/// - the handle of the input thread itself
pub struct ConcurrentInput {
    game_input_ref: Arc<Mutex<InputProcessor>>,
    input_thread_tx: Sender<InputThreadCommand>,
    input_thread: JoinHandle<()>,
}

/// Since joining the thread consumes the handle, the concurrent input component
/// needs to be moved out of the game input structure before doing so.
/// To restart the thread, the game input will instantiate a new concurrent component.
impl ConcurrentInput {
    fn join_thread(self) {
        match self.input_thread.join() {
            Ok(_) => debug!("successfully joined user input thread"),
            Err(e) => error!("error while trying to join user input thread: {:#?}", e),
        }
    }
}

/// The game input contains fields for
/// - current mouse position
/// - the names of all objects currently `inspected` by the mouse cursor
/// - a handle for the concurrent input component
/// - next player action, based on what key was pressed last
/// This encapsulated the complete game input handling and constitutes the
/// interface towards the frontend.
pub struct GameInput {
    pub current_mouse_pos: MousePosition,
    pub names_under_mouse: String,
    concurrent_input: Option<ConcurrentInput>,
    next_action: Option<PlayerAction>,
}

impl GameInput {
    /// Create a new instance of the user input.
    pub fn new() -> Self {
        GameInput {
            current_mouse_pos: MousePosition { x: 0, y: 0 },
            names_under_mouse: Default::default(),
            concurrent_input: None,
            next_action: None,
        }
    }

    /// Start a new instance of the input listener thread.
    pub fn start_concurrent_input(&mut self) {
        let (tx, rx) = mpsc::channel();
        let game_input = Arc::new(Mutex::new(InputProcessor::new()));
        let mut game_input_ref = Arc::clone(&game_input);
        let input_thread = start_input_proc_thread(&mut game_input_ref, rx);

        self.concurrent_input = Some(ConcurrentInput {
            game_input_ref,
            input_thread_tx: tx,
            input_thread,
        })
    }

    /// Tell the input thread to stop listening to input and idle around,
    /// but NOT to stop completely.
    pub fn pause_concurrent_input(&mut self) {
        self.notify_concurrent_input(InputThreadCommand::Pause);
    }

    /// Tell the input thread to resume listening for input.
    pub fn resume_concurrent_input(&mut self) {
        self.notify_concurrent_input(InputThreadCommand::Resume);
    }

    /// Stop the input thread completely and wait for it to join.
    pub fn stop_concurrent_input(&mut self) {
        // clean up any existing threads before creating a new one
        self.notify_concurrent_input(InputThreadCommand::Stop);
        match self.concurrent_input.take() {
            Some(concurrent) => {
                concurrent.join_thread();
            }
            None => panic!("[game input] ERROR: failed to stop concurrent thread!"),
        }
    }

    /// Reset the `next action` field to signal that it has been performed.
    pub fn is_action_consumed(&self) -> bool {
        self.next_action.is_none()
    }

    /// Retrieve the next player action
    pub fn get_next_action(&mut self) -> Option<PlayerAction> {
        self.next_action.take()
    }

    /// Check for new input data:
    /// - update inspection if the mouse has moved
    /// - inject a new action from the queue into the player object if the current one has been consumed
    pub fn check_for_player_actions(
        &mut self,
        game_state: &mut GameState,
        game_frontend: &mut GameFrontend,
        objects: &mut GameObjects,
    ) {
        // use separate scope to get mutex to unlock again, could also use `Mutex::drop()`
        let concurrent_option = self.concurrent_input.take();
        match &concurrent_option {
            Some(concurrent) => {
                let mut data = concurrent.game_input_ref.lock().unwrap();
                // If the mouse moved, update the names and trigger re-render
                if self.check_set_mouse_position(data.mouse_x, data.mouse_y) {
                    self.names_under_mouse = get_names_under_mouse(
                        objects,
                        &game_frontend.fov,
                        self.current_mouse_pos.x,
                        self.current_mouse_pos.y,
                    );
                    re_render(game_state, game_frontend, objects, &self.names_under_mouse);
                    self.names_under_mouse = Default::default();
                }

                if let Some(ref player) = objects[PLAYER] {
                    // ... but only if the previous user action is used up
                    if player.next_action.is_none() {
                        if let Some(new_action) = data.next_player_actions.pop_front() {
                            self.next_action = Some(new_action);
                            debug!("popped next action from queue {:?}", self.next_action);
                        }
                    }
                }
            }
            None => {
                panic!("[game input] concurrent is not there");
            }
        }

        match concurrent_option {
            Some(concurrent) => {
                self.concurrent_input.replace(concurrent);
            }
            None => {
                panic!("[game input] concurrent is still not there");
            }
        }
    }

    /// Check whether the mouse position has changed, and if so, store it.
    /// Return **true** if the mouse position changed, false otherwise.
    fn check_set_mouse_position(&mut self, new_x: i32, new_y: i32) -> bool {
        if self.current_mouse_pos.x != new_x || self.current_mouse_pos.y != new_y {
            self.current_mouse_pos.x = new_x;
            self.current_mouse_pos.y = new_y;
            return true;
        }
        false
    }

    /// Send a given command to the input thread.
    fn notify_concurrent_input(&self, command: InputThreadCommand) {
        if let Some(concurrent) = &self.concurrent_input {
            match concurrent.input_thread_tx.send(command) {
                Ok(_) => {
                    debug!("successfully sent command {:?} to thread", command);
                }
                Err(e) => {
                    error!("error while sending command {:?} to thread: {}", command, e);
                }
            }
        }
    }
}

/// As tcod's key codes don't implement `Eq` and `Hash` they cannot be used
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
    L,
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
    ToggleDarkLightMode,
}

/// The input processor maps user input to player actions.
pub struct InputProcessor {
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub next_player_actions: VecDeque<PlayerAction>,
}

impl InputProcessor {
    pub fn new() -> Self {
        InputProcessor {
            mouse_x: 0,
            mouse_y: 0,
            next_player_actions: VecDeque::new(),
        }
    }
}

pub fn get_names_under_mouse(
    objects: &GameObjects,
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
        .map(|o| o.visual.name.clone())
        .collect::<Vec<_>>()
        .join(", ")

    //names//.join(", ") // return names separated by commas
}

/// Start an input processing thread.
/// Listen to keystrokes and mouse movement at the same time.
fn start_input_proc_thread(
    game_input: &mut Arc<Mutex<InputProcessor>>,
    rx: Receiver<InputThreadCommand>,
) -> JoinHandle<()> {
    let game_input_buf = Arc::clone(&game_input);
    let key_to_action_mapping = create_key_mapping();

    thread::spawn(move || {
        let mut mouse_x: i32 = 0;
        let mut mouse_y: i32 = 0;

        let mut is_paused = false;

        loop {
            if !is_paused {
                let _mouse: Mouse = Default::default(); // this is not really used right now
                let mut _key: Key = Default::default();
                match input::check_for_event(input::MOUSE | input::KEY_PRESS) {
                    // record mouse position for later use
                    Some((_, Event::Mouse(_m))) => {
                        mouse_x = _m.cx as i32;
                        mouse_y = _m.cy as i32;
                        trace!("mouse moved {},{}", _m.cx, _m.cy);
                    }
                    // get used key to create next user action
                    Some((_, Event::Key(k))) => {
                        _key = k;
                        trace!("key input {:?}", k.code);
                    }
                    _ => {}
                }

                // lock our mutex and get to work
                let mut input = game_input_buf.lock().unwrap();
                // let player_action: PlayerAction =
                if let Some(key) = key_to_action_mapping.get(&tcod_to_key_code(_key)) {
                    trace!("[input thread] push back {:?}", key);
                    input.next_player_actions.push_back(key.clone());
                };
                input.mouse_x = mouse_x;
                input.mouse_y = mouse_y;
            }

            match rx.try_recv() {
                Ok(InputThreadCommand::Pause) => {
                    is_paused = true;
                    debug!("pausing input thread");
                }
                Ok(InputThreadCommand::Resume) => {
                    is_paused = false;
                    debug!("resuming input thread");
                }
                Ok(InputThreadCommand::Stop) => {
                    debug!("stopping input thread");
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    debug!("Error: input thread disconnected");
                    break;
                }
                _ => {}
            }
        }
    })
}

/// Translate between tcod's keys and our own key codes.
fn tcod_to_key_code(tcod_key: tcod::input::Key) -> self::KeyCode {
    use tcod::input::KeyCode::*;

    match tcod_key {
        // letters
        Key {
            code: Char,
            printable: 'c',
            ..
        } => self::KeyCode::C,
        Key {
            code: Char,
            printable: 'l',
            ..
        } => self::KeyCode::L,
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

/// Create a mapping between our own key codes and player actions.
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
    key_map.insert(L, MetaAction(ToggleDarkLightMode));

    key_map
}

/// Construct a new player action from a given key code.
/// NOTE: In the future we'll have to consider mouse clicks as well.
pub fn get_player_action_instance(player_action: PlayerAction) -> Box<dyn Action> {
    // TODO: Use actual action energy costs.
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
