//! The GUI in Innit consists of the sidebar and any tooltips appearing over objects on the map or
//! UI elements.
//!
//! TODO: Draft data structure for tooltips and generate them more selectively, not for empty tiles.
//! - varying visibility and detail, depending on perception (add to genetic traits)
//! - potential structure:
//!   - header
//!   - table with attributes and values:
//!     - hp
//!     - energy
//!     - receptor and whether it's matching with us

use crate::core::game_state::{GameState, MsgClass};
use crate::entity::action::Target;
use crate::entity::genetics::{GeneticTrait, TraitFamily};
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
    pub tooltip: ToolTip,
    pub layout: Rect,
    pub color: ColorPair,
}

impl<T> UiItem<T> {
    pub fn new<S1: Into<String>>(
        item_enum: T,
        text: S1,
        tooltip: ToolTip,
        layout: Rect,
        color: ColorPair,
    ) -> Self {
        UiItem {
            item_enum,
            text: text.into(),
            tooltip,
            layout,
            color,
        }
    }

    pub fn top_left_corner(&self) -> Point {
        Point::new(self.layout.x1, self.layout.y1)
    }
}

#[derive(PartialEq)]
pub enum HudItem {
    PrimaryAction,
    SecondaryAction,
    Quick1Action,
    Quick2Action,
    DnaItem,
}

fn create_hud_items(hud_layout: &Rect, cp: &ColorPalette) -> Vec<UiItem<HudItem>> {
    let button_len = SIDE_PANEL_WIDTH / 2;
    let button_x = hud_layout.x1 + 3;
    let items = vec![
        UiItem::new(
            HudItem::PrimaryAction,
            "",
            ToolTip::header_only("select new primary action"),
            Rect::with_size(button_x, 6, button_len, 1),
            ColorPair::new(cp.fg_hud, cp.bg_hud_content),
        ),
        UiItem::new(
            HudItem::SecondaryAction,
            "",
            ToolTip::header_only("select new secondary action"),
            Rect::with_size(button_x, 7, button_len, 1),
            ColorPair::new(cp.fg_hud, cp.bg_hud_content),
        ),
        UiItem::new(
            HudItem::Quick1Action,
            "",
            ToolTip::header_only("select new quick action"),
            Rect::with_size(button_x, 8, button_len, 1),
            ColorPair::new(cp.fg_hud, cp.bg_hud_content),
        ),
        UiItem::new(
            HudItem::Quick2Action,
            "",
            ToolTip::header_only("select new quick action"),
            Rect::with_size(button_x, 9, button_len, 1),
            ColorPair::new(cp.fg_hud, cp.bg_hud_content),
        ),
    ];

    items
}

#[derive(Clone, Debug)]
pub struct ToolTip {
    header: Option<String>,
    attributes: Vec<(String, String)>,
}

impl ToolTip {
    fn new<S1: Into<String>>(header: S1, attrs: Vec<(String, String)>) -> Self {
        ToolTip {
            header: Some(header.into()),
            attributes: attrs
                .iter()
                .map(|(e1, e2)| (e1.into(), e2.into()))
                .collect(),
        }
    }

    fn no_header(attrs: Vec<(String, String)>) -> Self {
        ToolTip {
            header: None,
            attributes: attrs
                .iter()
                .map(|(e1, e2)| (e1.into(), e2.into()))
                .collect(),
        }
    }

    fn header_only<S1: Into<String>>(header: S1) -> Self {
        ToolTip {
            header: Some(header.into()),
            attributes: Vec::new(),
        }
    }

    /// Calculate the width in `[cells]` that a text box will require to be rendered on screen.
    fn render_width(&self) -> i32 {
        let header_width: usize = if let Some(h) = &self.header {
            h.len()
        } else {
            0
        };

        let attributes_width: usize = self
            .attributes
            .iter()
            .map(|(s1, s2)| s1.len() + s2.len())
            .max()
            .unwrap_or(0);

        // pad header with 2 and attributes with 3 to account for borders and separators
        (header_width + 2).max(attributes_width + 3) as i32
    }

    /// Calculate the height in `[cells]` that a text box will require to be rendered on screen.
    fn render_height(&self) -> i32 {
        let header_height = if self.header.is_some() { 1 } else { 0 };

        let attributes_height = self.attributes.len();

        // pad height with 3 cells to account for rendering borders
        (header_height + attributes_height) as i32 + 3
    }
}

// fn tooltip_from(g_trait: &GeneticTrait) -> Vec<String> {
//     vec![
//         format!("trait: {}", g_trait.trait_name),
//         format!("group: {}", g_trait.trait_family),
//     ]
// }

