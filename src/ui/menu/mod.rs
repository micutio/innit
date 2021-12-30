pub mod choose_action;
pub mod credits;
pub mod game_over;
pub mod game_won;
pub mod main;

use crate::game::{self, State};
use crate::ui::hud::{ToolTip, UiItem};
use crate::{game::objects::ObjectStore, ui::palette};
use bracket_lib::prelude as rltk;

pub trait MenuItem: Clone {
    fn process(
        state: &mut State,
        objects: &mut ObjectStore,
        menu: &mut Menu<Self>,
        item: &Self,
    ) -> game::RunState;
}

/// Non-click-away-able window menu.
#[derive(Clone, Debug)]
pub struct Menu<T: MenuItem> {
    header: Option<String>,
    items: Vec<UiItem<T>>,
    selection: usize,
    layout: rltk::Rect,
}

impl<T: MenuItem> Menu<T> {
    pub fn new(item_vec: Vec<(T, String)>) -> Self {
        let x1 = (game::consts::SCREEN_WIDTH) - game::consts::MENU_WIDTH;
        let y1 = 0;
        let x2 = x1 + game::consts::MENU_WIDTH;
        let y2 = game::consts::SCREEN_HEIGHT;
        let item_y = 0;
        let items: Vec<UiItem<T>> = Menu::create_items(x1, item_y, item_vec);
        Menu {
            header: None,
            items,
            selection: 0,
            layout: rltk::Rect::with_exact(x1, y1, x2, y2),
        }
    }

    pub fn with_header(header: &str, item_vec: Vec<(T, String)>) -> Self {
        let x1 = (game::consts::SCREEN_WIDTH) - game::consts::MENU_WIDTH;
        let y1 = 0;
        let x2 = x1 + game::consts::MENU_WIDTH;
        let y2 = game::consts::SCREEN_HEIGHT;
        let item_y = 2;
        let items: Vec<UiItem<T>> = Menu::create_items(x1, item_y, item_vec);
        Menu {
            header: Some(header.into()),
            items,
            selection: 0,
            layout: rltk::Rect::with_exact(x1, y1, x2, y2),
        }
    }

    fn create_items(x1: i32, item_y: i32, item_vec: Vec<(T, String)>) -> Vec<UiItem<T>> {
        item_vec
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, (enum_item, text))| {
                UiItem::new(
                    enum_item,
                    text,
                    ToolTip::header_only(""),
                    rltk::Rect::with_size(
                        x1 + 1,
                        item_y + 1 + i as i32,
                        game::consts::MENU_WIDTH - 2,
                        1,
                    ),
                    rltk::ColorPair::new((0, 0, 0), (0, 0, 0)),
                )
            })
            .collect()
    }

    fn render(&self, ctx: &mut rltk::BTerm) {
        ctx.set_active_console(game::consts::HUD_CON);
        ctx.cls();
        let mut draw_batch = rltk::DrawBatch::new();
        let bg_menu = palette().hud_bg;
        let fg_menu = palette().hud_fg;
        let fg_menu_highlight = palette().hud_fg_highlight;
        draw_batch.fill_region(
            self.layout,
            rltk::ColorPair::new(fg_menu, bg_menu),
            rltk::to_cp437(' '),
        );
        for (index, item) in self.items.iter().enumerate() {
            let color = if index == self.selection {
                rltk::ColorPair::new(fg_menu_highlight, bg_menu)
            } else {
                rltk::ColorPair::new(fg_menu, bg_menu)
            };
            draw_batch.print_color(item.top_left_corner(), &item.text, color);
        }

        let fg_hud = palette().hud_fg;
        let bg_hud = palette().hud_bg;

        if let Some(head) = &self.header {
            draw_batch.print_color(
                rltk::Point::new(self.layout.x1 + 1, 1),
                head,
                rltk::ColorPair::new(fg_hud, bg_hud),
            );
        }

        // draw bottom line
        let btm_y = game::consts::SCREEN_HEIGHT - 1;
        draw_batch.fill_region(
            rltk::Rect::with_exact(7, btm_y, game::consts::SCREEN_WIDTH - 1, btm_y + 1),
            rltk::ColorPair::new(fg_hud, bg_hud),
            rltk::to_cp437(' '),
        );
        draw_batch.print_color(
            rltk::Point::new(9, btm_y),
            "Mobile Fluorescence Microscope",
            rltk::ColorPair::new(fg_hud, bg_hud),
        );
        draw_batch.submit(game::consts::HUD_CON_Z).unwrap();
    }

    /// Main menu of the game.
    /// Display a background image and three options for the player to choose
    ///     - starting a new game
    ///     - loading an existing game
    ///     - quitting the game
    pub fn display(&mut self, ctx: &mut rltk::BTerm) -> Option<T> {
        // render current menu
        self.render(ctx);

        // wait for user input
        // a) keyboard input
        // if we have a key activity, process and return immediately
        if let Some(key) = ctx.key {
            match key {
                rltk::VirtualKeyCode::Up => {
                    self.selection = (self.selection as i32 - 1) as usize % self.items.len();
                }
                rltk::VirtualKeyCode::Down => {
                    self.selection = (self.selection as i32 + 1) as usize % self.items.len();
                }
                rltk::VirtualKeyCode::Return => {
                    // return process_item(game, ctx, &self.items[self.selection].item_enum);
                    return Some(self.items[self.selection].item_enum.clone());
                }
                _ => {}
            }
        }

        // b) mouse input
        // if we have a mouse activity, check first for clicks, then for hovers
        if let Some(item) = self
            .items
            .iter()
            .find(|i| i.layout.point_in_rect(ctx.mouse_point()))
        {
            // update active index
            if let Some(index) = self.items.iter().position(|m| m.text.eq(&item.text)) {
                self.selection = index;
            }
            if ctx.left_click {
                return Some(self.items[self.selection].item_enum.clone());
            }
        }

        None
    }
}
