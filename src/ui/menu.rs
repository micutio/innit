use crate::game::{Game, RunState, MENU_WIDTH, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::ui::color_palette::ColorPalette;
use crate::ui::gui::UiItem;
use crate::util::modulus;
use rltk::{ColorPair, DrawBatch, Rect, Rltk, VirtualKeyCode};

pub trait MenuItem: Clone {
    fn process(game: &mut Game, ctx: &mut Rltk, menu: &mut Menu<Self>, item: &Self) -> RunState;
}

/// Non-click-away-able window menu.
#[derive(Clone)]
pub struct Menu<T: MenuItem> {
    items: Vec<UiItem<T>>,
    selection: usize,
    layout: Rect,
}

impl<T: MenuItem> Menu<T> {
    pub fn new(item_vec: Vec<(T, &str)>) -> Self {
        let menu_height = item_vec.len() as i32 + 2;
        let x1 = (SCREEN_WIDTH / 2) - (MENU_WIDTH / 2);
        let y1 = (SCREEN_HEIGHT / 2) - (menu_height / 2);
        let x2 = x1 + MENU_WIDTH;
        let y2 = y1 + menu_height;
        let items: Vec<UiItem<T>> = item_vec
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, (enum_item, text))| {
                UiItem::new(
                    enum_item,
                    text,
                    Rect::with_size(x1 + 1, y1 + 1 + i as i32, MENU_WIDTH - 2, 1),
                )
            })
            .collect();
        Menu {
            items,
            selection: 0,
            layout: Rect::with_exact(x1, y1, x2, y2),
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

    /// Main menu of the game.
    /// Display a background image and three options for the player to choose
    ///     - starting a new game
    ///     - loading an existing game
    ///     - quitting the game
    pub fn display(&mut self, ctx: &mut Rltk, palette: &ColorPalette) -> Option<T> {
        // render current menu
        self.render(palette);

        // wait for user input
        // a) keyboard input
        // if we have a key activity, process and return immediately
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Up => {
                    self.selection = modulus(self.selection - 1, self.items.len());
                }
                VirtualKeyCode::Down => {
                    self.selection = modulus(self.selection + 1, self.items.len());
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