pub struct Hud {
    layout: Rect,
    last_mouse: Point,
    pub require_refresh: bool,
    pub items: Vec<UiItem<HudItem>>,
    tooltips: Vec<ToolTip>, // TODO: Find elegant way to render this and tooltips.
}

impl Hud {
    pub fn new(cp: &ColorPalette) -> Self {
        let x1 = SCREEN_WIDTH - SIDE_PANEL_WIDTH - 1;
        let y1 = 0;
        let x2 = x1 + SIDE_PANEL_WIDTH;
        let y2 = SIDE_PANEL_HEIGHT - 1;
        let layout = Rect::with_exact(x1, y1, x2, y2);
        Hud {
            layout,
            last_mouse: Point::new(0, 0),
            require_refresh: false,
            items: create_hud_items(&layout, cp),
            tooltips: Vec::new(),
        }
    }

    pub fn update_tooltips(&mut self, mouse_pos: Point, names: Vec<String>) {
        if let Some(item) = self
            .items
            .iter()
            .find(|i| i.layout.point_in_rect(mouse_pos))
        {
            self.tooltips.push(item.tooltip.clone());
        } else {
            self.tooltip = names;
        }

        self.require_refresh = self.last_mouse != mouse_pos;
        self.last_mouse = mouse_pos;
    }

    pub fn update_ui_items(&mut self, player: &Object, cp: &ColorPalette) {
        self.items.retain(|i| i.item_enum != HudItem::DnaItem);

        for (h_offset, g_trait) in player
            .dna
            .simplified
            .iter()
            .take(SIDE_PANEL_WIDTH as usize - 4)
            .enumerate()
        {
            let col: (u8, u8, u8) = match g_trait.trait_family {
                TraitFamily::Sensing => cp.cyan,
                TraitFamily::Processing => cp.magenta,
                TraitFamily::Actuating => cp.yellow,
                TraitFamily::Junk => (100, 100, 100), // TODO
                TraitFamily::Ltr => (255, 255, 255),  // TODO
            };

            let c: char = if modulus(h_offset, 2) == 0 {
                '►'
            } else {
                '◄'
            };

            let tooltip = ToolTip::no_header(vec![
                ("trait:".to_string(), g_trait.trait_name.clone()),
                ("group:".to_string(), g_trait.trait_family.to_string()),
            ]);

            self.items.push(UiItem::new(
                HudItem::DnaItem,
                c,
                tooltip,
                Rect::with_size(
                    SCREEN_WIDTH - SIDE_PANEL_WIDTH + 3 + h_offset as i32,
                    0,
                    1,
                    1,
                ),
                ColorPair::new(col, cp.bg_dna),
            ));
        }

        for (v_offset, g_trait) in player
            .dna
            .simplified
            .iter()
            .skip(SIDE_PANEL_WIDTH as usize - 4)
            .enumerate()
        {
            let col: (u8, u8, u8) = match g_trait.trait_family {
                TraitFamily::Sensing => cp.cyan,
                TraitFamily::Processing => cp.magenta,
                TraitFamily::Actuating => cp.yellow,
                TraitFamily::Junk => (100, 100, 100), // TODO
                TraitFamily::Ltr => (255, 255, 255),  // TODO
            };

            let c: char = if modulus(v_offset, 2) == 0 {
                '▼'
            } else {
                '▲'
            };

            let tooltip = ToolTip::no_header(vec![
                ("trait:".to_string(), g_trait.trait_name.clone()),
                ("group:".to_string(), g_trait.trait_family.to_string()),
            ]);

            self.items.push(UiItem::new(
                HudItem::DnaItem,
                c,
                tooltip,
                Rect::with_size(SCREEN_WIDTH - 1, v_offset as i32, 1, 1),
                ColorPair::new(col, cp.bg_dna),
            ));
        }
    }
}

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

    let inv_area = Rect::with_exact(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 12, SCREEN_WIDTH - 2, 22);
    let log_area = Rect::with_exact(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 25, SCREEN_WIDTH - 2, 58);

    render_dna_region(cp, &mut draw_batch);
    render_bars(player, cp, &mut draw_batch);
    render_action_fields(player, hud, cp, &mut draw_batch);
    render_inventory(player, inv_area, cp, &mut draw_batch);
    render_log(state, log_area, cp, &mut draw_batch);
    render_ui_items(hud, &mut draw_batch);
    render_tooltip(hud, cp, &mut draw_batch);

    draw_batch.submit(5000).unwrap();
}

