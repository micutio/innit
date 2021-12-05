//! The GUI in Innit consists of the sidebar and any tooltips appearing over objects on the map or
//! UI elements.
//!
//! - varying visibility and detail, depending on perception (add to genetic traits)
//! - potential structure:
//!   - header
//!   - table with attributes and values:
//!     - hp
//!     - energy
//!     - receptor and whether it's matching with us

use crate::core::game_state::{GameState, MsgClass};
use crate::entity::genetics::TraitFamily;
use crate::entity::object::Object;
use crate::game::{SCREEN_HEIGHT, SCREEN_WIDTH, SIDE_PANEL_HEIGHT, SIDE_PANEL_WIDTH};
use crate::ui::palette;
use crate::{entity::action::Target, util::text_to_width};
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
    DnaItem, // little dna pieces that line the top and right side of the hud
    BarItem, // health, energy and lifetime bars
    UseInventory { idx: usize },
    DropInventory { idx: usize },
}

impl HudItem {
    fn is_use_inventory_item(&self) -> bool {
        match *self {
            Self::UseInventory { idx: _ } => true,
            _ => false,
        }
    }

    fn is_drop_inventory_item(&self) -> bool {
        match *self {
            Self::DropInventory { idx: _ } => true,
            _ => false,
        }
    }

    fn is_dna_item(&self) -> bool {
        match *self {
            Self::DnaItem => true,
            _ => false,
        }
    }
}

