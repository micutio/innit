pub mod character;

use crate::game::{MENU_WIDTH, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::ui::color_palette::ColorPalette;
use rltk::{ColorPair, DrawBatch, Point, Rect, Rltk, VirtualKeyCode};

/// Non-click-away-able window menu.
#[derive(Clone)]
pub struct InfoBox {
    title: String,
    lines: Vec<String>,
    layout: Rect,
}

impl InfoBox {
    pub fn new(title: String, lines: Vec<String>) -> Self {
        let menu_width: i32 = if lines.is_empty() {
            MENU_WIDTH
        } else {
            i32::max(
                MENU_WIDTH,
                lines.iter().map(|l| l.len()).max().unwrap() as i32,
            )
        };
        let menu_height = lines.len() as i32 + 2;
        let x1 = (SCREEN_WIDTH / 2) - (menu_width / 2);
        let y1 = (SCREEN_HEIGHT / 2) - (menu_height / 2);
        let x2 = x1 + menu_width;
        let y2 = y1 + menu_height;
        InfoBox {
            title,
            lines,
            layout: Rect::with_exact(x1, y1, x2, y2),
        }
    }

    fn render(&self, palette: &ColorPalette) {
        let mut draw_batch = DrawBatch::new();
        // draw box
        draw_batch.draw_box(
            self.layout,
            ColorPair::new(palette.fg_dialog_border, palette.bg_dialog),
        );
        // draw title
        let title_pos = Point::new(self.layout.x1 + 3, self.layout.y1);
        draw_batch.print_color(
            title_pos,
            &self.title,
            ColorPair::new(palette.fg_dialog, palette.bg_dialog),
        );
        let title_open = Point::new(self.layout.x1 + 2, self.layout.y1);
        draw_batch.print_color(
            title_open,
            rltk::to_cp437('/'),
            ColorPair::new(palette.fg_dialog_border, palette.bg_dialog),
        );
        let title_close = Point::new(self.layout.x1 + 2 + self.title.len() as i32, self.layout.y1);
        draw_batch.print_color(
            title_close,
            rltk::to_cp437('/'),
            ColorPair::new(palette.fg_dialog_border, palette.bg_dialog),
        );

        for (index, line) in self.lines.iter().enumerate() {
            draw_batch.print_color(
                Point::new(self.layout.x1 + 1, self.layout.y1 + 1 + index as i32),
                line,
                ColorPair::new(palette.fg_dialog, palette.bg_dialog),
            );
        }

        draw_batch.submit(6000);
    }

    /// Main menu of the game.
    /// Display a background image and three options for the player to choose
    ///     - starting a new game
    ///     - loading an existing game
    ///     - quitting the game
    pub fn display(self, ctx: &mut Rltk, palette: &ColorPalette) -> Option<InfoBox> {
        // render current menu
        self.render(palette);

        // wait for user input
        // a) keyboard input
        // if we have a key activity, process and return immediately
        if let Some(VirtualKeyCode::Escape) = ctx.key {
            return None;
        }

        // b) mouse input
        if ctx.left_click && self.layout.point_in_rect(ctx.mouse_point()) {
            return None;
        }

        Some(self)
    }
}
