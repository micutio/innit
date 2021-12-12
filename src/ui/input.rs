use crate::entity::act::*;
use crate::entity::control::Controller::Player;
use crate::game::objects::ObjectStore;
use crate::game::position::Position;
use crate::game::State;
use crate::game::{self, env};
use crate::ui::hud::{Hud, HudItem, ToolTip};
use crate::ui::input::PlayerAction::PrimaryAction;
use crate::ui::input::PlayerInput::{Game, Meta};

use rltk::prelude::INPUT;
use rltk::{BEvent, Point, Rltk, VirtualKeyCode};

#[derive(Clone, Debug)]
pub enum PlayerInput {
    Meta(UiAction),
    Game(PlayerAction),
    Undefined,
}

#[derive(Clone, Debug)]
pub enum UiAction {
    ExitGameLoop,
    CharacterScreen,
    ChoosePrimaryAction,
    ChooseSecondaryAction,
    ChooseQuick1Action,
    ChooseQuick2Action,
    GenomeEditor,
    Help,
    SetFont(usize),
}

#[derive(Clone, Debug)]
pub enum PlayerAction {
    PrimaryAction(Target),   // using the arrow keys
    SecondaryAction(Target), // using 'W','A','S','D' keys
    Quick1Action,            // using 'Q', un-targeted quick action
    Quick2Action,            // using 'E', un-targeted second quick action
    PassTurn,
    UseInventoryItem(usize),
    DropItem(usize),
}

/// Translate between bracket's keys and our own key codes.
fn key_to_action(key: VirtualKeyCode, ctrl: bool, shift: bool) -> PlayerInput {
    use self::PlayerAction::*;
    use self::PlayerInput::*;
    use self::Target::*;
    use self::UiAction::*;
    match (key, ctrl, shift) {
        // letters
        (VirtualKeyCode::A, false, false) => Game(SecondaryAction(West)),
        (VirtualKeyCode::C, false, false) => Meta(CharacterScreen),
        (VirtualKeyCode::D, false, false) => Game(SecondaryAction(East)),
        (VirtualKeyCode::E, false, false) => Game(Quick2Action),
        (VirtualKeyCode::E, false, true) => Meta(ChooseQuick2Action),
        (VirtualKeyCode::G, false, false) => {
            if env().is_debug_mode {
                Meta(GenomeEditor)
            } else {
                Undefined
            }
        }
        (VirtualKeyCode::P, false, true) => Meta(ChoosePrimaryAction),
        (VirtualKeyCode::Q, false, false) => Game(Quick1Action),
        (VirtualKeyCode::Q, false, true) => Meta(ChooseQuick1Action),
        (VirtualKeyCode::S, false, false) => Game(SecondaryAction(South)),
        (VirtualKeyCode::S, false, true) => Meta(ChooseSecondaryAction),
        (VirtualKeyCode::W, false, false) => Game(SecondaryAction(North)),
        (VirtualKeyCode::Up, false, false) => Game(PrimaryAction(North)),
        (VirtualKeyCode::Down, false, false) => Game(PrimaryAction(South)),
        (VirtualKeyCode::Left, false, false) => Game(PrimaryAction(West)),
        (VirtualKeyCode::Right, false, false) => Game(PrimaryAction(East)),
        (VirtualKeyCode::Space, false, false) => Game(PassTurn),
        (VirtualKeyCode::Escape, false, false) => Meta(ExitGameLoop),
        (VirtualKeyCode::F1, false, false) => Meta(Help),
        (VirtualKeyCode::Key1, false, false) => Meta(SetFont(0)),
        (VirtualKeyCode::Key2, false, false) => Meta(SetFont(1)),
        (VirtualKeyCode::Key3, false, false) => Meta(SetFont(2)),
        (VirtualKeyCode::Key4, false, false) => Meta(SetFont(3)),
        (VirtualKeyCode::Key5, false, false) => Meta(SetFont(4)),
        (VirtualKeyCode::Key6, false, false) => Meta(SetFont(5)),
        (VirtualKeyCode::Key7, false, false) => Meta(SetFont(6)),
        (VirtualKeyCode::Key8, false, false) => Meta(SetFont(7)),
        _ => Undefined,
    }
}

