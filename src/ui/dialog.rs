/// Module Dialog
///
/// Dialogs can be menus with options to select from
/// or simple message boxes.
use tcod::{chars, console::*, input::Key};

use crate::core::game_objects::GameObjects;
use crate::core::game_state::GameState;
use crate::ui::game_input::GameInput;
use crate::ui::old_frontend::{GameFrontend, BAR_WIDTH, PANEL_HEIGHT, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::util::modulus;

// message box measurements
pub const MSG_X: i32 = BAR_WIDTH + 2;
pub const MSG_WIDTH: i32 = SCREEN_WIDTH - BAR_WIDTH - 2;
pub const MSG_HEIGHT: usize = PANEL_HEIGHT as usize - 1;
// width of the character info screen.
pub const CHARACTER_SCREEN_WIDTH: i32 = 30;

// Display a generic menu with multiple options to choose from.
/// Return the number of the menu item that has been chosen.
pub fn menu<T: AsRef<str>>(
    frontend: &mut GameFrontend,
    input: &mut Option<&mut GameInput>,
    header: &str,
    options: &[T],
    width: i32,
) -> Option<usize> {
    assert!(
        options.len() <= 26,
        "Cannot have a menu with more than 26 options."
    );

    // keep track of which option is currently selected
    let mut current_option: i32 = 0;

    // calculate total height for the header (after auto-wrap) and one line per option
    let header_height = frontend
        .root
        .get_height_rect(0, 0, width, SCREEN_HEIGHT, header);

    let height = options.len() as i32 + header_height + 2;

    // check if we have a running instance of GameInput;
    // if yes, suspend it so we can read from the input directly
    if let Some(instance) = input {
        instance.pause_concurrent_input();
    }

    // keep redrawing everything as long as we just move around the options of the menu
    // once an option is selected, return it
    loop {
        // create an off-screen console that represents the menu's window
        let mut window = Offscreen::new(width, height);

        // initialize coloring for each cell in the text box
        // choose different color for currently selected option
        let color_normal = frontend.coloring.bg_dialog;
        let color_option_highlight = frontend.coloring.fg_dialog_highlight;
        for x in 0..width {
            for y in 0..height {
                // offset by 2 because the first to lines are header and separator
                let bg_color = if y == current_option + 2 && x > 0 && x < width - 1 {
                    color_option_highlight
                } else {
                    color_normal
                };
                window.set_char_background(x, y, bg_color, BackgroundFlag::Set);
                window.set_char_foreground(x, y, frontend.coloring.fg_dialog_border);
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

        window.set_default_foreground(frontend.coloring.fg_dialog);

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
            &mut frontend.root,
            (x, y),
            1.0,
            1.0,
        );

        // present the root console to the player and wait for a key-press
        frontend.root.flush();

        // listen for keypress
        let key = frontend.root.wait_for_keypress(true);
        use tcod::input::KeyCode::*;
        match key {
            // for arrow up/down increase/decrease the current option index
            // for enter return current option number
            // for any alphabetic letter also return current option
            // NOTE: In the future replace letters with numbers or omit completely.
            Key { code: Up, .. } => {
                current_option = modulus(current_option - 1, options.len() as i32)
            }
            Key { code: Down, .. } => {
                current_option = modulus(current_option + 1, options.len() as i32)
            }
            Key { code: Char, .. } => {
                // convert the ASCII code to an index
                // if it corresponds to an option, return it

                if key.printable.is_alphabetic() {
                    let index = key.printable.to_ascii_lowercase() as usize - 'a' as usize;
                    if index < options.len() {
                        // before returning, re-start concurrent user input again
                        if let Some(instance) = input {
                            instance.resume_concurrent_input();
                        }
                        return Some(index);
                    }
                }
            }
            Key { code: Enter, .. } => {
                // return current option index
                // but before returning, re-start concurrent user input again
                if let Some(instance) = input {
                    instance.start_concurrent_input();
                }

                return Some(current_option as usize);
            }
            _ => {}
        }
    }
}

/// Display a generic textbox with optional header and text.
pub fn msg_box(
    frontend: &mut GameFrontend,
    input: &mut Option<&mut GameInput>,
    header: &str,
    text: &str,
    width: i32,
) {
    // calculate total height for the header and text (after auto-wrap)
    let header_height = frontend
        .root
        .get_height_rect(0, 0, width, SCREEN_HEIGHT, header);
    let text_height = frontend
        .root
        .get_height_rect(0, 0, width, SCREEN_HEIGHT, text);

    let height = header_height + text_height + 2;

    // create an off-screen console that represents the menu's window
    let mut window = Offscreen::new(width, height);

    // set background and foreground colors
    for x in 0..width {
        for y in 0..height {
            window.set_char_background(x, y, frontend.coloring.bg_dialog, BackgroundFlag::Set);
            window.set_char_foreground(x, y, frontend.coloring.fg_dialog_border);
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

    window.set_default_foreground(frontend.coloring.fg_dialog);
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
        &mut frontend.root,
        (x, y),
        1.0,
        1.0,
    );

    // present the root console to the player and wait for a key-press
    frontend.root.flush();

    // if we have an instance of GameInput, pause the input listener thread first
    // so that we can receive input events directly
    match input {
        Some(ref mut handle) => {
            handle.pause_concurrent_input();
            frontend.root.wait_for_keypress(true);
            // after we got he key, restart input listener thread
            handle.resume_concurrent_input();
        }
        None => {
            frontend.root.wait_for_keypress(true);
        }
    }
}

pub fn show_character_screen(
    state: &mut GameState,
    frontend: &mut GameFrontend,
    input: &mut Option<&mut GameInput>,
    objects: &mut GameObjects,
) {
    if let Some(ref player) = objects[state.current_player_index] {
        let header: String = "Character Information".to_string();
        let msg: String = format!(
            "Energy:        {}/{} \n\
             Metabolism:    {} \n\
             Sense Range:   {} \n\
             HP:            {} \n\
             Alive:         {} \n\
             Turn:          {}",
            player.processors.energy,
            player.processors.energy_storage,
            player.processors.metabolism,
            player.sensors.sensing_range,
            player.actuators.max_hp,
            player.alive,
            state.turn
        );
        msg_box(frontend, input, &header, &msg, CHARACTER_SCREEN_WIDTH);
    };
}
