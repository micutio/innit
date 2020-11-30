use crate::core::game_objects::GameObjects;
use crate::core::game_state::GameState;
use crate::core::position::Position;
use crate::entity::action::*;
use crate::entity::control::Controller::Player;
use crate::game::WORLD_WIDTH;
use crate::ui::game_input::PlayerAction::PrimaryAction;
use crate::ui::game_input::PlayerInput::{MetaInput, PlayInput};
use crate::ui::gui::{Hud, HudItem};
use rltk::{Point, Rltk, VirtualKeyCode};

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

pub fn read_input(
    state: &mut GameState,
    objects: &mut GameObjects,
    hud: &mut Hud,
    ctx: &mut Rltk,
) -> PlayerInput {
    // 1) check if key has been pressed

    if let Some(key) = ctx.key {
        debug!("key: {:#?}", ctx.key);
        return key_to_action(key, ctx.control, ctx.shift);
    }
    let mouse = Position::from_point(ctx.mouse_point());
    let clicked: bool = ctx.left_click;

    // 2) if mouse is over world
    if mouse.x < WORLD_WIDTH {
        // 2a) update hovered objects
        hud.set_names_under_mouse(get_names_under_mouse(objects, mouse));
        // 2b) check whether a mouse button has been pressed for player action
        if clicked {
            // get clicked cell, check if it is adjacent to player, perform primary action
            if let Some(player) = &objects[state.current_player_index] {
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
        // 3) is mouse is over sidebar
        // 3a) update hovered button
        if let Some(item) = hud
            .items
            .iter()
            .find(|i| i.layout.point_in_rect(Point::new(mouse.x, mouse.y)))
        {
            // TODO: Change item appearance!
            return match item.item_enum {
                HudItem::PrimaryAction => MetaInput(UiAction::ChoosePrimaryAction),
                HudItem::SecondaryAction => MetaInput(UiAction::ChooseSecondaryAction),
                HudItem::Quick1Action => MetaInput(UiAction::ChooseQuick1Action),
                HudItem::QuickAction2 => MetaInput(UiAction::ChooseQuick2Action),
            };
        };
        // 3b) check for button press to activate ui buttons
        PlayerInput::Undefined
    }
}
