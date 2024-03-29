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

use crate::entity::act;
use crate::entity::genetics;
use crate::entity::Object;
use crate::game::{self, State};
use crate::ui;
use crate::ui::palette;
use crate::util;

use bracket_lib::prelude as rltk;

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
    pub layout: rltk::Rect,
    pub color: rltk::ColorPair,
}

impl<T> UiItem<T> {
    pub fn new<S1: Into<String>>(
        item_enum: T,
        text: S1,
        tooltip: ToolTip,
        layout: rltk::Rect,
        color: rltk::ColorPair,
    ) -> Self {
        Self {
            item_enum,
            text: text.into(),
            tooltip,
            layout,
            color,
        }
    }

    pub fn top_left_corner(&self) -> rltk::Point {
        rltk::Point::new(self.layout.x1, self.layout.y1)
    }
}

#[derive(PartialEq, Eq)]
pub enum Item {
    PrimaryAction,
    SecondaryAction,
    Quick1Action,
    Quick2Action,
    DnaElement, // little dna pieces that line the top and right side of the hud
    BarElement, // health, energy and lifetime bars
    UseInventory { idx: usize },
    DropInventory { idx: usize },
}

impl Item {
    const fn is_use_inventory_item(&self) -> bool {
        matches!(*self, Self::UseInventory { idx: _ })
    }

    const fn is_drop_inventory_item(&self) -> bool {
        matches!(*self, Self::DropInventory { idx: _ })
    }

    const fn is_dna_item(&self) -> bool {
        matches!(*self, Self::DnaElement)
    }
}

fn create_hud_items(hud_layout: &rltk::Rect) -> Vec<UiItem<Item>> {
    let button_len = game::consts::SIDE_PANEL_WIDTH / 2;
    let button_x = hud_layout.x1 + 3;
    let fg_col = palette().hud_fg;
    let bg_col = palette().hud_bg;
    let col_pair = rltk::ColorPair::new(fg_col, bg_col);
    let bar_width = game::consts::SIDE_PANEL_WIDTH - 3;
    let items = vec![
        UiItem::new(
            Item::BarElement,
            "",
            ToolTip::header_only("your health"),
            rltk::Rect::with_size(button_x, 2, bar_width, 1),
            col_pair,
        ),
        UiItem::new(
            Item::BarElement,
            "",
            ToolTip::header_only("your energy"),
            rltk::Rect::with_size(button_x, 3, bar_width, 1),
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
            Item::PrimaryAction,
            "",
            ToolTip::header_only("select new primary action"),
            rltk::Rect::with_size(button_x, 7, button_len, 1),
            col_pair,
        ),
        UiItem::new(
            Item::SecondaryAction,
            "",
            ToolTip::header_only("select new secondary action"),
            rltk::Rect::with_size(button_x, 8, button_len, 1),
            col_pair,
        ),
        UiItem::new(
            Item::Quick1Action,
            "",
            ToolTip::header_only("select new quick action"),
            rltk::Rect::with_size(button_x, 9, button_len, 1),
            col_pair,
        ),
        UiItem::new(
            Item::Quick2Action,
            "",
            ToolTip::header_only("select new quick action"),
            rltk::Rect::with_size(button_x, 10, button_len, 1),
            col_pair,
        ),
    ];

    items
}

#[derive(Clone, Debug)]
pub struct ToolTip {
    header: Option<String>,
    text: Vec<String>,
    attributes: Vec<(String, String)>,
}

impl ToolTip {
    pub fn new<S1: Into<String>>(
        header: S1,
        text: Vec<String>,
        attributes: Vec<(String, String)>,
    ) -> Self {
        Self {
            header: Some(header.into()),
            text,
            attributes,
        }
    }

    pub fn no_header(text: Vec<String>, attributes: Vec<(String, String)>) -> Self {
        Self {
            header: None,
            text,
            attributes,
        }
    }

    pub fn header_only<S1: Into<String>>(header: S1) -> Self {
        Self {
            header: Some(header.into()),
            text: Vec::new(),
            attributes: Vec::new(),
        }
    }

