pub mod character;

use crate::game::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::ui::color_palette::ColorPalette;
use rltk::{to_cp437, ColorPair, DrawBatch, Point, Rect, Rltk, VirtualKeyCode};

/// Non-click-away-able window menu.
#[derive(Clone, Debug)]
pub struct InfoBox {
    title: String,
    lines: Vec<String>,
    layout: Rect,
}

impl InfoBox {
    pub fn new(title: String, lines: Vec<String>) -> Self {
        let box_width: i32 = usize::max(
            title.len() + 3,
            lines.iter().map(|l| l.len()).max().unwrap(),
        ) as i32;
        let box_height = lines.len() as i32 + 1;
        let x1 = (SCREEN_WIDTH / 2) - (box_width / 2);
        let y1 = (SCREEN_HEIGHT / 2) - (box_height / 2);
        let x2 = x1 + box_width;
        let y2 = y1 + box_height;
        InfoBox {
            title,
            lines,
            layout: Rect::with_exact(x1, y1, x2, y2),
        }
    }

    fn render(&self, palette: &ColorPalette) {
        let mut draw_batch = DrawBatch::new();
        // draw box
        draw_batch.fill_region(
            self.layout,
            ColorPair::new(palette.fg_hud_border, palette.bg_hud),
            to_cp437(' '),
        );
        draw_batch.draw_hollow_box(self.layout, ColorPair::new(palette.fg_hud, palette.bg_hud));

        // draw title
        let title_pos = Point::new(self.layout.x1 + 2, self.layout.y1);
        draw_batch.print_color(
            title_pos,
            &self.title,
            ColorPair::new(palette.fg_hud_border, palette.bg_hud),
        );

        for (index, line) in self.lines.iter().enumerate() {
            draw_batch.print_color(
                Point::new(self.layout.x1 + 1, self.layout.y1 + 1 + index as i32),
                line,
                ColorPair::new(palette.fg_hud, palette.bg_hud),
            );
        }

        draw_batch.submit(6000).unwrap();
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
        if ctx.left_click && !self.layout.point_in_rect(ctx.mouse_point()) {
            return None;
        }

        Some(self)
    }
}
