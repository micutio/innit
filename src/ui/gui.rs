use crate::core::game_state::{GameState, MsgClass};
use crate::entity::action::Target;
use crate::entity::genetics::TraitFamily;
use crate::entity::object::Object;
use crate::game::{SCREEN_WIDTH, SIDE_PANEL_HEIGHT, SIDE_PANEL_WIDTH};
use crate::ui::color_palette::ColorPalette;
use crate::util::modulus;
use rltk::{to_cp437, ColorPair, DrawBatch, Point, Rect, Rltk};

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
        let button_len = (SIDE_PANEL_WIDTH / 2) - 4;
        let button_x = x1 + 4;
        Hud {
            layout: Rect::with_exact(x1, y1, x2, y2),
            items: vec![
                UiItem::new(
                    HudItem::PrimaryAction,
                    String::new(),
                    Rect::with_size(button_x, 2, button_len, 1),
                ),
                UiItem::new(
                    HudItem::SecondaryAction,
                    String::new(),
                    Rect::with_size(button_x, 3, button_len, 1),
                ),
                UiItem::new(
                    HudItem::Quick1Action,
                    String::new(),
                    Rect::with_size(button_x, 4, button_len, 1),
                ),
                UiItem::new(
                    HudItem::Quick2Action,
                    String::new(),
                    Rect::with_size(button_x, 5, button_len, 1),
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
pub fn render_gui(
    state: &GameState,
    hud: &mut Hud,
    _ctx: &mut Rltk,
    cp: &ColorPalette,
    player: &Object,
) {
    let mut draw_batch = DrawBatch::new();

    // fill side panel background
    draw_batch.fill_region(
        hud.layout,
        ColorPair::new(cp.fg_dialog, cp.bg_dialog),
        rltk::to_cp437(' '),
    );

    // draw action header
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 1),
        "Actions",
        ColorPair::new(cp.fg_dialog, cp.bg_dialog),
    );

    // draw buttons
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 2),
        "P ".to_string(),
        ColorPair::new(cp.fg_dialog, cp.bg_dialog),
    );
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 3),
        "S ".to_string(),
        ColorPair::new(cp.fg_dialog, cp.bg_dialog),
    );
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 4),
        "Q1".to_string(),
        ColorPair::new(cp.fg_dialog, cp.bg_dialog),
    );
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 5),
        "Q2".to_string(),
        ColorPair::new(cp.fg_dialog, cp.bg_dialog),
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
            ColorPair::new(cp.fg_dialog, cp.bg_dialog_selected),
        );
    }

    // draw headers for vertical bars
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - 3, 1),
        '♥',
        ColorPair::new(cp.fg_dialog, cp.bg_dialog),
    );
    draw_batch.print(Point::new(SCREEN_WIDTH - 2, 1), '√');
    draw_batch.print(Point::new(SCREEN_WIDTH - 1, 1), '☼');

    draw_batch.bar_vertical(
        Point::new(SCREEN_WIDTH - 3, 2),
        10,
        player.actuators.hp,
        player.actuators.max_hp,
        ColorPair::new(cp.magenta, cp.bg_bar),
    );
    draw_batch.bar_vertical(
        Point::new(SCREEN_WIDTH - 2, 2),
        10,
        player.processors.energy,
        player.processors.energy_storage,
        ColorPair::new(cp.yellow, cp.bg_bar),
    );

    render_dna(_ctx, cp, player, &mut draw_batch);

    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 13),
        "Inventory",
        ColorPair::new(cp.fg_dialog_highlight, cp.bg_dialog),
    );
    render_inventory(
        &mut draw_batch,
        Rect::with_exact(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 14, SCREEN_WIDTH - 2, 22),
        cp,
        player,
    );

    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 25),
        "Log",
        ColorPair::new(cp.fg_dialog_highlight, cp.bg_dialog),
    );
    render_log(
        state,
        &mut draw_batch,
        Rect::with_exact(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 26, SCREEN_WIDTH - 2, 57),
        cp,
    );

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

fn render_inventory(draw_batch: &mut DrawBatch, layout: Rect, cp: &ColorPalette, player: &Object) {
    draw_batch.fill_region(
        layout.clone(),
        ColorPair::new(cp.fg_dialog, cp.bg_dialog_selected),
        to_cp437(' '),
    );

    for (idx, obj) in player.inventory.items.iter().enumerate() {
        if idx as i32 > layout.height() {
            break;
        }
        draw_batch.print_color(
            Point::new(layout.x1, layout.y1 + idx as i32),
            format!("{} ", obj.visual.glyph),
            ColorPair::new(obj.visual.fg_color, cp.bg_dialog),
        );
        let name_chars: Vec<char> = obj.visual.name.chars().collect();
        let name_fitted: String = name_chars[..(layout.width() - 3) as usize]
            .to_vec()
            .iter()
            .collect();
        draw_batch.print_color(
            Point::new(layout.x1 + 3, layout.y1 + idx as i32),
            format!("{}", name_fitted),
            ColorPair::new(obj.visual.fg_color, cp.bg_dialog),
        );
    }
}

fn render_log(state: &GameState, draw_batch: &mut DrawBatch, layout: Rect, cp: &ColorPalette) {
    draw_batch.fill_region(
        layout,
        ColorPair::new(cp.fg_dialog, cp.bg_dialog_selected),
        to_cp437(' '),
    );

    // print game messages, one line at a time
    let mut y = layout.height();
    for (ref msg, class) in &mut state.log.iter().rev() {
        if y < 1 {
            break;
        }

        // TODO: Use custom color scheme instead.
        let color = match class {
            MsgClass::Alert => cp.msg_alert,
            MsgClass::Info => cp.msg_info,
            MsgClass::Action => cp.msg_action,
            MsgClass::Story => cp.msg_story,
        };

        let mut text_width = 0;
        for c in msg.chars() {
            draw_batch.print_color(
                Point::new(layout.x1 + text_width, layout.y1 + y),
                c,
                ColorPair::new(color, cp.bg_dialog_selected),
            );
            text_width += 1;
            if text_width >= layout.width() {
                y -= 1;
                text_width = 0;
            }
        }
    }
}
