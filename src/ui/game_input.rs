use crate::core::game_objects::GameObjects;
use crate::core::game_state::GameState;
use crate::core::position::Position;
use crate::entity::action::Target::North;
use crate::entity::action::*;
use crate::game::MS_PER_FRAME;
use crate::ui::game_input::PlayerAction::PrimaryAction;
use crate::ui::game_input::PlayerInput::PlayInput;
use crate::ui::old_frontend::{re_render, FovMap, GameFrontend};
use rltk::{Rltk, VirtualKeyCode};
use std::collections::{HashMap, VecDeque};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tcod::input::{self, Event, Key, Mouse};

#[derive(Clone, Debug)]
pub enum PlayerInput {
    MetaInput(UiAction),
    PlayInput(PlayerAction),
    Undefined,
}

// TODO: Add `SetPrimaryAction`, `SetSecondaryAction`, `SetQuickAction`
#[derive(Clone, Debug)]
pub enum UiAction {
    ExitGameLoop,
    Fullscreen,
    CharacterScreen,
    ToggleDarkLightMode,
    ChoosePrimaryAction,
    ChooseSecondaryAction,
    ChooseQuick1Action,
    ChooseQuick2Action,
}

#[derive(Clone, Debug)]
pub enum PlayerAction {
    PrimaryAction(Target),   // using the arrow keys
    SecondaryAction(Target), // using 'W','A','S','D' keys
    Quick1Action,            // using 'Q', un-targeted quick action
    Quick2Action,            // using 'E', un-targeted second quick action
    PassTurn,
}

/// Translate between bracket's keys and our own key codes.
fn key_to_action(key: VirtualKeyCode, ctrl: bool, shift: bool) -> PlayerInput {
    use self::PlayerAction::*;
    use self::PlayerInput::*;
    use self::Target::*;
    use self::UiAction::*;
    match (key, ctrl, shift) {
        // letters
        (VirtualKeyCode::A, false, false) => PlayInput(SecondaryAction(West)),
        (VirtualKeyCode::C, false, false) => MetaInput(CharacterScreen),
        (VirtualKeyCode::D, false, false) => PlayInput(SecondaryAction(East)),
        (VirtualKeyCode::E, false, false) => PlayInput(Quick2Action),
        (VirtualKeyCode::E, true, false) => MetaInput(ChooseQuick2Action),
        (VirtualKeyCode::L, false, false) => MetaInput(ToggleDarkLightMode),
        (VirtualKeyCode::P, true, false) => MetaInput(ChoosePrimaryAction),
        (VirtualKeyCode::Q, false, false) => PlayInput(Quick1Action),
        (VirtualKeyCode::Q, true, false) => MetaInput(ChooseQuick1Action),
        (VirtualKeyCode::S, false, false) => PlayInput(SecondaryAction(South)),
        (VirtualKeyCode::S, true, false) => MetaInput(ChooseSecondaryAction),
        (VirtualKeyCode::W, false, false) => PlayInput(SecondaryAction(North)),
        (VirtualKeyCode::Up, true, false) => PlayInput(PrimaryAction(North)),
        (VirtualKeyCode::Down, false, false) => PlayInput(PrimaryAction(South)),
        (VirtualKeyCode::Left, false, false) => PlayInput(PrimaryAction(West)),
        (VirtualKeyCode::Right, false, false) => PlayInput(PrimaryAction(East)),
        (VirtualKeyCode::Space, false, false) => PlayInput(PassTurn),
        (VirtualKeyCode::Escape, false, false) => MetaInput(ExitGameLoop),
        (VirtualKeyCode::F4, false, false) => MetaInput(Fullscreen),
        _ => Undefined,
    }
}

fn get_names_under_mouse(objects: &GameObjects, mouse: Position) -> String {
    // create a list with the names of all objects at the mouse's coordinates and in FOV
    objects
        .get_vector()
        .iter()
        .flatten()
        .filter(|o| o.pos.eq(&mouse) && o.physics.is_visible)
        .map(|o| o.visual.name.clone())
        .collect::<Vec<_>>()
        .join(", ")

    // names//.join(", ") // return names separated by commas
}

// TODO: Complete
fn read_input(ctx: &mut Rltk) {}