    /// Calculate the width in `[cells]` that a text box will require to be rendered on screen.
    fn content_width(&self) -> i32 {
        let header_width: usize = self.header.as_ref().map_or(0, std::string::String::len);

        let text_width: usize = self.text.iter().map(|s| s.len() + 1).max().unwrap_or(0);

        let attributes_width: usize = self
            .attributes
            .iter()
            .map(|(s1, s2)| s1.len() + s2.len() + 1)
            .max()
            .unwrap_or(0);

        // pad header with 2 and attributes with 3 to account for borders and separators
        *vec![header_width + 2, text_width, attributes_width]
            .iter()
            .max()
            .unwrap() as i32
    }

    /// Calculate the height in `[cells]` that a text box will require to be rendered on screen.
    fn content_height(&self) -> i32 {
        // header takes two lines for header text and separator
        let header_height = if self.header.is_some() {
            match (self.attributes.is_empty(), self.text.is_empty()) {
                (true, true) => 1,
                (false, false) => 3,
                _ => 2,
            }
        } else {
            0
        };

        let text_height = self.text.len();
        let attributes_height = self.attributes.len();

        (header_height + text_height + attributes_height) as i32
    }
}

pub struct Hud {
    layout: rltk::Rect,
    pub inv_area: rltk::Rect,
    pub log_area: rltk::Rect,
    last_mouse: rltk::Point,
    pub require_refresh: bool,

    pub items: Vec<UiItem<Item>>,
    tooltips: Vec<ToolTip>,
}

impl Hud {
    pub fn new() -> Self {
        let x1 = game::consts::SCREEN_WIDTH - game::consts::SIDE_PANEL_WIDTH;
        let y1 = 0;
        let x2 = x1 + game::consts::SIDE_PANEL_WIDTH;
        let y2 = game::consts::SIDE_PANEL_HEIGHT;
        let layout = rltk::Rect::with_exact(x1, y1, x2, y2);
        let inv_area = rltk::Rect::with_exact(x1 + 1, 13, game::consts::SCREEN_WIDTH - 1, 23);
        let log_area = rltk::Rect::with_exact(x1 + 1, 26, game::consts::SCREEN_WIDTH - 1, 58);
        Self {
            layout,
            inv_area,
            log_area,
            last_mouse: rltk::Point::new(0, 0),
            require_refresh: false,

            items: create_hud_items(&layout),
            tooltips: Vec::new(),
        }
    }

