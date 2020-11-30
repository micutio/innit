use crate::game::{SCREEN_WIDTH, SIDE_PANEL_HEIGHT, SIDE_PANEL_WIDTH};
use crate::ui::color_palette::ColorPalette;
use rltk::{ColorPair, DrawBatch, Point, Rect, Rltk};

/// Menu item properties
/// - `text` for rendering
/// - `layout` for checking mouse interaction
/// - `prev menu item` for cycling through via keys (use vector indices!)
/// - `next menu item` for cycling through via keys (use vector indices!)
#[derive(Clone, Debug)]
pub struct UiItem<T> {
    pub item_enum: T,
    pub text: String,
    pub(crate) layout: Rect,
}

impl<T> UiItem<T> {
    pub fn new(item_enum: T, text: String, layout: Rect) -> Self {
        UiItem {
            item_enum,
            text,
            layout,
        }
    }

    pub fn top_left_corner(&self) -> Point {
        Point::new(self.layout.x1, self.layout.y1)
    }
}

pub enum HudItem {
    PrimaryAction,
    SecondaryAction,
    Quick1Action,
    QuickAction2,
}

pub struct Hud {
    layout: Rect,
    pub items: Vec<UiItem<HudItem>>,
    names_under_mouse: String,
}

impl Hud {
    pub fn new() -> Self {
        let x1 = SCREEN_WIDTH - SIDE_PANEL_WIDTH - 1;
        let y1 = 0;
        let x2 = x1 + SIDE_PANEL_WIDTH;
        let y2 = SIDE_PANEL_HEIGHT - 1;
        Hud {
            layout: Rect::with_exact(x1, y1, x2, y2),
            items: Vec::new(),
            names_under_mouse: "".to_string(),
        }
    }

    pub fn set_names_under_mouse(&mut self, names: String) {
        self.names_under_mouse = names;
    }
}

// TODO: Keep track of UI elements for mouse detection purposes.
// TODO: Create gui struct to hold elements, hold parallel to game struct.
pub fn render_gui(hud: &mut Hud, _ctx: &mut Rltk, color_palette: &ColorPalette) {
    // draw buttons
    let mut draw_batch = DrawBatch::new();
    // draw_batch.dra
    draw_batch.draw_box(
        hud.layout,
        ColorPair::new(color_palette.fg_dialog, color_palette.bg_dialog),
    );
    for item in hud.items {
        draw_batch.print_color(
            item.top_left_corner(),
            &item.text,
            ColorPair::new(color_palette.fg_dialog, color_palette.bg_dialog),
        );
    }

    // draw_batch.submit(6000);
    //
    // // TODO: draw ui boxes
    // if let Some(player) = game
    //     .objects
    //     .extract_by_index(game.state.current_player_index)
    // {}
    // draw_batch.submit(5000);
}
