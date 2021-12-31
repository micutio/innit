use crate::entity::act;
use crate::entity::control;
use crate::game::position::Position;
use crate::game::{self, env, ObjectStore, State};
use crate::ui::hud;



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
    ChoosePrimary,
    ChooseSecondary,
    ChooseQuick1,
    ChooseQuick2,
    GenomeEditor,
    Help,
    SetFont(usize),
}

#[derive(Clone, Debug)]
pub enum PlayerAction {
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
) -> PlayerInput {
    use self::act::Target::*;
    use rltk::VirtualKeyCode as Vkc;
    match (key, ctrl, shift) {
        // letters
        (Vkc::A, false, false) => PlayerInput::Game(PlayerAction::Secondary(West)),
        (Vkc::C, false, false) => PlayerInput::Meta(UiAction::CharacterScreen),
        (Vkc::D, false, false) => PlayerInput::Game(PlayerAction::Secondary(East)),
        (Vkc::E, false, false) => PlayerInput::Game(PlayerAction::Quick2),
        (Vkc::E, false, true) => PlayerInput::Meta(UiAction::ChooseQuick2),
        (Vkc::G, false, false) => {
            if env().is_debug_mode {
                PlayerInput::Meta(UiAction::GenomeEditor)
            } else {
                PlayerInput::Undefined
            }
        }
        (Vkc::P, false, true) => PlayerInput::Meta(UiAction::ChoosePrimary),
        (Vkc::Q, false, false) => PlayerInput::Game(PlayerAction::Quick1),
        (Vkc::Q, false, true) => PlayerInput::Meta(UiAction::ChooseQuick1),
        (Vkc::S, false, false) => PlayerInput::Game(PlayerAction::Secondary(South)),
        (Vkc::S, false, true) => PlayerInput::Meta(UiAction::ChooseSecondary),
        (Vkc::S, true, true) => {
            take_screenshot(ctx);
            PlayerInput::Undefined
        }
        (Vkc::W, false, false) => PlayerInput::Game(PlayerAction::Secondary(North)),
        (Vkc::Up, false, false) => PlayerInput::Game(PlayerAction::Primary(North)),
        (Vkc::Down, false, false) => PlayerInput::Game(PlayerAction::Primary(South)),
        (Vkc::Left, false, false) => PlayerInput::Game(PlayerAction::Primary(West)),
        (Vkc::Right, false, false) => PlayerInput::Game(PlayerAction::Primary(East)),
        (Vkc::Space, false, false) => PlayerInput::Game(PlayerAction::PassTurn),
        (Vkc::Escape, false, false) => PlayerInput::Meta(UiAction::ExitGameLoop),
        (Vkc::F1, false, false) => PlayerInput::Meta(UiAction::Help),
        (Vkc::Key1, false, false) => PlayerInput::Meta(UiAction::SetFont(0)),
        (Vkc::Key2, false, false) => PlayerInput::Meta(UiAction::SetFont(1)),
        (Vkc::Key3, false, false) => PlayerInput::Meta(UiAction::SetFont(2)),
        (Vkc::Key4, false, false) => PlayerInput::Meta(UiAction::SetFont(3)),
        (Vkc::Key5, false, false) => PlayerInput::Meta(UiAction::SetFont(4)),
        _ => PlayerInput::Undefined,
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
) -> PlayerInput {
    let mut input = rltk::INPUT.lock();
    #[allow(clippy::single_match)]
    input.for_each_message(|event| match event {
        rltk::BEvent::CloseRequested => ctx.quitting = true,
        _ => (),
    });

    // 1) check whether key has been pressed
    use rltk::VirtualKeyCode as Vkc;
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
                    if let act::TargetCategory::Any = ctrl.primary_action.get_target_category() {
                        return PlayerInput::Game(PlayerAction::Primary(act::Target::from_pos(
                            &player.pos,
                            &mouse,
                        )));
                    } else if player.pos.is_adjacent(&mouse) {
                        return PlayerInput::Game(PlayerAction::Primary(act::Target::from_pos(
                            &player.pos,
                            &mouse,
                        )));
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
            .find(|i| i.layout.point_in_rect(mouse.into()))
        {
            return if is_clicked {
                match item.item_enum {
                    hud::HudItem::PrimaryAction => PlayerInput::Meta(UiAction::ChoosePrimary),
                    hud::HudItem::SecondaryAction => PlayerInput::Meta(UiAction::ChooseSecondary),
                    hud::HudItem::Quick1Action => PlayerInput::Meta(UiAction::ChooseQuick1),
                    hud::HudItem::Quick2Action => PlayerInput::Meta(UiAction::ChooseQuick2),
                    hud::HudItem::DnaItem => PlayerInput::Undefined, // no action when clicked
                    hud::HudItem::BarItem => PlayerInput::Undefined, // no action when clicked
                    hud::HudItem::UseInventory { idx } => {
                        PlayerInput::Game(PlayerAction::UseInventoryItem(idx))
                    }
                    hud::HudItem::DropInventory { idx } => {
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

#[cfg(not(target_arch = "wasm32"))]
fn take_screenshot(ctx: &mut rltk::BTerm) {
    ctx.screenshot("innit_screenshot.png");
}

#[cfg(target_arch = "wasm32")]
fn take_screenshot(_ctx: &mut rltk::BTerm) {
    info!("screenshots no supported in wasm")
}