    pub fn update_tooltips(&mut self, mouse_pos: rltk::Point, names: Vec<ToolTip>) {
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

    #[allow(clippy::too_many_lines)]
    pub fn update_ui_items(&mut self, player: &Object) {
        self.items.retain(|i| {
            !i.item_enum.is_dna_item()
                && !i.item_enum.is_use_inventory_item()
                && !i.item_enum.is_drop_inventory_item()
        });

        let combined_simplified_dna = player.get_combined_simplified_dna();
        let player_dna_length = player.dna.simplified.len();
        let horiz_display_count = game::consts::SIDE_PANEL_WIDTH as usize - 5;
        for (h_offset, g_trait) in combined_simplified_dna
            .iter()
            .take(horiz_display_count)
            .enumerate()
        {
            use genetics::TraitFamily;
            let col: ui::Rgba = match g_trait.trait_family {
                TraitFamily::Sensing => palette().hud_fg_dna_processor,
                TraitFamily::Processing => palette().hud_fg_dna_actuator,
                TraitFamily::Actuating => palette().hud_fg_dna_sensor,
                TraitFamily::Junk(_) => ui::Rgba::new(59, 59, 59, 255), // TODO
                TraitFamily::Ltr => ui::Rgba::new(255, 255, 255, 255),  // TODO
            };
            let dna_glyph: char = if h_offset % 2 == 0 { '►' } else { '◄' };

            let gene_title = format!("gene {:2}:", h_offset + 1);
            let is_from_plasmid = h_offset >= player_dna_length;
            let mut tooltip_attrs = vec![
                (gene_title, g_trait.trait_name.clone()),
                ("group  :".to_string(), g_trait.trait_family.to_string()),
            ];
            if is_from_plasmid {
                tooltip_attrs.push(("origin: ".to_string(), "plasmid".to_string()));
            }

            let bg_color = if is_from_plasmid {
                palette().hud_bg_plasmid_dna
            } else {
                palette().hud_bg_dna
            };

            self.items.push(UiItem::new(
                Item::DnaElement,
                dna_glyph,
                ToolTip::no_header(Vec::new(), tooltip_attrs),
                rltk::Rect::with_size(
                    game::consts::SCREEN_WIDTH - game::consts::SIDE_PANEL_WIDTH
                        + 3
                        + h_offset as i32,
                    0,
                    1,
                    1,
                ),
                rltk::ColorPair::new(col, bg_color),
            ));
        }

        for (v_offset, g_trait) in combined_simplified_dna
            .iter()
            .skip(horiz_display_count)
            .enumerate()
        {
            // abort if we're running out of vertical rendering space
            if v_offset >= game::consts::SCREEN_HEIGHT as usize {
                break;
            }

            let col: ui::Rgba = match g_trait.trait_family {
                genetics::TraitFamily::Sensing => palette().hud_fg_dna_processor,
                genetics::TraitFamily::Processing => palette().hud_fg_dna_actuator,
                genetics::TraitFamily::Actuating => palette().hud_fg_dna_sensor,
                genetics::TraitFamily::Junk(_) => ui::Rgba::new(59, 59, 59, 255), // TODO
                genetics::TraitFamily::Ltr => ui::Rgba::new(255, 255, 255, 255),  // TODO
            };

            let dna_glyph: char = if v_offset % 2 == 0 { '▼' } else { '▲' };

            let gene_title = format!("gene {:2}:", v_offset + 1);
            let is_from_plasmid = v_offset + horiz_display_count >= player_dna_length;
            let mut tooltip_attrs = vec![
                (gene_title, g_trait.trait_name.clone()),
                ("group  :".to_string(), g_trait.trait_family.to_string()),
            ];
            if is_from_plasmid {
                tooltip_attrs.push(("origin :".to_string(), "plasmid".to_string()));
            }

            let bg_color = if is_from_plasmid {
                palette().hud_bg_plasmid_dna
            } else {
                palette().hud_bg_dna
            };
            self.items.push(UiItem::new(
                Item::DnaElement,
                dna_glyph,
                ToolTip::no_header(Vec::new(), tooltip_attrs),
                rltk::Rect::with_size(game::consts::SCREEN_WIDTH - 1, v_offset as i32, 1, 1),
                rltk::ColorPair::new(col, bg_color),
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

            let use_layout = rltk::Rect::with_size(
                self.inv_area.x1 + 1,
                self.inv_area.y1 + idx as i32,
                self.inv_area.width() - 3,
                1,
            );
            let bg_col = palette().hud_bg;

            let item_tooltip = if let Some(item) = &obj.item {
                let lines = util::text_to_width(&item.description, self.layout.width() as usize);
                ToolTip::new(format!("use {}", &obj.visual.name), lines, Vec::new())
            } else {
                ToolTip::header_only("this can't be used or consumed".to_string())
            };

            self.items.push(UiItem::new(
                Item::UseInventory { idx },
                format!("{} {}", obj.visual.glyph, name_fitted),
                item_tooltip,
                use_layout,
                rltk::ColorPair::new(obj.visual.fg_color, bg_col),
            ));

            let drop_layout = rltk::Rect::with_size(
                self.inv_area.x1 + self.inv_area.width() - 2,
                self.inv_area.y1 + idx as i32,
                2,
                1,
            );
            let magenta = palette().hud_fg_bar_health;
            self.items.push(UiItem::new(
                Item::DropInventory { idx },
                " x",
                ToolTip::header_only(format!("drop {}", &obj.visual.name)),
                drop_layout,
                rltk::ColorPair::new(magenta, bg_col),
            ));
        }
    }
}

impl Default for Hud {
    fn default() -> Self {
        Self::new()
    }
}

pub fn render_gui(state: &State, hud: &mut Hud, ctx: &mut rltk::BTerm, player: &Object) {
    ctx.set_active_console(game::consts::HUD_CON);
    hud.update_ui_items(player);
    let mut draw_batch = rltk::DrawBatch::new();
    let fg_hud = palette().hud_fg;
    let bg_hud = palette().hud_bg;

    // fill side panel background
    draw_batch.fill_region(
        hud.layout,
        rltk::ColorPair::new(fg_hud, bg_hud),
        rltk::to_cp437(' '),
    );

    // draw bottom line
    let btm_y = game::consts::SCREEN_HEIGHT - 1;
    draw_batch.fill_region(
        rltk::Rect::with_exact(7, btm_y, game::consts::SCREEN_WIDTH - 1, btm_y + 1),
        rltk::ColorPair::new(fg_hud, bg_hud),
        rltk::to_cp437(' '),
    );

    draw_batch.print_color(
        rltk::Point::new(9, btm_y),
        "Mobile Fluorescence Microscope",
        rltk::ColorPair::new(fg_hud, bg_hud),
    );

    render_dna_region(&mut draw_batch);
    render_bars(player, &mut draw_batch);
    render_action_fields(player, hud, &mut draw_batch);
    render_inventory(hud, player, hud.inv_area, &mut draw_batch);
    render_log(state, hud.log_area, &mut draw_batch);
    render_ui_items(hud, &mut draw_batch);
    render_tooltip(hud, &mut draw_batch);

    draw_batch.submit(game::consts::HUD_CON_Z).unwrap();
}

fn render_dna_region(draw_batch: &mut rltk::DrawBatch) {
    let fg_hud = palette().hud_fg;
    let bg_dna = palette().hud_bg_dna;
    draw_batch.fill_region(
        rltk::Rect::with_size(
            game::consts::SCREEN_WIDTH - 1,
            0,
            1,
            game::consts::SCREEN_HEIGHT,
        ),
        rltk::ColorPair::new(bg_dna, bg_dna),
        rltk::to_cp437(' '),
    );
    draw_batch.fill_region(
        rltk::Rect::with_size(
            game::consts::SCREEN_WIDTH - game::consts::SIDE_PANEL_WIDTH,
            0,
            game::consts::SIDE_PANEL_WIDTH,
            1,
        ),
        rltk::ColorPair::new(bg_dna, bg_dna),
        rltk::to_cp437(' '),
    );
    draw_batch.print_color(
        rltk::Point::new(
            game::consts::SCREEN_WIDTH - game::consts::SIDE_PANEL_WIDTH,
            0,
        ),
        "DNA ",
        rltk::ColorPair::new(fg_hud, bg_dna),
    );
}

fn render_bars(player: &Object, draw_batch: &mut rltk::DrawBatch) {
    let fg_hud = palette().hud_fg;
    let bg_bar = palette().hud_bg_bar;
    let bg_hud = palette().hud_bg;
    let health = palette().hud_fg_bar_health;
    let energy = palette().hud_fg_bar_energy;

    let bar_icon_col = rltk::ColorPair::new(fg_hud, bg_hud);
    let bar_icon_x = (game::consts::SCREEN_WIDTH - game::consts::SIDE_PANEL_WIDTH) + 1;
    let bar_x = bar_icon_x + 2;
    let bar_width = game::consts::SIDE_PANEL_WIDTH - 4;
    let bar_ctr = bar_width / 2;

    // draw headers for bars
    draw_batch.print_color(rltk::Point::new(bar_icon_x, 2), '+', bar_icon_col);
    draw_batch.print_color(rltk::Point::new(bar_icon_x, 3), '√', bar_icon_col);
    // draw_batch.print_color(Point::new(bar_x, 4), '♥', bar_icon_col);

    // draw bars
    // - health bar
    draw_batch.bar_horizontal(
        rltk::Point::new(bar_x, 2),
        bar_width,
        player.actuators.hp,
        player.actuators.max_hp,
        rltk::ColorPair::new(health, bg_bar),
    );
    draw_batch.print_color_centered_at(
        rltk::Point::new(bar_x + bar_ctr, 2),
        format!("{}/{}", player.actuators.hp, player.actuators.max_hp),
        rltk::ColorPair::new(bg_hud, health),
    );
    // - energy bar
    draw_batch.bar_horizontal(
        rltk::Point::new(bar_x, 3),
        bar_width,
        player.processors.energy,
        player.processors.energy_storage,
        rltk::ColorPair::new(energy, bg_bar),
    );

    draw_batch.print_color_centered_at(
        rltk::Point::new(bar_x + bar_ctr, 3),
        format!(
            "{}/{}",
            player.processors.energy, player.processors.energy_storage
        ),
        rltk::ColorPair::new(bg_hud, energy),
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

fn render_action_fields(player: &Object, hud: &mut Hud, draw_batch: &mut rltk::DrawBatch) {
    let action_header_bg = palette().hud_bg_dna;
    let action_bg = palette().hud_bg;
    let action_fg = palette().hud_fg;
    let action_fg_hl = palette().hud_fg_highlight;
    let x = game::consts::SCREEN_WIDTH - game::consts::SIDE_PANEL_WIDTH + 1;

    // draw action header
    draw_batch.fill_region(
        rltk::Rect::with_size(x - 1, 6, game::consts::SIDE_PANEL_WIDTH + 1, 1),
        rltk::ColorPair::new(action_fg, action_header_bg),
        rltk::to_cp437(' '),
    );
    draw_batch.print_color(
        rltk::Point::new(x, 6),
        "Actions",
        rltk::ColorPair::new(action_fg, action_header_bg),
    );
    // draw buttons
    draw_batch.print_color(
        rltk::Point::new(x, 7),
        "P",
        rltk::ColorPair::new(action_fg_hl, action_bg),
    );
    draw_batch.print_color(
        rltk::Point::new(x, 8),
        "S",
        rltk::ColorPair::new(action_fg_hl, action_bg),
    );
    draw_batch.print_color(
        rltk::Point::new(x, 9),
        "Q",
        rltk::ColorPair::new(action_fg_hl, action_bg),
    );
    draw_batch.print_color(
        rltk::Point::new(x, 10),
        "E",
        rltk::ColorPair::new(action_fg_hl, action_bg),
    );

    // update action button texts
    let p_action = player.get_primary_action(act::Target::Center);
    let s_action = player.get_secondary_action(act::Target::Center);
    let q1_action = player.get_quick1_action();
    let q2_action = player.get_quick2_action();
    hud.items.iter_mut().for_each(|i| match i.item_enum {
        Item::PrimaryAction => {
            i.text = format!(
                "{} ({}√)",
                p_action.get_identifier(),
                p_action.get_energy_cost()
            );
        }
        Item::SecondaryAction => {
            i.text = format!(
                "{} ({}√)",
                s_action.get_identifier(),
                s_action.get_energy_cost()
            );
        }
        Item::Quick1Action => {
            i.text = format!(
                "{} ({}√)",
                q1_action.get_identifier(),
                q1_action.get_energy_cost()
            );
        }
        Item::Quick2Action => {
            i.text = format!(
                "{} ({}√)",
                q2_action.get_identifier(),
                q2_action.get_energy_cost()
            );
        }
        _ => {} // HudItem::DnaItem => {}
                // HudItem::UseInventory(_) => {}
    });
}

fn render_inventory(
    hud: &Hud,
    player: &Object,
    layout: rltk::Rect,
    draw_batch: &mut rltk::DrawBatch,
) {
    let fg_inv = palette().hud_fg;
    let bg_inv_header = palette().hud_bg_dna;

    draw_batch.fill_region(
        rltk::Rect::with_size(layout.x1 - 1, layout.y1 - 1, layout.width() + 1, 1),
        rltk::ColorPair::new(fg_inv, bg_inv_header),
        rltk::to_cp437(' '),
    );

    draw_batch.print_color(
        rltk::Point::new(layout.x1, layout.y1 - 1),
        format!(
            "Inventory [{}/{}]",
            player.inventory.items.len(),
            player.actuators.volume
        ),
        rltk::ColorPair::new(fg_inv, bg_inv_header),
    );

    hud.items
        .iter()
        .filter(|item| item.item_enum.is_use_inventory_item())
        .for_each(|item| {
            draw_batch.print_color(item.top_left_corner(), &item.text, item.color);
        });
}

pub fn render_log(state: &State, layout: rltk::Rect, draw_batch: &mut rltk::DrawBatch) {
    let fg_log = palette().hud_fg;
    let bg_log_header = palette().hud_bg_dna;
    let bg_log_default = palette().hud_bg;

    draw_batch.fill_region(
        rltk::Rect::with_size(layout.x1 - 1, layout.y1 - 1, layout.width() + 1, 1),
        rltk::ColorPair::new(fg_log, bg_log_header),
        rltk::to_cp437(' '),
    );

    draw_batch.print_color(
        rltk::Point::new(layout.x1, layout.y1 - 1),
        "Log",
        rltk::ColorPair::new(fg_log, bg_log_header),
    );

    // convert messages into log text lines (str, fg_col, bg_col)
    let mut bg_flag: bool = state.log.messages.len() % 2 == 0;
    let mut log_lines: Vec<(String, ui::Rgba, ui::Rgba)> = Vec::new();
    let line_width = layout.width() as usize;
    for (msg, class) in &state.log.messages {
        let lines = util::text_to_width(msg, line_width);
        let fg_color = match class {
            game::msg::Class::Alert => palette().hud_fg_msg_alert,
            game::msg::Class::Info => palette().hud_fg_msg_info,
            game::msg::Class::Action => palette().hud_fg_msg_action,
            game::msg::Class::Story(_) => palette().hud_fg_msg_story,
        };
        let bg_color = if bg_flag {
            palette().hud_bg_log1
        } else {
            palette().hud_bg_log2
        };
        bg_flag = !bg_flag;

        log_lines.push((" ".into(), fg_color, bg_log_default));
        // if we're displaying text messages, prepend the author's name
        if let game::msg::Class::Story(author_opt) = class {
            let author_line = author_opt.as_ref().map_or_else(
                || ("Dr. S.".into(), ui::palette().col_acc1, bg_color),
                |author| (author.into(), ui::palette().col_acc4, bg_color),
            );
            log_lines.push(author_line);
        }

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
            rltk::Rect::with_exact(layout.x1, y, layout.x2, y + 1),
            rltk::ColorPair::new(l.1, l.2),
            rltk::to_cp437(' '),
        );
        draw_batch.print_color(
            rltk::Point::new(layout.x1, y),
            &l.0,
            rltk::ColorPair::new(l.1, l.2),
        );
        y += 1;
    }
}

fn render_ui_items(hud: &Hud, draw_batch: &mut rltk::DrawBatch) {
    for item in &hud.items {
        draw_batch.print_color(item.top_left_corner(), &item.text, item.color);
    }
}

#[allow(clippy::too_many_lines)]
fn render_tooltip(hud: &Hud, draw_batch: &mut rltk::DrawBatch) {
    if hud.tooltips.is_empty() {
        return;
    }

    let max_width = hud
        .tooltips
        .iter()
        .map(ui::hud::ToolTip::content_width)
        .max()
        .unwrap_or(0)
        + 2;
    let max_height = hud
        .tooltips
        .iter()
        .map(ui::hud::ToolTip::content_height)
        .max()
        .unwrap_or(0)
        + 2;

    // check whether to render horizontally or vertically first
    let is_render_horiz = max_width < max_height;
    let is_forwards: bool = if is_render_horiz {
        // check whether to render tooltips left-to-right or the other way around
        hud.last_mouse.x < game::consts::SCREEN_WIDTH - hud.last_mouse.x
    } else {
        // check whether to render tooltips up-down or the other way around
        hud.last_mouse.y < game::consts::SCREEN_HEIGHT - hud.last_mouse.y
    };

    let x_direction = match (is_render_horiz, is_forwards) {
        (true, true) | (false, _) => 1,
        (true, false) => -1,
        // (false, _) => 1,
    };
    let mut next_x = if hud.last_mouse.x + x_direction + max_width < game::consts::SCREEN_WIDTH {
        hud.last_mouse.x + x_direction
    } else {
        hud.last_mouse.x - (hud.last_mouse.x + x_direction + max_width - game::consts::SCREEN_WIDTH)
    };

    let y_direction = match (is_render_horiz, is_forwards) {
        (true, _) => 0,
        (false, true) => 1,
        (false, false) => -1,
    };
    let mut next_y = if hud.last_mouse.y + y_direction + max_height < game::consts::SCREEN_HEIGHT {
        hud.last_mouse.y + y_direction
    } else {
        hud.last_mouse.y
            - (hud.last_mouse.y + y_direction + max_height - game::consts::SCREEN_HEIGHT)
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

        // draw the tooltip box
        draw_batch.fill_region(
            rltk::Rect::with_size(next_x, next_y, tt_width, tt_height),
            rltk::ColorPair::new(fg_tt, bg_tt),
            rltk::to_cp437(' '),
        );
        draw_batch.draw_hollow_box(
            rltk::Rect::with_size(next_x, next_y, tt_width, tt_height),
            rltk::ColorPair::new(fg_tt_border, bg_tt),
        );

        let mut top_offset: i32 = 1;
        if tooltip.header.is_some() {
            top_offset = 2;

            // draw header
            draw_batch.print_color_centered_at(
                rltk::Point::new(next_x + (tt_width / 2), next_y + 1),
                tooltip.header.as_ref().unwrap(),
                rltk::ColorPair::new(fg_tt, bg_tt),
            );
        }

        // draw separator if there is a header
        if tooltip.header.is_some() && !tooltip.text.is_empty() {
            draw_batch.print_color(
                rltk::Point::new(next_x, next_y + top_offset),
                // to_cp437('─'),
                "├",
                rltk::ColorPair::new(fg_tt_border, bg_tt),
            );
            draw_batch.print_color(
                rltk::Point::new(next_x + tt_width, next_y + top_offset),
                // to_cp437('─'),
                "┤",
                rltk::ColorPair::new(fg_tt_border, bg_tt),
            );
            for x in (next_x + 1)..(next_x + tt_width) {
                draw_batch.print_color(
                    rltk::Point::new(x, next_y + top_offset),
                    // to_cp437('─'),
                    "─",
                    rltk::ColorPair::new(fg_tt_border, bg_tt),
                );
            }
            top_offset += 1;
        }

        // draw text
        let text_len = tooltip.text.len() as i32;
        for (idx, s) in tooltip.text.iter().enumerate() {
            draw_batch.print_color(
                rltk::Point::new(next_x + 1, next_y + top_offset + idx as i32),
                s,
                rltk::ColorPair::new(fg_tt, bg_tt),
            );
        }

        let has_prev_content = tooltip.header.is_some() || !tooltip.text.is_empty();
        if has_prev_content && !tooltip.attributes.is_empty() {
            draw_batch.print_color(
                rltk::Point::new(next_x, next_y + top_offset + text_len),
                // to_cp437('─'),
                "├",
                rltk::ColorPair::new(fg_tt_border, bg_tt),
            );
            draw_batch.print_color(
                rltk::Point::new(next_x + tt_width, next_y + top_offset + text_len),
                // to_cp437('─'),
                "┤",
                rltk::ColorPair::new(fg_tt_border, bg_tt),
            );
            for x in (next_x + 1)..(next_x + tt_width) {
                draw_batch.print_color(
                    rltk::Point::new(x, next_y + top_offset + text_len),
                    // to_cp437('─'),
                    "─",
                    rltk::ColorPair::new(fg_tt_border, bg_tt),
                );
            }
            top_offset += 1;
        }

        for (idx, (s1, s2)) in tooltip.attributes.iter().enumerate() {
            draw_batch.print_color(
                rltk::Point::new(next_x + 1, next_y + text_len + top_offset + idx as i32),
                s1,
                rltk::ColorPair::new(fg_tt, bg_tt),
            );
            draw_batch.print_color_right(
                rltk::Point::new(
                    next_x + tt_width,
                    next_y + text_len + idx as i32 + top_offset,
                ),
                s2,
                rltk::ColorPair::new(fg_tt, bg_tt),
            );
        }

        // advance x and y coordinates for next box
        if is_render_horiz {
            let projected_x = next_x + (tt_width * x_direction);
            if projected_x > 0 && projected_x < game::consts::SCREEN_WIDTH {
                next_x = projected_x;
            } else {
                if x_direction < 0 {
                    next_x = game::consts::SCREEN_WIDTH - 1;
                } else {
                    next_x = 1;
                }
                next_y += tt_height * y_direction;
            }
        } else {
            let projected_y = next_y + 1 + (tt_height * x_direction);
            if projected_y > 0 && projected_y < game::consts::SCREEN_HEIGHT {
                next_y = projected_y;
            } else {
                if y_direction < 0 {
                    next_y = game::consts::SCREEN_HEIGHT - 1;
                } else {
                    next_y = 1;
                }
                next_x += tt_width * x_direction;
            }
        }
    }
}
