use tcod::chars;
/// Module Dialog
///
/// Dialogs can be menus with options to select from
/// or simple message boxes.
use tcod::console::*;

use core::game_objects::GameObjects;
use core::game_state::{GameState, LEVEL_UP_BASE, LEVEL_UP_FACTOR, PLAYER};
use ui::game_frontend::{GameFrontend, BAR_WIDTH, PANEL_HEIGHT, SCREEN_HEIGHT, SCREEN_WIDTH};
use ui::game_input::GameInput;

// message box measurements
pub const MSG_X: i32 = BAR_WIDTH + 2;
pub const MSG_WIDTH: i32 = SCREEN_WIDTH - BAR_WIDTH - 2;
pub const MSG_HEIGHT: usize = PANEL_HEIGHT as usize - 1;
// width of the character info screen.
pub const CHARACTER_SCREEN_WIDTH: i32 = 30;

// Display a generic menu with multiple options to choose from.
/// Returns the number of the menu item that has been chosen.
/// TODO: Make this private
pub fn menu<T: AsRef<str>>(
    game_frontend: &mut GameFrontend,
    game_input: &mut Option<&mut GameInput>,
    header: &str,
    options: &[T],
    width: i32,
) -> Option<usize> {
    assert!(
        options.len() <= 26,
        "Cannot have a mnu with more than 26 options."
    );

    // calculate total height for the header (after auto-wrap) and one line per option
    let header_height = game_frontend
        .root
        .get_height_rect(0, 0, width, SCREEN_HEIGHT, header);

    let height = options.len() as i32 + header_height + 2;

    // create an off-screen console that represents the menu's window
    let mut window = Offscreen::new(width, height);

    // print the header, with auto-wrap
    for x in 0..width {
        for y in 0..height {
            window.set_char_background(
                x,
                y,
                game_frontend.coloring.get_col_menu_bg(),
                BackgroundFlag::Set,
            );
            window.set_char_foreground(x, y, game_frontend.coloring.get_col_menu_fg());
            window.set_char(x, y, ' ');
        }
    }

    // render horizontal borders
    for x in 0..width - 1 {
        window.set_char(x, 0, chars::HLINE);
        window.set_char(x, 1, chars::DHLINE);
        window.set_char(x, height - 1, chars::HLINE);
    }
    // render vertical borders
    for y in 0..height - 1 {
        window.set_char(0, y, chars::VLINE);
        window.set_char(width - 1, y, chars::VLINE);
    }

    // render corners
    window.set_char(0, 0, '\u{da}');
    window.set_char(0, 1, '\u{d4}');
    window.set_char(width - 1, 0, chars::NE);
    window.set_char(width - 1, 1, chars::COPYRIGHT);
    window.set_char(0, height - 1, chars::SW);
    window.set_char(width - 1, height - 1, chars::SE);
    // window.set_char(width, height - 1, chars::SW);
    // window.set_char(width, 1, chars::SE);

    window.print_rect_ex(
        width / 2 as i32,
        0,
        width,
        height,
        BackgroundFlag::None,
        TextAlignment::Center,
        header,
    );

    // print all the options
    for (index, option_text) in options.iter().enumerate() {
        let menu_letter = (b'a' + index as u8) as char;
        let text = format!(" ({}) {}", menu_letter, option_text.as_ref());
        window.print_ex(
            0,
            header_height + index as i32 + 1,
            BackgroundFlag::None,
            TextAlignment::Left,
            text,
        );
    }

    // blit contents of "window" to the root console
    let x = SCREEN_WIDTH / 2 - width / 2;
    let y = SCREEN_HEIGHT / 2 - height / 2;
    tcod::console::blit(
        &window,
        (0, 0),
        (width, height),
        &mut game_frontend.root,
        (x, y),
        1.0,
        0.7,
    );

    // present the root console to the player and wait for a key-press
    game_frontend.root.flush();

    // if we have an instance of GameInput, pause the input listener thread first
    // so that we can receive input events directly
    let key: tcod::input::Key;
    match game_input {
        // NOTE: We can't use pause_concurrent_input() and resume_concurrent_input(). Why?
        // NOTE: If we do that, the game ends up unable to process any key input.
        Some(ref mut handle) => {
            handle.stop_concurrent_input();
            key = game_frontend.root.wait_for_keypress(true);
            // after we got he key, restart input listener thread
            handle.start_concurrent_input();
        }
        None => {
            key = game_frontend.root.wait_for_keypress(true);
        }
    }

    // convert the ASCII code to an index
    // if it corresponds to an option, return it
    if key.printable.is_alphabetic() {
        let index = key.printable.to_ascii_lowercase() as usize - 'a' as usize;
        if index < options.len() {
            Some(index)
        } else {
            None
        }
    } else {
        None
    }
}

