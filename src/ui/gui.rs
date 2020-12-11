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
    pub tooltip: Vec<String>,
    pub layout: Rect,
    pub color: ColorPair,
}

impl<T> UiItem<T> {
    pub fn new<S1: Into<String>>(
        item_enum: T,
        text: S1,
        tooltip: Vec<String>,
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
            vec!["select new primary action".to_string()],
            Rect::with_size(button_x, 6, button_len, 1),
            ColorPair::new(cp.fg_hud, cp.bg_hud_content),
        ),
        UiItem::new(
            HudItem::SecondaryAction,
            "",
            vec!["select new secondary action".to_string()],
            Rect::with_size(button_x, 7, button_len, 1),
            ColorPair::new(cp.fg_hud, cp.bg_hud_content),
        ),
        UiItem::new(
            HudItem::Quick1Action,
            "",
            vec!["select new quick action".to_string()],
            Rect::with_size(button_x, 8, button_len, 1),
            ColorPair::new(cp.fg_hud, cp.bg_hud_content),
        ),
        UiItem::new(
            HudItem::Quick2Action,
            "",
            vec!["select new quick action".to_string()],
            Rect::with_size(button_x, 9, button_len, 1),
            ColorPair::new(cp.fg_hud, cp.bg_hud_content),
        ),
    ];

    items
}

fn tooltip_from(g_trait: &GeneticTrait) -> Vec<String> {
    vec![
        format!("trait: {}", g_trait.trait_name),
        format!("group: {}", g_trait.trait_family),
    ]
}

pub struct Hud {
    layout: Rect,
    last_mouse: Point,
    pub(crate) require_refresh: bool,
    pub(crate) items: Vec<UiItem<HudItem>>,
    tooltip: Vec<String>, // TODO: Find elegant way to render this and tooltips.
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
            tooltip: Vec::new(),
        }
    }

    pub fn update_tooltips(&mut self, mouse_pos: Point, names: Vec<String>) {
        if let Some(item) = self
            .items
            .iter()
            .find(|i| i.layout.point_in_rect(mouse_pos))
        {
            self.tooltip = item.tooltip.clone()
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

            self.items.push(UiItem::new(
                HudItem::DnaItem,
                c,
                tooltip_from(g_trait),
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
            self.items.push(UiItem::new(
                HudItem::DnaItem,
                c,
                tooltip_from(g_trait),
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

fn render_ui_items(hud: &Hud, draw_batch: &mut DrawBatch) {
    for item in &hud.items {
        draw_batch.print_color(item.top_left_corner(), &item.text, item.color);
    }
}

fn render_tooltip(hud: &Hud, cp: &ColorPalette, draw_batch: &mut DrawBatch) {
    if hud.tooltip.is_empty() {
        return;
    }

    let tt_width: i32 = hud.tooltip.iter().map(|s| s.len()).max().unwrap() as i32 + 2;
    let tt_height: i32 = hud.tooltip.len() as i32 + 2;
    let tt_x: i32 = if hud.last_mouse.x + tt_width >= SCREEN_WIDTH {
        hud.last_mouse.x - tt_width - 1
    } else {
        hud.last_mouse.x + 1
    };
    let tt_y: i32 = if hud.last_mouse.y + tt_height >= SCREEN_HEIGHT {
        hud.last_mouse.y - tt_height - 1
    } else {
        hud.last_mouse.y + 1
    };

    draw_batch.fill_region(
        Rect::with_size(tt_x, tt_y, tt_width, tt_height - 1),
        ColorPair::new(cp.fg_hud, cp.bg_hud_selected),
        to_cp437(' '),
    );
    draw_batch.draw_hollow_box(
        Rect::with_size(tt_x, tt_y, tt_width, tt_height - 1),
        ColorPair::new(cp.fg_hud, cp.bg_hud_selected),
    );

    for (idx, s) in hud.tooltip.iter().enumerate() {
        draw_batch.print_color(
            Point::new(tt_x + 1, tt_y + idx as i32 + 1),
            s,
            ColorPair::new(cp.fg_hud, cp.bg_hud_selected),
        );
    }
}