fn render_dna_region(cp: &ColorPalette, draw_batch: &mut DrawBatch) {
    draw_batch.fill_region(
        Rect::with_size(SCREEN_WIDTH - 1, 0, 0, SCREEN_HEIGHT - 1),
        ColorPair::new(cp.bg_dna, cp.bg_dna),
        to_cp437(' '),
    );
    draw_batch.fill_region(
        Rect::with_size(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 0, SIDE_PANEL_WIDTH, 0),
        ColorPair::new(cp.bg_dna, cp.bg_dna),
        to_cp437(' '),
    );
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH - 1, 0),
        "DNA ",
        ColorPair::new(cp.fg_hud, cp.bg_dna),
    );
}

fn render_bars(player: &Object, cp: &ColorPalette, draw_batch: &mut DrawBatch) {
    // draw headers for bars
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 2),
        '♥',
        ColorPair::new(cp.fg_hud, cp.bg_hud),
    );

    draw_batch.print(Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 3), '√');

    // draw bars
    draw_batch.bar_horizontal(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH + 2, 2),
        17,
        player.actuators.hp,
        player.actuators.max_hp,
        ColorPair::new(cp.magenta, cp.bg_hud_content),
    );
    draw_batch.print_centered_at(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH + 10, 2),
        format!("{}/{}", player.actuators.hp, player.actuators.max_hp),
    );

    draw_batch.bar_horizontal(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH + 2, 3),
        17,
        player.processors.energy,
        player.processors.energy_storage,
        ColorPair::new(cp.yellow, cp.bg_bar),
    );

    draw_batch.print_centered_at(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH + 10, 3),
        format!(
            "{}/{}",
            player.processors.energy, player.processors.energy_storage
        ),
    );
}

fn render_action_fields(
    player: &Object,
    hud: &mut Hud,
    cp: &ColorPalette,
    draw_batch: &mut DrawBatch,
) {
    // draw action header
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 5),
        "Actions",
        ColorPair::new(cp.fg_hud, cp.bg_hud),
    );

    // draw buttons
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 6),
        "P",
        ColorPair::new(cp.fg_hud, cp.bg_hud),
    );
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 7),
        "S",
        ColorPair::new(cp.fg_hud, cp.bg_hud),
    );
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 8),
        "Q",
        ColorPair::new(cp.fg_hud, cp.bg_hud),
    );
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 9),
        "E",
        ColorPair::new(cp.fg_hud, cp.bg_hud),
    );

    // update action button texts
    hud.items.iter_mut().for_each(|i| match i.item_enum {
        HudItem::PrimaryAction => {
            i.text = player.get_primary_action(Target::Center).get_identifier()
        }
        HudItem::SecondaryAction => {
            i.text = player.get_secondary_action(Target::Center).get_identifier()
        }
        HudItem::Quick1Action => i.text = player.get_quick1_action().get_identifier(),
        HudItem::Quick2Action => i.text = player.get_quick2_action().get_identifier(),
        HudItem::DnaItem => {}
    });
}

fn render_inventory(player: &Object, layout: Rect, cp: &ColorPalette, draw_batch: &mut DrawBatch) {
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 11),
        "Inventory",
        ColorPair::new(cp.fg_hud, cp.bg_hud),
    );

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
        // take only as many chars as fit into the inventory item name field, or less
        // if the name is shorter
        let name_fitted: String = obj
            .visual
            .name
            .chars()
            .take((layout.width() - 3) as usize)
            .collect();
        draw_batch.print_color(
            Point::new(layout.x1 + 3, layout.y1 + idx as i32),
            name_fitted,
            ColorPair::new(obj.visual.fg_color, cp.bg_hud),
        );
    }
}