/// Display a generic menu with multiple options to choose from.
/// Returns the number of the menu item that has been chosen.
/// TODO: Make this private
pub fn msgbox(
    game_frontend: &mut GameFrontend,
    game_input: &mut Option<&mut GameInput>,
    header: &str,
    text: &str,
    width: i32,
) {
    // calculate total height for the header and text (after auto-wrap)
    let header_height = game_frontend
        .root
        .get_height_rect(0, 0, width, SCREEN_HEIGHT, header);
    let text_height = game_frontend
        .root
        .get_height_rect(0, 0, width, SCREEN_HEIGHT, text);

    let height = header_height + text_height + 2;

    // create an off-screen console that represents the menu's window
    let mut window = Offscreen::new(width, height);

    // set background and foreground colors
    for x in 0..width {
        for y in 0..height {
            window.set_char_background(
                x,
                y,
                game_frontend.coloring.get_col_menu_bg(),
                BackgroundFlag::Set,
            );
            window.set_char_foreground(x, y, game_frontend.coloring.get_col_menu_fg());
            window.set_char(x, y, ' ');
        }
    }

    // render horizontal borders
    for x in 0..width - 1 {
        window.set_char(x, 0, chars::HLINE);
        window.set_char(x, 1, chars::DHLINE);
        window.set_char(x, height - 1, chars::HLINE);
    }
    // render vertical borders
    for y in 0..height - 1 {
        window.set_char(0, y, chars::VLINE);
        window.set_char(width - 1, y, chars::VLINE);
    }

    // render corners
    window.set_char(0, 0, '\u{da}');
    window.set_char(0, 1, '\u{d4}');
    window.set_char(width - 1, 0, chars::NE);
    window.set_char(width - 1, 1, chars::COPYRIGHT);
    window.set_char(0, height - 1, chars::SW);
    window.set_char(width - 1, height - 1, chars::SE);
    // window.set_char(width, height - 1, chars::SW);
    // window.set_char(width, 1, chars::SE);

    // print header with multi-line wrap
    window.print_rect_ex(
        width / 2 as i32,
        0,
        width,
        height,
        BackgroundFlag::None,
        TextAlignment::Center,
        header,
    );

    // print text with multi-line wrap
    window.print_rect_ex(
        1,
        2,
        width,
        height,
        BackgroundFlag::None,
        TextAlignment::Left,
        text,
    );

    // blit contents of "window" to the root console
    let x = SCREEN_WIDTH / 2 - width / 2;
    let y = SCREEN_HEIGHT / 2 - height / 2;
    tcod::console::blit(
        &window,
        (0, 0),
        (width, height),
        &mut game_frontend.root,
        (x, y),
        1.0,
        1.0,
    );

    // present the root console to the player and wait for a key-press
    game_frontend.root.flush();

    // if we have an instance of GameInput, pause the input listener thread first
    // so that we can receive input events directly
    match game_input {
        // NOTE: We can't use pause_concurrent_input() and resume_concurrent_input(). Why?
        // NOTE: If we do that, the game ends up unable to process any key input.
        Some(ref mut handle) => {
            handle.stop_concurrent_input();
            game_frontend.root.wait_for_keypress(true);
            // after we got he key, restart input listener thread
            handle.start_concurrent_input();
        }
        None => {
            game_frontend.root.wait_for_keypress(true);
        }
    }
}

pub fn show_character_screen(
    game_state: &mut GameState,
    game_frontend: &mut GameFrontend,
    game_input: &mut Option<&mut GameInput>,
    game_objects: &mut GameObjects,
) {
    if let Some(ref player) = game_objects[PLAYER] {
        let level = player.level;
        let level_up_xp = LEVEL_UP_BASE + player.level * LEVEL_UP_FACTOR;
        if let Some(fighter) = player.fighter.as_ref() {
            let header: String = "Character Information".to_string();
            let msg: String = format!(
                "Level: {} \n\
                 Experience: {} \n\
                 Experience to level up: {} \n\
                 \n\
                 Maximum HP: {} \n\
                 Attack: {} \n\
                 Defense: {} ",
                level,
                fighter.xp,
                level_up_xp,
                player.max_hp(game_state),
                player.power(game_state),
                player.defense(game_state),
            );
            msgbox(
                game_frontend,
                game_input,
                &header,
                &msg,
                CHARACTER_SCREEN_WIDTH,
            );
        }
    };
}
