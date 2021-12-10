pub mod choose_action_menu;
pub mod game_over_menu;
pub mod main_menu;

use crate::core::game_state::GameState;
use crate::game::{RunState, HUD_CON, MENU_WIDTH, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::ui::hud::{ToolTip, UiItem};
use crate::{core::game_objects::GameObjects, ui::palette};
use rltk::{to_cp437, ColorPair, DrawBatch, Rect, Rltk, VirtualKeyCode};

pub trait MenuItem: Clone {
    fn process(
        state: &mut GameState,
        objects: &mut GameObjects,
        menu: &mut Menu<Self>,
        item: &Self,
    ) -> RunState;
}

/// Non-click-away-able window menu.
#[derive(Clone, Debug)]
pub struct Menu<T: MenuItem> {
    items: Vec<UiItem<T>>,
    selection: usize,
    layout: Rect,
}

impl<T: MenuItem> Menu<T> {
    pub fn new(item_vec: Vec<(T, String)>) -> Self {
        let menu_height = item_vec.len() as i32 + 2;
        let x1 = (SCREEN_WIDTH / 2) - (MENU_WIDTH / 2);
        let y1 = (SCREEN_HEIGHT / 2) - (menu_height / 2);
        let x2 = x1 + MENU_WIDTH;
        let y2 = y1 + menu_height - 1;
        let items: Vec<UiItem<T>> = item_vec
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, (enum_item, text))| {
                UiItem::new(
                    enum_item,
                    text,
                    ToolTip::header_only(""),
                    Rect::with_size(x1 + 1, y1 + 1 + i as i32, MENU_WIDTH - 2, 1),
                    ColorPair::new((0, 0, 0), (0, 0, 0)),
                )
            })
            .collect();
        Menu {
            items,
            selection: 0,
            layout: Rect::with_exact(x1, y1, x2, y2),
        }
    }

    fn render(&self, ctx: &mut Rltk) {
        ctx.set_active_console(HUD_CON);
        ctx.cls();
        let mut draw_batch = DrawBatch::new();
        let bg_menu = palette().hud_bg;
        let fg_menu = palette().hud_fg;
        let fg_menu_border = palette().hud_fg_border;
        let fg_menu_highlight = palette().hud_fg_highlight;
        draw_batch.fill_region(self.layout, ColorPair::new(fg_menu, bg_menu), to_cp437(' '));
        draw_batch.draw_hollow_box(self.layout, ColorPair::new(fg_menu_border, bg_menu));
        for (index, item) in self.items.iter().enumerate() {
            let color = if index == self.selection {
                ColorPair::new(fg_menu_highlight, bg_menu)
            } else {
                ColorPair::new(fg_menu, bg_menu)
            };
            draw_batch.print_color(item.top_left_corner(), &item.text, color);
        }

        draw_batch.submit(HUD_CON).unwrap();
    }

    /// Main menu of the game.
    /// Display a background image and three options for the player to choose
    ///     - starting a new game
    ///     - loading an existing game
    ///     - quitting the game
    pub fn display(&mut self, ctx: &mut Rltk) -> Option<T> {
        // render current menu
        self.render(ctx);

        // wait for user input
        // a) keyboard input
        // if we have a key activity, process and return immediately
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Up => {
                    self.selection = (self.selection as i32 - 1) as usize % self.items.len();
                }
                VirtualKeyCode::Down => {
                    self.selection = (self.selection as i32 + 1) as usize % self.items.len();
                }
                VirtualKeyCode::Return => {
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