fn render_log(state: &GameState, layout: Rect, cp: &ColorPalette, draw_batch: &mut DrawBatch) {
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 24),
        "Log",
        ColorPair::new(cp.fg_hud, cp.bg_hud),
    );

    draw_batch.fill_region(
        layout,
        ColorPair::new(cp.fg_hud, cp.bg_hud_content),
        to_cp437(' '),
    );

    // print game messages, one line at a time
    let mut y = layout.height();
    let mut bg_flag: bool = modulus(state.log.messages.len(), 2) == 0;
    for (ref msg, class) in &mut state.log.messages.iter().rev() {
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
        let msg_end_y = layout.y1 + y;
        let msg_start_y: i32 = msg_end_y - msg_lines.len() as i32 + 1;
        let msg_area_start_y: i32 = i32::max(layout.y1, msg_end_y - msg_lines.len() as i32 + 1);

        // message background
        draw_batch.fill_region(
            Rect::with_exact(layout.x1, msg_area_start_y, layout.x2, msg_end_y),
            ColorPair::new(bg_color, bg_color),
            to_cp437(' '),
        );

        for (idx, line) in msg_lines.iter().enumerate() {
            if (msg_start_y + idx as i32) >= layout.y1 {
                draw_batch.print_color(
                    Point::new(layout.x1, msg_start_y + idx as i32),
                    line,
                    ColorPair::new(fg_color, bg_color),
                );
            }
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

fn render_ui_items(hud: &Hud, draw_batch: &mut DrawBatch) {
    for item in &hud.items {
        draw_batch.print_color(item.top_left_corner(), &item.text, item.color);
    }
}

fn render_tooltip(hud: &Hud, cp: &ColorPalette, draw_batch: &mut DrawBatch) {
    if hud.tooltips.is_empty() {
        return;
    }

    let max_width = hud
        .tooltips
        .iter()
        .map(|t| t.render_width())
        .max()
        .unwrap_or(0);
    let max_height = hud
        .tooltips
        .iter()
        .map(|t| t.render_height())
        .max()
        .unwrap_or(0);

    // check whether to render horizontally or vertically first
    let is_wide = max_width > max_height;
    let is_forwards: bool = if is_wide {
        // check whether to render tooltips left-to-right or the other way around
        hud.last_mouse.x + max_width <= SCREEN_WIDTH
    } else {
        // check whether to render tooltips up-down or the other way around
        hud.last_mouse.y + max_height <= SCREEN_HEIGHT
    };

    let x_direction = match (is_wide, is_forwards) {
        (true, _) => 0,
        (false, true) => 1,
        (false, false) => -1,
    };
    let mut next_x = hud.last_mouse.x + x_direction;

    let y_direction = match (is_wide, is_forwards) {
        (false, _) => 0,
        (true, true) => 1,
        (true, false) => -1,
    };
    let mut next_y = hud.last_mouse.y + y_direction;

    for tooltip in hud.tooltips {
        let tt_width = tooltip.render_width();
        let tt_height = tooltip.render_height();

        draw_batch.fill_region(
            Rect::with_size(next_x, next_y, tt_width, tt_height - 1),
            ColorPair::new(cp.fg_hud, cp.bg_hud_selected),
            to_cp437(' '),
        );
        draw_batch.draw_hollow_box(
            Rect::with_size(next_x, next_y, tt_width, tt_height - 1),
            ColorPair::new(cp.fg_hud, cp.bg_hud_selected),
        );
        let mut top_offset: i32 = 1;
        if tooltip.header.is_some() {
            top_offset = 3;
            draw_batch.draw_hollow_box(
                Rect::with_size(next_x, next_y, tt_width, 3),
                ColorPair::new(cp.fg_hud, cp.bg_hud_selected),
            );
            draw_batch.print_color_centered_at(
                Point::new(next_x + (tt_width / 2), next_y + 1),
                tooltip.header.unwrap(),
                ColorPair::new(cp.fg_hud, cp.bg_hud_selected),
            );
        }

        for (idx, (s1, s2)) in tooltip.attributes.iter().enumerate() {
            draw_batch.print_color(
                Point::new(next_x + 1, next_y + idx as i32 + top_offset),
                s1,
                ColorPair::new(cp.fg_hud, cp.bg_hud_selected),
            );
            draw_batch.print_color_right(
                Point::new(next_x + tt_width - 1, next_y + idx as i32 + top_offset),
                s2,
                ColorPair::new(cp.fg_hud, cp.bg_hud_selected),
            );
        }

        // advance x and y coordinates for next box
        if is_wide {
            let projected_x = next_x + (tt_width * x_direction);
            if projected_x > 0 && projected_x < SCREEN_WIDTH {
                next_x = projected_x;
            } else {
                if x_direction < 0 {
                    next_x = SCREEN_WIDTH - 1;
                } else {
                    next_x = 1;
                }
                next_y += tt_height * y_direction;
            }
        } else {
            let projected_y = next_y + (tt_height * x_direction);
            if projected_y > 0 && projected_y < SCREEN_HEIGHT {
                next_y = projected_y;
            } else {
                if y_direction < 0 {
                    next_y = SCREEN_HEIGHT - 1;
                } else {
                    next_y = 1
                }
                next_x += tt_width * x_direction;
            }
        }
    }
}