fn create_hud_items(hud_layout: &Rect) -> Vec<UiItem<HudItem>> {
    let button_len = SIDE_PANEL_WIDTH / 2;
    let button_x = hud_layout.x1 + 3;
    let fg_col = palette().hud_fg;
    let bg_col = palette().hud_bg;
    let col_pair = ColorPair::new(fg_col, bg_col);
    let bar_width = SIDE_PANEL_WIDTH - 3;
    let items = vec![
        UiItem::new(
            HudItem::BarItem,
            "",
            ToolTip::header_only("your health"),
            Rect::with_size(button_x, 2, bar_width, 1),
            col_pair,
        ),
        UiItem::new(
            HudItem::BarItem,
            "",
            ToolTip::header_only("your energy"),
            Rect::with_size(button_x, 3, bar_width, 1),
            col_pair,
        ),
        /* lifetime display disabled for now
        UiItem::new(
            HudItem::BarItem,
            "",
            ToolTip::header_only("your life time until regeneration"),
            Rect::with_size(button_x, 4, bar_width, 1),
            col_pair,
        ),
        */
        UiItem::new(
            HudItem::PrimaryAction,
            "",
            ToolTip::header_only("select new primary action"),
            Rect::with_size(button_x, 7, button_len, 1),
            col_pair,
        ),
        UiItem::new(
            HudItem::SecondaryAction,
            "",
            ToolTip::header_only("select new secondary action"),
            Rect::with_size(button_x, 8, button_len, 1),
            col_pair,
        ),
        UiItem::new(
            HudItem::Quick1Action,
            "",
            ToolTip::header_only("select new quick action"),
            Rect::with_size(button_x, 9, button_len, 1),
            col_pair,
        ),
        UiItem::new(
            HudItem::Quick2Action,
            "",
            ToolTip::header_only("select new quick action"),
            Rect::with_size(button_x, 10, button_len, 1),
            col_pair,
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
    pub fn new<S1: Into<String>>(header: S1, attrs: Vec<(String, String)>) -> Self {
        ToolTip {
            header: Some(header.into()),
            attributes: attrs
                .iter()
                .map(|(e1, e2)| (e1.into(), e2.into()))
                .collect(),
        }
    }

    pub fn no_header(attrs: Vec<(String, String)>) -> Self {
        ToolTip {
            header: None,
            attributes: attrs
                .iter()
                .map(|(e1, e2)| (e1.into(), e2.into()))
                .collect(),
        }
    }

    pub fn header_only<S1: Into<String>>(header: S1) -> Self {
        ToolTip {
            header: Some(header.into()),
            attributes: Vec::new(),
        }
    }

    /// Calculate the width in `[cells]` that a text box will require to be rendered on screen.
    fn content_width(&self) -> i32 {
        let header_width: usize = if let Some(h) = &self.header {
            h.len()
        } else {
            0
        };

        let attributes_width: usize = self
            .attributes
            .iter()
            .map(|(s1, s2)| s1.len() + s2.len() + 1)
            .max()
            .unwrap_or(0);

        // pad header with 2 and attributes with 3 to account for borders and separators
        (header_width + 2).max(attributes_width) as i32
    }

    /// Calculate the height in `[cells]` that a text box will require to be rendered on screen.
    fn content_height(&self) -> i32 {
        // header takes two lines for header text and separator
        let header_height = if self.header.is_some() {
            if self.attributes.is_empty() {
                1
            } else {
                2
            }
        } else {
            0
        };

        let attributes_height = self.attributes.len();
        // println!("ATTRIBUTES LEN {}", self.attributes.len());

        // pad height with 2 cells to account for rendering borders top and bottom
        (header_height + attributes_height) as i32
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
    pub inv_area: Rect,
    pub log_area: Rect,
    last_mouse: Point,
    pub require_refresh: bool,
    pub items: Vec<UiItem<HudItem>>,
    tooltips: Vec<ToolTip>,
}

impl Hud {
    pub fn new() -> Self {
        let x1 = SCREEN_WIDTH - SIDE_PANEL_WIDTH;
        let y1 = 0;
        let x2 = x1 + SIDE_PANEL_WIDTH - 1;
        let y2 = SIDE_PANEL_HEIGHT - 1;
        let layout = Rect::with_exact(x1, y1, x2, y2);
        let inv_area = Rect::with_exact(x1 + 1, 13, SCREEN_WIDTH - 2, 23);
        let log_area = Rect::with_exact(x1 + 1, 26, SCREEN_WIDTH - 2, 58);
        Hud {
            layout,
            inv_area,
            log_area,
            last_mouse: Point::new(0, 0),
            require_refresh: false,
            items: create_hud_items(&layout),
            tooltips: Vec::new(),
        }
    }

    pub fn update_tooltips(&mut self, mouse_pos: Point, names: Vec<ToolTip>) {
        self.tooltips.clear();
        if let Some(item) = self
            .items
            .iter()
            .find(|i| i.layout.point_in_rect(mouse_pos))
        {
            self.tooltips.push(item.tooltip.clone());
        } else {
            self.tooltips = names;
        }

        self.require_refresh = self.last_mouse != mouse_pos;
        self.last_mouse = mouse_pos;
    }

    pub fn update_ui_items(&mut self, player: &Object) {
        self.items.retain(|i| {
            !i.item_enum.is_dna_item()
                && !i.item_enum.is_use_inventory_item()
                && !i.item_enum.is_drop_inventory_item()
        });

        let combined_simplified_dna = player.get_combined_simplified_dna();
        let player_dna_length = player.dna.simplified.len();
        let horiz_display_count = SIDE_PANEL_WIDTH as usize - 5;
        for (h_offset, g_trait) in combined_simplified_dna
            .iter()
            .take(horiz_display_count)
            .enumerate()
        {
            let col: (u8, u8, u8, u8) = match g_trait.trait_family {
                TraitFamily::Sensing => palette().hud_fg_dna_processor,
                TraitFamily::Processing => palette().hud_fg_dna_actuator,
                TraitFamily::Actuating => palette().hud_fg_dna_sensor,
                TraitFamily::Junk(_) => (59, 59, 59, 255), // TODO
                TraitFamily::Ltr => (255, 255, 255, 255),  // TODO
            };
            let dna_glyph: char = if h_offset % 2 == 0 { '►' } else { '◄' };

            let gene_title = format!("gene {:2}:", h_offset + 1);
            let is_from_plasmid = h_offset >= player_dna_length;
            let mut tooltip_text = vec![
                (gene_title, g_trait.trait_name.clone()),
                ("group  :".to_string(), g_trait.trait_family.to_string()),
            ];
            if is_from_plasmid {
                tooltip_text.push(("origin: ".to_string(), "plasmid".to_string()));
            }

            let bg_color = if is_from_plasmid {
                palette().hud_bg_tooltip
            } else {
                palette().hud_bg
            };

            self.items.push(UiItem::new(
                HudItem::DnaItem,
                dna_glyph,
                ToolTip::no_header(tooltip_text),
                Rect::with_size(
                    SCREEN_WIDTH - SIDE_PANEL_WIDTH + 3 + h_offset as i32,
                    0,
                    1,
                    1,
                ),
                ColorPair::new(col, bg_color),
            ));
        }

        for (v_offset, g_trait) in combined_simplified_dna
            .iter()
            .skip(horiz_display_count)
            .enumerate()
        {
            // abort if we're running out of vertical rendering space
            if v_offset >= SCREEN_HEIGHT as usize {
                break;
            }
            let col: (u8, u8, u8, u8) = match g_trait.trait_family {
                TraitFamily::Sensing => palette().hud_fg_dna_processor,
                TraitFamily::Processing => palette().hud_fg_dna_actuator,
                TraitFamily::Actuating => palette().hud_fg_dna_sensor,
                TraitFamily::Junk(_) => (59, 59, 59, 255), // TODO
                TraitFamily::Ltr => (255, 255, 255, 255),  // TODO
            };

            let dna_glyph: char = if v_offset % 2 == 0 { '▼' } else { '▲' };

            let gene_title = format!("gene {:2}:", v_offset + 1);
            let is_from_plasmid = v_offset + horiz_display_count >= player_dna_length;
            let mut tooltip_text = vec![
                (gene_title, g_trait.trait_name.clone()),
                ("group  :".to_string(), g_trait.trait_family.to_string()),
            ];
            if is_from_plasmid {
                tooltip_text.push(("origin :".to_string(), "plasmid".to_string()));
            }

            let bg_color = if is_from_plasmid {
                palette().hud_bg_tooltip
            } else {
                palette().hud_bg
            };
            self.items.push(UiItem::new(
                HudItem::DnaItem,
                dna_glyph,
                ToolTip::no_header(tooltip_text),
                Rect::with_size(SCREEN_WIDTH - 1, v_offset as i32, 1, 1),
                ColorPair::new(col, bg_color),
            ));
        }

        for (idx, obj) in player.inventory.items.iter().enumerate() {
            if idx as i32 > self.inv_area.height() {
                break;
            }

            // take only as many chars as fit into the inventory item name field, or less
            // if the name is shorter
            let name_fitted: String = obj
                .visual
                .name
                .chars()
                .take((self.inv_area.width() - 5) as usize)
                .collect();

            let use_layout = Rect::with_size(
                self.inv_area.x1 + 1,
                self.inv_area.y1 + idx as i32,
                self.inv_area.width() - 3,
                1,
            );
            let bg_col = palette().hud_bg;

            let item_tooltip = if let Some(item) = &obj.item {
                ToolTip::new(
                    format!("use {}", &obj.visual.name),
                    vec![(item.description.clone(), "".to_string())],
                )
            } else {
                ToolTip::header_only(format!("this can't be used or consumed"))
            };

            self.items.push(UiItem::new(
                HudItem::UseInventory { idx },
                format!("{} {}", obj.visual.glyph, name_fitted),
                item_tooltip,
                use_layout,
                ColorPair::new(obj.visual.fg_color, bg_col),
            ));

            let drop_layout = Rect::with_size(
                self.inv_area.x1 + self.inv_area.width() - 2,
                self.inv_area.y1 + idx as i32,
                2,
                1,
            );
            let magenta = palette().hud_fg_bar_health;
            self.items.push(UiItem::new(
                HudItem::DropInventory { idx },
                " x",
                ToolTip::header_only(format!("drop {}", &obj.visual.name)),
                drop_layout,
                ColorPair::new(magenta, bg_col),
            ));
        }
    }
}

pub fn render_gui(state: &GameState, hud: &mut Hud, _ctx: &mut Rltk, player: &Object) {
    hud.update_ui_items(player);
    let mut draw_batch = DrawBatch::new();
    let fg_hud = palette().hud_fg;
    let bg_hud = palette().hud_bg;

    // fill side panel background
    draw_batch.fill_region(
        hud.layout,
        ColorPair::new(fg_hud, bg_hud),
        rltk::to_cp437(' '),
    );

    render_dna_region(&mut draw_batch);
    render_bars(player, &mut draw_batch);
    render_action_fields(player, hud, &mut draw_batch);
    render_inventory(hud, player, hud.inv_area, &mut draw_batch);
    render_log(state, hud.log_area, &mut draw_batch);
    render_ui_items(hud, &mut draw_batch);
    render_tooltip(hud, &mut draw_batch);

    draw_batch.submit(5000).unwrap();
}

fn render_dna_region(draw_batch: &mut DrawBatch) {
    let fg_hud = palette().hud_fg;
    let bg_dna = palette().hud_bg_dna;
    draw_batch.fill_region(
        Rect::with_size(SCREEN_WIDTH - 1, 0, 0, SCREEN_HEIGHT - 1),
        ColorPair::new(bg_dna, bg_dna),
        to_cp437(' '),
    );
    draw_batch.fill_region(
        Rect::with_size(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 0, SIDE_PANEL_WIDTH - 1, 0),
        ColorPair::new(bg_dna, bg_dna),
        to_cp437(' '),
    );
    draw_batch.print_color(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH, 0),
        "DNA ",
        ColorPair::new(fg_hud, bg_dna),
    );
}

fn render_bars(player: &Object, draw_batch: &mut DrawBatch) {
    let fg_hud = palette().hud_fg;
    let bg_bar = palette().hud_bg_bar;
    let bg_hud = palette().hud_bg;
    let bg_hud_content = palette().hud_bg_content;
    let health = palette().hud_fg_bar_health;
    let energy = palette().hud_fg_bar_energy;

    let bar_icon_col = ColorPair::new(fg_hud, bg_hud);
    let bar_x = SCREEN_WIDTH - SIDE_PANEL_WIDTH + 1;
    let bar_width = SIDE_PANEL_WIDTH - 4;

    // draw headers for bars
    draw_batch.print_color(Point::new(bar_x, 2), '+', bar_icon_col);
    draw_batch.print_color(Point::new(bar_x, 3), '√', bar_icon_col);
    // draw_batch.print_color(Point::new(bar_x, 4), '♥', bar_icon_col);

    // draw bars
    // - health bar
    draw_batch.bar_horizontal(
        Point::new(bar_x + 2, 2),
        bar_width,
        player.actuators.hp,
        player.actuators.max_hp,
        ColorPair::new(health, bg_hud_content),
    );
    draw_batch.print_centered_at(
        Point::new(bar_x + 9, 2),
        format!("{}/{}", player.actuators.hp, player.actuators.max_hp),
    );
    // - energy bar
    draw_batch.bar_horizontal(
        Point::new(bar_x + 2, 3),
        bar_width,
        player.processors.energy,
        player.processors.energy_storage,
        ColorPair::new(energy, bg_bar),
    );

    draw_batch.print_centered_at(
        Point::new(bar_x + 9, 3),
        format!(
            "{}/{}",
            player.processors.energy, player.processors.energy_storage
        ),
    );
    // - lifetime bar
    // disabled for now, because it has no useful functionality
    /*
    let lifetime = palette().hud_fg_bar_lifetime;
    let has_finite_life = player
        .processors
        .actions
        .iter()
        .any(|f| f.get_identifier().contains("killswitch"));
    let current_life = if has_finite_life {
        player.processors.life_elapsed
    } else {
        1
    };
    let max_life = if has_finite_life {
        player.processors.life_expectancy
    } else {
        1
    };
    draw_batch.bar_horizontal(
        Point::new(SCREEN_WIDTH - SIDE_PANEL_WIDTH + 3, 4),
        bar_width,
        current_life,
        max_life,
        ColorPair::new(lifetime, bg_bar),
    );

    let life_indicator = if has_finite_life {
        format!("{}/{}", current_life, max_life)
    } else {
        "∞".into()
    };

    draw_batch.print_centered_at(Point::new(bar_x + 9, 4), life_indicator);
    */
}

fn render_action_fields(player: &Object, hud: &mut Hud, draw_batch: &mut DrawBatch) {
    let action_header_bg = palette().hud_bg_dna;
    let action_bg = palette().hud_bg;
    let action_fg = palette().hud_fg;
    let action_fg_hl = palette().hud_fg_highlight;
    let x = SCREEN_WIDTH - SIDE_PANEL_WIDTH + 1;

    // draw action header
    draw_batch.fill_region(
        Rect::with_size(x - 1, 6, SIDE_PANEL_WIDTH - 1, 0),
        ColorPair::new(action_fg, action_header_bg),
        to_cp437(' '),
    );
    draw_batch.print_color(
        Point::new(x, 6),
        "Actions",
        ColorPair::new(action_fg, action_header_bg),
    );
    // draw buttons
    draw_batch.print_color(
        Point::new(x, 7),
        "P",
        ColorPair::new(action_fg_hl, action_bg),
    );
    draw_batch.print_color(
        Point::new(x, 8),
        "S",
        ColorPair::new(action_fg_hl, action_bg),
    );
    draw_batch.print_color(
        Point::new(x, 9),
        "Q",
        ColorPair::new(action_fg_hl, action_bg),
    );
    draw_batch.print_color(
        Point::new(x, 10),
        "E",
        ColorPair::new(action_fg_hl, action_bg),
    );

    // update action button texts
    let p_action = player.get_primary_action(Target::Center);
    let s_action = player.get_secondary_action(Target::Center);
    let q1_action = player.get_quick1_action();
    let q2_action = player.get_quick2_action();
    hud.items.iter_mut().for_each(|i| match i.item_enum {
        HudItem::PrimaryAction => {
            i.text = format!(
                "{} ({}√)",
                p_action.get_identifier(),
                p_action.get_energy_cost()
            )
        }
        HudItem::SecondaryAction => {
            i.text = format!(
                "{} ({}√)",
                s_action.get_identifier(),
                s_action.get_energy_cost()
            )
        }
        HudItem::Quick1Action => {
            i.text = format!(
                "{} ({}√)",
                q1_action.get_identifier(),
                q1_action.get_energy_cost()
            )
        }
        HudItem::Quick2Action => {
            i.text = format!(
                "{} ({}√)",
                q2_action.get_identifier(),
                q2_action.get_energy_cost()
            )
        }
        _ => {} // HudItem::DnaItem => {}
                // HudItem::UseInventory(_) => {}
    });
}

fn render_inventory(hud: &Hud, player: &Object, layout: Rect, draw_batch: &mut DrawBatch) {
    let fg_inv = palette().hud_fg;
    let bg_inv_header = palette().hud_bg_dna;

    draw_batch.fill_region(
        Rect::with_size(layout.x1 - 1, layout.y1 - 1, layout.width() + 1, 0),
        ColorPair::new(fg_inv, bg_inv_header),
        to_cp437(' '),
    );

    draw_batch.print_color(
        Point::new(layout.x1, layout.y1 - 1),
        format!(
            "Inventory [{}/{}]",
            player.inventory.items.len(),
            player.actuators.volume
        ),
        ColorPair::new(fg_inv, bg_inv_header),
    );

    hud.items
        .iter()
        .filter(|item| item.item_enum.is_use_inventory_item())
        .for_each(|item| {
            draw_batch.print_color(item.top_left_corner(), &item.text, item.color);
        });
}

fn render_log(state: &GameState, layout: Rect, draw_batch: &mut DrawBatch) {
    let fg_log = palette().hud_fg;
    let bg_log_header = palette().hud_bg_dna;

    draw_batch.fill_region(
        Rect::with_size(layout.x1 - 1, layout.y1 - 1, layout.width() + 1, 0),
        ColorPair::new(fg_log, bg_log_header),
        to_cp437(' '),
    );

    draw_batch.print_color(
        Point::new(layout.x1, layout.y1 - 1),
        "Log",
        ColorPair::new(fg_log, bg_log_header),
    );

    // convert messages into log text lines (str, fg_col, bg_col)
    let mut bg_flag: bool = state.log.messages.len() % 2 == 0;
    let mut log_lines: Vec<(String, (u8, u8, u8, u8), (u8, u8, u8, u8))> = Vec::new();
    for (msg, class) in &state.log.messages {
        let lines = text_to_width(&msg, layout.width());
        let fg_color = match class {
            MsgClass::Alert => palette().hud_fg_msg_alert,
            MsgClass::Info => palette().hud_fg_msg_info,
            MsgClass::Action => palette().hud_fg_msg_action,
            MsgClass::Story => palette().hud_fg_msg_story,
        };
        let bg_color = if bg_flag {
            palette().hud_bg_log1
        } else {
            palette().hud_bg_log2
        };
        bg_flag = !bg_flag;
        for l in lines {
            log_lines.push((l, fg_color, bg_color));
        }
    }

    // print game messages, one line at a time
    let visible_log = if log_lines.len() as i32 <= layout.height() {
        &log_lines
    } else {
        let start: usize = log_lines.len() - layout.height() as usize - 1;
        &log_lines[start..]
    };

    let mut y = layout.y1;
    for l in visible_log {
        draw_batch.fill_region(
            Rect::with_exact(layout.x1, y, layout.x2, y),
            ColorPair::new(l.1, l.2),
            to_cp437(' '),
        );
        draw_batch.print_color(
            Point::new(layout.x1, y as i32),
            &l.0,
            ColorPair::new(l.1, l.2),
        );
        y += 1;
    }
}

fn render_ui_items(hud: &Hud, draw_batch: &mut DrawBatch) {
    for item in &hud.items {
        draw_batch.print_color(item.top_left_corner(), &item.text, item.color);
    }
}

fn render_tooltip(hud: &Hud, draw_batch: &mut DrawBatch) {
    if hud.tooltips.is_empty() {
        return;
    }

    let max_width = hud
        .tooltips
        .iter()
        .map(|t| t.content_width())
        .max()
        .unwrap_or(0)
        + 2;
    let max_height = hud
        .tooltips
        .iter()
        .map(|t| t.content_height())
        .max()
        .unwrap_or(0)
        + 2;

    // check whether to render horizontally or vertically first
    let is_render_horiz = max_width < max_height;
    let is_forwards: bool = if is_render_horiz {
        // check whether to render tooltips left-to-right or the other way around
        hud.last_mouse.x < SCREEN_WIDTH - hud.last_mouse.x
    } else {
        // check whether to render tooltips up-down or the other way around
        hud.last_mouse.y < SCREEN_HEIGHT - hud.last_mouse.y
    };

    let x_direction = match (is_render_horiz, is_forwards) {
        (true, true) => 1,
        (true, false) => -1,
        (false, _) => 1,
    };
    let mut next_x = if hud.last_mouse.x + x_direction + max_width < SCREEN_WIDTH {
        hud.last_mouse.x + x_direction
    } else {
        hud.last_mouse.x - (hud.last_mouse.x + x_direction + max_width - SCREEN_WIDTH)
    };

    let y_direction = match (is_render_horiz, is_forwards) {
        (true, _) => 0,
        (false, true) => 1,
        (false, false) => -1,
    };
    let mut next_y = if hud.last_mouse.y + y_direction + max_height < SCREEN_HEIGHT {
        hud.last_mouse.y + y_direction
    } else {
        hud.last_mouse.y - (hud.last_mouse.y + y_direction + max_height - SCREEN_HEIGHT)
    };

    // define tooltip colors
    let fg_tt_border = palette().hud_fg_border;
    let fg_tt = palette().hud_fg;
    let bg_tt = palette().hud_bg_tooltip;

    for tooltip in &hud.tooltips {
        if tooltip.header.is_none() && tooltip.attributes.is_empty() {
            continue;
        }
        // (+2) for borders and (-1) for starting from 0, equals (+1)
        let tt_width = tooltip.content_width() + 1;
        let tt_height = tooltip.content_height() + 1;

        draw_batch.fill_region(
            Rect::with_size(next_x, next_y, tt_width, tt_height),
            ColorPair::new(fg_tt, bg_tt),
            to_cp437(' '),
        );
        draw_batch.draw_hollow_box(
            Rect::with_size(next_x, next_y, tt_width, tt_height),
            ColorPair::new(fg_tt_border, bg_tt),
        );
        let mut top_offset: i32 = 1;
        if tooltip.header.is_some() {
            top_offset = 3;

            if !tooltip.attributes.is_empty() {
                draw_batch.print_color(
                    Point::new(next_x, next_y + 2),
                    // to_cp437('─'),
                    "├",
                    ColorPair::new(fg_tt_border, bg_tt),
                );
                draw_batch.print_color(
                    Point::new(next_x + tt_width, next_y + 2),
                    // to_cp437('─'),
                    "┤",
                    ColorPair::new(fg_tt_border, bg_tt),
                );
                for x in (next_x + 1)..(next_x + tt_width) {
                    draw_batch.print_color(
                        Point::new(x, next_y + 2),
                        // to_cp437('─'),
                        "─",
                        ColorPair::new(fg_tt_border, bg_tt),
                    );
                }
            }

            draw_batch.print_color_centered_at(
                Point::new(next_x + (tt_width / 2), next_y + 1),
                tooltip.header.as_ref().unwrap(),
                ColorPair::new(fg_tt, bg_tt),
            );
        }

        for (idx, (s1, s2)) in tooltip.attributes.iter().enumerate() {
            draw_batch.print_color(
                Point::new(next_x + 1, next_y + idx as i32 + top_offset),
                s1,
                ColorPair::new(fg_tt, bg_tt),
            );
            draw_batch.print_color_right(
                Point::new(next_x + tt_width, next_y + idx as i32 + top_offset),
                s2,
                ColorPair::new(fg_tt, bg_tt),
            );
        }

        // advance x and y coordinates for next box
        if is_render_horiz {
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
            let projected_y = next_y + 1 + (tt_height * x_direction);
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
