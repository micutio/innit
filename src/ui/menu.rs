use crate::game::{load_game, Game, RunState};
use crate::ui::color_palette::ColorPalette;
use crate::ui::gui::UiItem;
use crate::util::modulus;
use rltk::{ColorPair, DrawBatch, Rect, Rltk, VirtualKeyCode};

pub enum MenuInstance {
    MainMenu(Option<MainMenu>),
}

pub fn display_menu(
    game: &mut Game,
    ctx: &mut Rltk,
    palette: &ColorPalette,
    instance: MenuInstance,
) -> RunState {
    match instance {
        MenuInstance::MainMenu(instance) => display_main_menu(game, ctx, palette, instance),
    }
}

enum MainMenuOption {
    NewGame,
    Resume,
    // Controls,
    // Options,
    Quit,
}

/// Non-click-away-able window menu.
pub struct MainMenu {
    items: Vec<UiItem<MainMenuOption>>,
    selection: usize,
    layout: Rect,
}

impl MainMenu {
    fn new() -> Self {
        MainMenu {
            items: vec![
                UiItem::new(
                    MainMenuOption::NewGame,
                    "New Game",
                    Rect::with_size(10, 20, 10, 1),
                ),
                UiItem::new(
                    MainMenuOption::Resume,
                    "Resume last Game",
                    Rect::with_size(10, 21, 10, 1),
                ),
                UiItem::new(MainMenuOption::Quit, "Quit", Rect::with_size(10, 22, 10, 1)),
            ],
            selection: 0,
            layout: Rect::with_size(9, 19, 12, 5),
        }
    }

    fn render(&self, palette: &ColorPalette) {
        let mut draw_batch = DrawBatch::new();
        draw_batch.draw_box(
            self.layout,
            ColorPair::new(palette.fg_dialog, palette.bg_dialog),
        );
        for (index, item) in self.items.iter().enumerate() {
            let color = if index == self.selection {
                ColorPair::new(palette.fg_dialog, palette.bg_dialog)
            } else {
                ColorPair::new(palette.fg_dialog, palette.bg_dialog.desaturate())
            };
            draw_batch.print_color(item.top_left_corner(), &item.text, color);
        }

        draw_batch.submit(6000);
    }

    // TODO: Implement re-usable input processing:
    //       - cycling through items with keys
    //       - hovering items with mouse
    //       - clicking items with mouse
}

/// Main menu of the game.
/// Display a background image and three options for the player to choose
///     - starting a new game
///     - loading an existing game
///     - quitting the game
pub fn display_main_menu(
    game: &mut Game,
    ctx: &mut Rltk,
    palette: &ColorPalette,
    instance: Option<MainMenu>,
) -> RunState {
    let mut main_menu = match instance {
        Some(menu) => menu,
        None => MainMenu::new(),
    };

    // render current menu
    main_menu.render(palette);

    // wait for user input

    // a) keyboard input
    // if we have a key activity, process and return immediately
    if let Some(key) = ctx.key {
        match key {
            VirtualKeyCode::Up => {
                main_menu.selection = modulus(main_menu.selection - 1, main_menu.items.len());
            }
            VirtualKeyCode::Down => {
                main_menu.selection = modulus(main_menu.selection + 1, main_menu.items.len());
            }
            VirtualKeyCode::Return => {
                return process_item(game, ctx, &main_menu.items[main_menu.selection].item_enum);
            }
            _ => {}
        }
    }

    // b) mouse input
    // if we have a mouse activity, check first for clicks, then for hovers
    if let Some(item) = UiItem::get_active_item(&main_menu.items, ctx.mouse_point()) {
        if ctx.left_click {
            return process_item(game, ctx, &item.item_enum);
        } else {
            // update active index
            if let Some(index) = main_menu.items.iter().position(|m| m.text.eq(&item.text)) {
                main_menu.selection = index;
            }
        }
    }

    RunState::Menu(MenuInstance::MainMenu(Some(main_menu)))
}

fn process_item(game: &mut Game, ctx: &mut Rltk, item: &MainMenuOption) -> RunState {
    match item {
        MainMenuOption::NewGame => {
            // start new game
            let (mut state, mut objects) = Game::new_game(game.state.env, ctx);
            game.reset(state, objects);
            RunState::Ticking
            // game_loop(&mut state, frontend, &mut input, &mut objects);
        }
        MainMenuOption::Resume => {
            // load game from file
            match load_game() {
                Ok((mut state, mut objects)) => {
                    game.reset(state, objects);
                    RunState::Ticking
                }
                Err(_e) => {
                    // TODO: Show alert to user... or not?
                    // msg_box(frontend, &mut None, "", "\nNo saved game to load\n", 24);
                    RunState::Menu(MenuInstance::MainMenu(None))
                }
            }
        }
        MainMenuOption::Quit => {
            std::process::exit(0);
        }
    }
}
