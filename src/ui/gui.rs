use crate::core::game_state::{GameState, MsgClass};
use crate::entity::action::Target;
use crate::entity::genetics::TraitFamily;
use crate::entity::object::Object;
use crate::game::{SCREEN_HEIGHT, SCREEN_WIDTH, SIDE_PANEL_HEIGHT, SIDE_PANEL_WIDTH};
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
                    Rect::with_size(button_x, 3, button_len, 0),
                ),
                UiItem::new(
                    HudItem::SecondaryAction,
                    String::new(),
                    Rect::with_size(button_x, 4, button_len, 0),
                ),
                UiItem::new(
                    HudItem::Quick1Action,
                    String::new(),
                    Rect::with_size(button_x, 5, button_len, 0),
                ),
                UiItem::new(
                    HudItem::Quick2Action,
                    String::new(),
                    Rect::with_size(button_x, 6, button_len, 0),
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
        ColorPair::new(cp.fg_hud, cp.bg_hud),
        rltk::to_cp437(' '),
    );

    // draw action header
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 2),
        "Actions",
        ColorPair::new(cp.fg_hud, cp.bg_hud),
    );

    // draw buttons
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 3),
        "P ",
        ColorPair::new(cp.fg_hud, cp.bg_hud),
    );
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 4),
        "S ",
        ColorPair::new(cp.fg_hud, cp.bg_hud),
    );
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 5),
        "Q1",
        ColorPair::new(cp.fg_hud, cp.bg_hud),
    );
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 6),
        "Q2",
        ColorPair::new(cp.fg_hud, cp.bg_hud),
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

        draw_batch.fill_region(
            Rect::with_size(item.layout.x1, item.layout.y1, SIDE_PANEL_WIDTH - 9, 0),
            ColorPair::new(cp.bg_hud_content, cp.bg_hud_content),
            to_cp437(' '),
        );
        draw_batch.print_color(
            item.top_left_corner(),
            text,
            ColorPair::new(cp.fg_hud, cp.bg_hud_content),
        );
    }

    // draw headers for vertical bars
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - 4, 2),
        '♥',
        ColorPair::new(cp.fg_hud, cp.bg_hud),
    );
    draw_batch.print(Point::new(SCREEN_WIDTH - 3, 2), '√');

    draw_batch.bar_vertical(
        Point::new(SCREEN_WIDTH - 4, 3),
        10,
        player.actuators.hp,
        player.actuators.max_hp,
        ColorPair::new(cp.magenta, cp.bg_hud_content),
    );
    draw_batch.bar_vertical(
        Point::new(SCREEN_WIDTH - 3, 3),
        10,
        player.processors.energy,
        player.processors.energy_storage,
        ColorPair::new(cp.yellow, cp.bg_bar),
    );

    render_dna(_ctx, cp, player, &mut draw_batch);

    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 13),
        "Inventory",
        ColorPair::new(cp.fg_hud, cp.bg_hud),
    );
    render_inventory(
        &mut draw_batch,
        Rect::with_exact(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 14, SCREEN_WIDTH - 2, 23),
        cp,
        player,
    );

    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 25),
        "Log",
        ColorPair::new(cp.fg_hud, cp.bg_hud),
    );
    render_log(
        state,
        &mut draw_batch,
        Rect::with_exact(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 26, SCREEN_WIDTH - 2, 59),
        cp,
    );

    draw_batch.submit(5000).unwrap();
}

