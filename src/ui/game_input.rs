use crate::core::game_objects::GameObjects;
use crate::core::game_state::GameState;
use crate::core::position::Position;
use crate::entity::action::*;
use crate::entity::control::Controller::Player;
use crate::game::WORLD_WIDTH;
use crate::ui::game_input::PlayerAction::PrimaryAction;
use crate::ui::game_input::PlayerInput::{MetaInput, PlayInput};
use crate::ui::gui::{Hud, HudItem};
use rltk::prelude::INPUT;
use rltk::{BEvent, Point, Rltk, VirtualKeyCode};

#[derive(Clone, Debug)]
pub enum PlayerInput {
    MetaInput(UiAction),
    PlayInput(PlayerAction),
    Undefined,
}

#[derive(Clone, Debug)]
pub enum UiAction {
    ExitGameLoop,
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
        (VirtualKeyCode::Up, false, false) => PlayInput(PrimaryAction(North)),
        (VirtualKeyCode::Down, false, false) => PlayInput(PrimaryAction(South)),
        (VirtualKeyCode::Left, false, false) => PlayInput(PrimaryAction(West)),
        (VirtualKeyCode::Right, false, false) => PlayInput(PrimaryAction(East)),
        (VirtualKeyCode::Space, false, false) => PlayInput(PassTurn),
        (VirtualKeyCode::Escape, false, false) => MetaInput(ExitGameLoop),
        _ => Undefined,
    }
}

fn get_names_under_mouse(objects: &GameObjects, mouse: Position) -> Vec<String> {
    // create a list with the names of all objects at the mouse's coordinates and in FOV
    objects
        .get_vector()
        .iter()
        .flatten()
        .filter(|o| o.pos.eq(&mouse) && o.physics.is_visible)
        .map(|o| o.visual.name.clone())
        .collect::<Vec<_>>()
    // .join(", ")
}

pub fn read_input(
    state: &mut GameState,
    objects: &mut GameObjects,
    hud: &mut Hud,
    ctx: &mut Rltk,
) -> PlayerInput {
    let mut input = INPUT.lock();
    #[allow(clippy::single_match)]
    input.for_each_message(|event| match event {
        BEvent::CloseRequested => ctx.quitting = true,
        _ => (),
    });

    // 1) check if key has been pressed
    let ctrl = input.key_pressed_set().contains(&VirtualKeyCode::LControl)
        || input.key_pressed_set().contains(&VirtualKeyCode::RControl);
    let shift = input.key_pressed_set().contains(&VirtualKeyCode::LShift)
        || input.key_pressed_set().contains(&VirtualKeyCode::RShift);

    if let Some(key) = ctx.key {
        return key_to_action(key, ctrl, shift);
    }

    let mouse = Position::from(ctx.mouse_point());
    let is_clicked: bool = ctx.left_click;

    // 2) update hovered objects
    hud.update_tooltips(
        Point::from((mouse.x, mouse.y)),
        get_names_under_mouse(objects, mouse),
    );

    // 3) if mouse is over world
    if mouse.x < WORLD_WIDTH {
        // 3b) check whether a mouse button has been pressed for player action
        if is_clicked {
            // get clicked cell, check if it is adjacent to player, perform primary action
            if let Some(player) = &objects[state.player_idx] {
                if let Some(Player(ctrl)) = &player.control {
                    if let TargetCategory::Any = ctrl.primary_action.get_target_category() {
                        return PlayInput(PrimaryAction(Target::from_pos(&player.pos, &mouse)));
                    } else if player.pos.is_adjacent(&mouse) {
                        return PlayInput(PrimaryAction(Target::from_pos(&player.pos, &mouse)));
                    }
                }
            }
        }
        PlayerInput::Undefined
    } else {
        // 4) is mouse is over sidebar
        // 4a) update hovered button
        if let Some(item) = hud
            .items
            .iter()
            .find(|i| i.layout.point_in_rect(Point::new(mouse.x, mouse.y)))
        {
            return if is_clicked {
                match item.item_enum {
                    HudItem::PrimaryAction => MetaInput(UiAction::ChoosePrimaryAction),
                    HudItem::SecondaryAction => MetaInput(UiAction::ChooseSecondaryAction),
                    HudItem::Quick1Action => MetaInput(UiAction::ChooseQuick1Action),
                    HudItem::Quick2Action => MetaInput(UiAction::ChooseQuick2Action),
                    HudItem::DnaItem => PlayerInput::Undefined,
                }
            } else {
                PlayerInput::Undefined
            };
        };
        // 3b) check for button press to activate ui buttons
        PlayerInput::Undefined
    }
}
