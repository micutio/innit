use crate::entity::act;
use crate::entity::control;
use crate::game::position::Position;
use crate::game::{self, env, ObjectStore, State};
use crate::ui::hud;

use bracket_lib::prelude as rltk;
use rltk::VirtualKeyCode as Vkc;

#[derive(Clone, Debug)]
pub enum PlayerSignal {
    Meta(UiAction),
    Game(InGameAction),
    Undefined,
}

#[derive(Clone, Debug)]
pub enum UiAction {
    ExitGameLoop,
    CharacterScreen,
    ChoosePrimary,
    ChooseSecondary,
    ChooseQuick1,
    ChooseQuick2,
    GenomeEditor,
    Help,
    SetComplementDisplay(usize),
    SetFont(usize),
}

#[derive(Clone, Debug)]
pub enum InGameAction {
    Primary(act::Target),   // using the arrow keys
    Secondary(act::Target), // using 'W','A','S','D' keys
    Quick1,                 // using 'Q', un-targeted quick action
    Quick2,                 // using 'E', un-targeted second quick action
    PassTurn,
    UseInventoryItem(usize),
    DropItem(usize),
}

/// Translate between bracket's keys and our own key codes.
fn key_to_action(
    ctx: &mut rltk::BTerm,
    key: rltk::VirtualKeyCode,
    ctrl: bool,
    shift: bool,
) -> PlayerSignal {
    match (key, ctrl, shift) {
        // letters
        (Vkc::A, false, false) => PlayerSignal::Game(InGameAction::Secondary(act::Target::West)),
        (Vkc::C, false, false) => PlayerSignal::Meta(UiAction::CharacterScreen),
        (Vkc::D, false, false) => PlayerSignal::Game(InGameAction::Secondary(act::Target::East)),
        (Vkc::E, false, false) => PlayerSignal::Game(InGameAction::Quick2),
        (Vkc::E, false, true) => PlayerSignal::Meta(UiAction::ChooseQuick2),
        #[allow(clippy::significant_drop_in_scrutinee)]
        (Vkc::G, false, false) => match env().debug_mode {
            game::env::GameOption::Enabled => PlayerSignal::Meta(UiAction::GenomeEditor),
            game::env::GameOption::Disabled => PlayerSignal::Undefined,
        },
        (Vkc::P, false, true) => PlayerSignal::Meta(UiAction::ChoosePrimary),
        (Vkc::Q, false, false) => PlayerSignal::Game(InGameAction::Quick1),
        (Vkc::Q, false, true) => PlayerSignal::Meta(UiAction::ChooseQuick1),
        (Vkc::S, false, false) => PlayerSignal::Game(InGameAction::Secondary(act::Target::South)),
        (Vkc::S, false, true) => PlayerSignal::Meta(UiAction::ChooseSecondary),
        (Vkc::S, true, true) => {
            take_screenshot(ctx);
            PlayerSignal::Undefined
        }
        (Vkc::W, false, false) => PlayerSignal::Game(InGameAction::Secondary(act::Target::North)),
        (Vkc::Up, false, false) => PlayerSignal::Game(InGameAction::Primary(act::Target::North)),
        (Vkc::Down, false, false) => PlayerSignal::Game(InGameAction::Primary(act::Target::South)),
        (Vkc::Left, false, false) => PlayerSignal::Game(InGameAction::Primary(act::Target::West)),
        (Vkc::Right, false, false) => PlayerSignal::Game(InGameAction::Primary(act::Target::East)),
        (Vkc::Space, false, false) => PlayerSignal::Game(InGameAction::PassTurn),
        (Vkc::Escape, false, false) => PlayerSignal::Meta(UiAction::ExitGameLoop),
        (Vkc::F1, false, false) => PlayerSignal::Meta(UiAction::Help),
        (Vkc::Key1, false, false) => PlayerSignal::Meta(UiAction::SetComplementDisplay(3)),
        (Vkc::Key2, false, false) => PlayerSignal::Meta(UiAction::SetComplementDisplay(0)),
        (Vkc::Key3, false, false) => PlayerSignal::Meta(UiAction::SetComplementDisplay(1)),
        (Vkc::Key4, false, false) => PlayerSignal::Meta(UiAction::SetComplementDisplay(2)),
        (Vkc::Key5, false, false) => PlayerSignal::Meta(UiAction::SetFont(0)),
        (Vkc::Key6, false, false) => PlayerSignal::Meta(UiAction::SetFont(1)),
        (Vkc::Key7, false, false) => PlayerSignal::Meta(UiAction::SetFont(2)),
        (Vkc::Key8, false, false) => PlayerSignal::Meta(UiAction::SetFont(3)),
        (Vkc::Key9, false, false) => PlayerSignal::Meta(UiAction::SetFont(4)),
        _ => PlayerSignal::Undefined,
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
) -> Vec<hud::ToolTip> {
    let mut tooltips: Vec<hud::ToolTip> = vec![];
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
    hud: &mut hud::Hud,
    ctx: &mut rltk::BTerm,
) -> PlayerSignal {
    let mut input = rltk::INPUT.lock();
    #[allow(clippy::single_match)]
    input.for_each_message(|event| match event {
        rltk::BEvent::CloseRequested => ctx.quitting = true,
        _ => (),
    });

    // 1) check whether key has been pressed

    let ctrl = input.key_pressed_set().contains(&Vkc::LControl)
        || input.key_pressed_set().contains(&Vkc::RControl);
    let shift = input.key_pressed_set().contains(&Vkc::LShift)
        || input.key_pressed_set().contains(&Vkc::RShift);

    if let Some(key) = ctx.key {
        return key_to_action(ctx, key, ctrl, shift);
    }

    let mouse = Position::from(ctx.mouse_point());
    let is_clicked: bool = ctx.left_click;

    // 2) update hovered objects
    hud.update_tooltips(mouse.into(), get_names_under_mouse(state, objects, mouse));

    // 3) if mouse is hovering over world
    if mouse.x() < game::consts::WORLD_WIDTH {
        // 3b) check whether a mouse button has been pressed for player action
        if is_clicked {
            // get clicked cell, check if it is adjacent to player, perform primary action
            if let Some(player) = &objects[state.player_idx] {
                if let Some(control::Controller::Player(ctrl)) = &player.control {
                    let is_any_possible = matches!(
                        ctrl.primary_action.get_target_category(),
                        act::TargetCategory::Any
                    );

                    if is_any_possible || player.pos.is_adjacent(&mouse) {
                        return PlayerSignal::Game(InGameAction::Primary(act::Target::from_pos(
                            &player.pos,
                            &mouse,
                        )));
                    }
                }
            }
        }
    } else {
        // 4) is mouse is hovering over sidebar
        // 4a) update hovered button
        if let Some(item) = hud
            .items
            .iter()
            .find(|i| i.layout.point_in_rect(mouse.into()))
        {
            if is_clicked {
                return match item.item_enum {
                    hud::Item::PrimaryAction => PlayerSignal::Meta(UiAction::ChoosePrimary),
                    hud::Item::SecondaryAction => PlayerSignal::Meta(UiAction::ChooseSecondary),
                    hud::Item::Quick1Action => PlayerSignal::Meta(UiAction::ChooseQuick1),
                    hud::Item::Quick2Action => PlayerSignal::Meta(UiAction::ChooseQuick2),
                    hud::Item::DnaElement | hud::Item::BarElement => PlayerSignal::Undefined, // no action when clicked
                    hud::Item::UseInventory { idx } => {
                        PlayerSignal::Game(InGameAction::UseInventoryItem(idx))
                    }
                    hud::Item::DropInventory { idx } => {
                        PlayerSignal::Game(InGameAction::DropItem(idx))
                    }
                };
            }
        };
        // 3b) check for button press to activate ui buttons
    }
    PlayerSignal::Undefined
}

#[cfg(not(target_arch = "wasm32"))]
fn take_screenshot(ctx: &mut rltk::BTerm) {
    ctx.screenshot("innit_screenshot.png");
}

#[cfg(target_arch = "wasm32")]
fn take_screenshot(_ctx: &mut rltk::BTerm) {
    info!("screenshots no supported in wasm")
}