fn render_dna(_ctx: &mut Rltk, cp: &ColorPalette, player: &Object, draw_batch: &mut DrawBatch) {
    draw_batch.fill_region(
        Rect::with_size(SCREEN_WIDTH - 1, 0, 0, SCREEN_HEIGHT - 1),
        ColorPair::new(cp.bg_dna, cp.bg_dna),
        to_cp437(' '),
    );
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - 5, 0),
        "DNA→",
        ColorPair::new(cp.fg_hud, cp.bg_dna),
    );
    for (vert_offset, g_trait) in player.dna.simplified.iter().enumerate() {
        let col: (u8, u8, u8) = match g_trait.trait_family {
            TraitFamily::Sensing => cp.cyan,
            TraitFamily::Processing => cp.magenta,
            TraitFamily::Actuating => cp.yellow,
            TraitFamily::Junk => (100, 100, 100), // TODO
            TraitFamily::Ltr => (255, 255, 255),  // TODO
        };

        let c: char = if modulus(vert_offset, 2) == 0 {
            '▼'
        } else {
            '▲'
        };
        draw_batch.print_color(
            Point::new(SCREEN_WIDTH - 1, vert_offset as i32),
            c,
            ColorPair::new(col, cp.bg_dna),
        );
    }
}

fn render_inventory(draw_batch: &mut DrawBatch, layout: Rect, cp: &ColorPalette, player: &Object) {
    draw_batch.fill_region(
        layout,
        ColorPair::new(cp.fg_hud, cp.bg_hud_content),
        to_cp437(' '),
    );

    for (idx, obj) in player.inventory.items.iter().enumerate() {
        if idx as i32 > layout.height() {
            break;
        }
        draw_batch.print_color(
            Point::new(layout.x1, layout.y1 + idx as i32),
            format!("{} ", obj.visual.glyph),
            ColorPair::new(obj.visual.fg_color, cp.bg_hud),
        );
        let name_chars: Vec<char> = obj.visual.name.chars().collect();
        let name_fitted: String = name_chars[..(layout.width() - 3) as usize]
            .to_vec()
            .iter()
            .collect();
        draw_batch.print_color(
            Point::new(layout.x1 + 3, layout.y1 + idx as i32),
            name_fitted,
            ColorPair::new(obj.visual.fg_color, cp.bg_hud),
        );
    }
}

fn render_log(state: &GameState, draw_batch: &mut DrawBatch, layout: Rect, cp: &ColorPalette) {
    draw_batch.fill_region(
        layout,
        ColorPair::new(cp.fg_hud, cp.bg_hud_content),
        to_cp437(' '),
    );

    // print game messages, one line at a time
    let mut y = layout.height();
    let mut bg_flag: bool = true;
    for (ref msg, class) in &mut state.log.iter().rev() {
        if y < 0 {
            break;
        }

        // set message color depending in message class
        let fg_color = match class {
            MsgClass::Alert => cp.msg_alert,
            MsgClass::Info => cp.msg_info,
            MsgClass::Action => cp.msg_action,
            MsgClass::Story => cp.msg_story,
        };

        // set alternating background color to make the log more readable
        let bg_color = if bg_flag {
            cp.bg_hud_log1
        } else {
            cp.bg_hud_log2
        };
        bg_flag = !bg_flag;

        let msg_lines = format_message(msg, layout.width());
        let msg_start_y = layout.y1 + y + 1 - msg_lines.len() as i32;

        // message background
        draw_batch.fill_region(
            Rect::with_exact(
                layout.x1,
                msg_start_y,
                layout.x2,
                msg_start_y - 1 + msg_lines.len() as i32,
            ),
            ColorPair::new(bg_color, bg_color),
            to_cp437(' '),
        );

        for (idx, line) in msg_lines.iter().enumerate() {
            draw_batch.print_color(
                Point::new(layout.x1, msg_start_y + idx as i32),
                line,
                ColorPair::new(fg_color, bg_color),
            );
        }

        y -= msg_lines.len() as i32;
    }
}

fn format_message(text: &str, line_width: i32) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current_line: Vec<&str> = Vec::new();
    let mut current_width = 0;
    for word in text.split(' ') {
        current_width += word.len() + 1;
        if current_width <= line_width as usize + 1 {
            current_line.push(word);
        } else {
            lines.push(current_line.join(" "));
            current_line.clear();
            current_line.push(word);
            current_width = word.len() + 1;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line.join(" "));
    }
    lines
}
