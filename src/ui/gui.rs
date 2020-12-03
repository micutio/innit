use crate::entity::action::Target;
use crate::entity::genetics::TraitFamily;
use crate::entity::object::Object;
use crate::game::{SCREEN_WIDTH, SIDE_PANEL_HEIGHT, SIDE_PANEL_WIDTH};
use crate::ui::color_palette::ColorPalette;
use crate::util::modulus;
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
    Quick2Action,
}

pub struct Hud {
    layout: Rect,
    pub items: Vec<UiItem<HudItem>>,
    names_under_mouse: String, // TODO: Find elegant way to render this and tooltips.
}

impl Hud {
    pub fn new() -> Self {
        let x1 = SCREEN_WIDTH - SIDE_PANEL_WIDTH - 1;
        let y1 = 0;
        let x2 = x1 + SIDE_PANEL_WIDTH;
        let y2 = SIDE_PANEL_HEIGHT - 1;
        let button_len = (SIDE_PANEL_WIDTH / 2) - 3;
        let button_x = SCREEN_WIDTH - (SIDE_PANEL_WIDTH / 2);
        Hud {
            layout: Rect::with_exact(x1, y1, x2, y2),
            items: vec![
                UiItem::new(
                    HudItem::PrimaryAction,
                    String::new(),
                    Rect::with_size(button_x, 3, button_len, 1),
                ),
                UiItem::new(
                    HudItem::SecondaryAction,
                    String::new(),
                    Rect::with_size(button_x, 4, button_len, 1),
                ),
                UiItem::new(
                    HudItem::Quick1Action,
                    String::new(),
                    Rect::with_size(button_x, 5, button_len, 1),
                ),
                UiItem::new(
                    HudItem::Quick2Action,
                    String::new(),
                    Rect::with_size(button_x, 6, button_len, 1),
                ),
            ],
            names_under_mouse: "".to_string(),
        }
    }

    pub fn set_names_under_mouse(&mut self, names: String) {
        self.names_under_mouse = names;
    }
}

// TODO: Keep track of UI elements for mouse detection purposes.
// TODO: Create gui struct to hold elements, hold parallel to game struct.
pub fn render_gui(hud: &mut Hud, _ctx: &mut Rltk, color_palette: &ColorPalette, player: &Object) {
    // draw buttons
    let mut draw_batch = DrawBatch::new();
    draw_batch.fill_region(
        hud.layout,
        ColorPair::new(color_palette.fg_dialog, color_palette.bg_dialog),
        rltk::to_cp437(' '),
    );

    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH + 1, 3),
        "Primary Action".to_string(),
        ColorPair::new(color_palette.fg_dialog, color_palette.bg_dialog),
    );
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH + 1, 4),
        "Secondary Action".to_string(),
        ColorPair::new(color_palette.fg_dialog, color_palette.bg_dialog),
    );
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH + 1, 5),
        "Quick Action 1".to_string(),
        ColorPair::new(color_palette.fg_dialog, color_palette.bg_dialog),
    );
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH + 1, 6),
        "Quick Action 2".to_string(),
        ColorPair::new(color_palette.fg_dialog, color_palette.bg_dialog),
    );

    for item in &hud.items {
        let text = match item.item_enum {
            HudItem::PrimaryAction => player.get_primary_action(Target::Center).get_identifier(),
            HudItem::SecondaryAction => {
                player.get_secondary_action(Target::Center).get_identifier()
            }
            HudItem::Quick1Action => player.get_quick1_action().get_identifier(),
            HudItem::Quick2Action => player.get_quick2_action().get_identifier(),
        };

        draw_batch.print_color(
            item.top_left_corner(),
            text,
            ColorPair::new(color_palette.fg_dialog, color_palette.bg_dialog_selected),
        );
    }

    render_dna(_ctx, color_palette, player, &mut draw_batch);

    draw_batch.submit(5000).unwrap();
}

fn render_dna(
    _ctx: &mut Rltk,
    color_palette: &ColorPalette,
    player: &Object,
    draw_batch: &mut DrawBatch,
) {
    for (vert_offset, g_trait) in player.dna.simplified.iter().enumerate() {
        let col: (u8, u8, u8) = match g_trait.trait_family {
            TraitFamily::Sensing => color_palette.cyan,
            TraitFamily::Processing => color_palette.magenta,
            TraitFamily::Actuating => color_palette.yellow,
            TraitFamily::Junk => (100, 100, 100), // TODO
            TraitFamily::Ltr => (255, 255, 255),  // TODO
        };

        let c: char = if modulus(vert_offset, 2) == 0 {
            '▼'
        } else {
            '▲'
        };
        draw_batch.print_color(
            Point::new(SCREEN_WIDTH - 1, (vert_offset as i32) + 2),
            c,
            ColorPair::new(col, color_palette.bg_dialog),
        );
    }
}
