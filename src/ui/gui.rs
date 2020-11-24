use crate::game::Game;
use rltk::{ColorPair, DrawBatch, Point, Rect, Rltk};

/// Menu item properties
/// - `text` for rendering
/// - `layout` for checking mouse interaction
/// - `prev menu item` for cycling through via keys (use vector indices!)
/// - `next menu item` for cycling through via keys (use vector indices!)
pub struct UiItem<T> {
    pub item_enum: T,
    pub text: String,
    layout: Rect,
}

impl<T> UiItem<T> {
    pub fn new(item_enum: T, text: &str, layout: Rect) -> Self {
        UiItem {
            item_enum,
            text: text.to_string(),
            layout,
        }
    }

    pub fn top_left_corner(&self) -> Point {
        Point::new(self.layout.x1, self.layout.y1)
    }

    // TODO: Read up on what static lifetimes are!
    pub fn get_active_item(
        items: &'static Vec<UiItem<T>>,
        mouse_pos: Point,
    ) -> Option<&'static UiItem<T>> {
        items.iter().find(|i| i.layout.point_in_rect(mouse_pos))
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
    pub(crate) items: Vec<UiItem<HudItem>>,
    names_under_mouse: String,
}

impl Hud {
    pub fn new(layout: Rect) -> Self {
        Hud {
            layout,
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
pub fn render_gui(game: &mut Game, ctx: &mut Rltk, hud: &mut Hud) {
    let mut draw_batch = DrawBatch::new();

    // draw buttons
    let mut draw_batch = DrawBatch::new();
    draw_batch.draw_box(
        hud.layout,
        ColorPair::new(game.color_palette.fg_dialog, game.color_palette.bg_dialog),
    );
    for item in hud.items {
        draw_batch.print_color(
            item.top_left_corner(),
            &item.text,
            ColorPair::new(game.color_palette.fg_dialog, game.color_palette.bg_dialog),
        );
    }

    draw_batch.submit(6000);

    // draw ui boxes
    if let Some(player) = game
        .objects
        .extract_by_index(game.state.current_player_index)
    {}
    draw_batch.submit(5000);
}