// Create A detailed info panel as tooltip.
// - list stats and (compare with player) to give hints about strength, receptors and such
// - get player sensor quality, quantity and adjust how much info is shown
// - either take the player out of the objects and compare to everything else
//   or just gather all info and adjust visibility later when rendering tooltips in UI
// useful info:
// - receptor matching or not
// - virus RNA or DNA
fn get_names_under_mouse(
    state: &State,
    objects: &mut ObjectStore,
    mouse: Position,
) -> Vec<ToolTip> {
    let mut tooltips: Vec<ToolTip> = vec![];
    if let Some(player) = objects.extract_by_index(state.player_idx) {
        if player.pos.eq(&mouse) {
            // tooltips.push(ToolTip::header_only("You".to_string()));
            tooltips.push(player.generate_tooltip(&player));
        }

        tooltips.append(
            &mut objects
                .get_vector()
                .iter()
                .flatten()
                .filter(|o| o.pos.eq(&mouse) && o.physics.is_visible)
                //                              vvvvv---- replace function with `key-value`-list generating function.
                .map(|o| o.generate_tooltip(&player))
                .collect::<Vec<_>>(),
        );

        objects.replace(state.player_idx, player);
    }

    tooltips
}

/// Check whether the user has given inputs either via mouse or keyboard. Also update any input-
/// dependent UI elements, like hover-tooltips etc.
pub fn read(
    state: &mut State,
    objects: &mut ObjectStore,
    hud: &mut Hud,
    ctx: &mut Rltk,
) -> PlayerInput {
    let mut input = INPUT.lock();
    #[allow(clippy::single_match)]
    input.for_each_message(|event| match event {
        BEvent::CloseRequested => ctx.quitting = true,
        _ => (),
    });

    // 1) check whether key has been pressed
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
        get_names_under_mouse(state, objects, mouse),
    );

    // 3) if mouse is hovering over world
    if mouse.x < game::consts::WORLD_WIDTH {
        // 3b) check whether a mouse button has been pressed for player action
        if is_clicked {
            // get clicked cell, check if it is adjacent to player, perform primary action
            if let Some(player) = &objects[state.player_idx] {
                if let Some(Player(ctrl)) = &player.control {
                    if let TargetCategory::Any = ctrl.primary_action.get_target_category() {
                        return Game(PrimaryAction(Target::from_pos(&player.pos, &mouse)));
                    } else if player.pos.is_adjacent(&mouse) {
                        return Game(PrimaryAction(Target::from_pos(&player.pos, &mouse)));
                    }
                }
            }
        }
        PlayerInput::Undefined
    } else {
        // 4) is mouse is hovering over sidebar
        // 4a) update hovered button
        if let Some(item) = hud
            .items
            .iter()
            .find(|i| i.layout.point_in_rect(Point::new(mouse.x, mouse.y)))
        {
            return if is_clicked {
                match item.item_enum {
                    HudItem::PrimaryAction => Meta(UiAction::ChoosePrimaryAction),
                    HudItem::SecondaryAction => Meta(UiAction::ChooseSecondaryAction),
                    HudItem::Quick1Action => Meta(UiAction::ChooseQuick1Action),
                    HudItem::Quick2Action => Meta(UiAction::ChooseQuick2Action),
                    HudItem::DnaItem => PlayerInput::Undefined, // no action when clicked
                    HudItem::BarItem => PlayerInput::Undefined, // no action when clicked
                    HudItem::UseInventory { idx } => {
                        PlayerInput::Game(PlayerAction::UseInventoryItem(idx))
                    }
                    HudItem::DropInventory { idx } => {
                        PlayerInput::Game(PlayerAction::DropItem(idx))
                    }
                }
            } else {
                PlayerInput::Undefined
            };
        };
        // 3b) check for button press to activate ui buttons
        PlayerInput::Undefined
    }
}
