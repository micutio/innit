//! The genome editor provides the facilities for the player to manipulate their own DNA, provided
//! they have a plasmid that allows this.

use crate::entity::genetics::{Dna, GeneticTrait, TraitAttribute, TraitFamily};
use crate::game::{self, ObjectStore, State};
use crate::rand::Rng;
use crate::ui;
use crate::util::random::RngExtended;

use bracket_lib::prelude as rltk;
use std::ops::Add;

const TOP_ROW_Y_OFFSET: i32 = 1;
const MID_ROW_Y_OFFSET: i32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditingState {
    ChooseGene,
    ChooseFunction,
    Move,
    Cut,
    FlipBit,
    Duplicate,
    Done,
}

#[derive(Debug)]
struct EditFunction {
    layout:     rltk::Rect,
    is_enabled: bool,
    state:      EditingState,
    idx:        usize,
    title:      String,
}

impl EditFunction {
    const fn new(
        layout: rltk::Rect,
        is_enabled: bool,
        state: EditingState,
        idx: usize,
        title: String,
    ) -> Self {
        Self {
            layout,
            is_enabled,
            state,
            idx,
            title,
        }
    }
}

#[derive(Debug, Clone)]
struct GeneItem {
    layout:   rltk::Rect,
    /// position of the represented gene within the genome
    gene_idx: usize,
    color:    ui::Rgba,
}

impl GeneItem {
    const fn new(layout: rltk::Rect, gene_idx: usize, color: ui::Rgba) -> Self {
        Self {
            layout,
            gene_idx,
            color,
        }
    }
}

/// Layout from top to bottom:
/// - row of buttons for edit functions
/// - row of DNA, rendered like in the sidebar
/// - space for detailed view of the gene
///   - complete binary representation
///   - name and family of the trait
///   - values and additional info about the trait
#[derive(Debug)]
pub struct GenomeEditor {
    layout:                rltk::Rect,
    gene_selection_locked: bool,
    /// Not all plasmids offer the same capabilities.
    /// The used plasmid determines which operations the genome editor can perform.
    /// How many operations can be performed with the editor.
    /// Plasmids are consumed when using the editor to avoid infinite amounts of gene editing.
    plasmid_charges:       usize,
    selected_gene:         usize,
    selected_function:     usize,
    pub state:             EditingState,
    pub player_dna:        Dna,
    edit_functions:        Vec<EditFunction>,
    gene_items:            Vec<GeneItem>,
}

impl GenomeEditor {
    // required functions
    // - constructor
    // - render items
    // - methods for each edit function
    //   - move, cut, flip bit, copy, add new

    /// Creates a new instance of the editor. DNA is parsed into items that can be cycled through.
    pub fn new(dna: Dna, charges: usize) -> Self {
        use EditingState::{Cut, Done, Duplicate, FlipBit, Move};
        let mut top_row_x = 1;
        let enabled_functions: Vec<EditingState> = vec![Move, Cut, FlipBit, Duplicate, Done];
        let mut edit_functions: Vec<EditFunction> = [Move, Cut, FlipBit, Duplicate, Done]
            .iter()
            .enumerate()
            .zip(["Move", "Cut", "Mutate", "Duplicate", "Done"].iter())
            .map(|((idx, e), s)| {
                let len: i32 = (s.len() + 3) as i32;
                let is_enabled = enabled_functions.contains(e);
                let item = EditFunction::new(
                    rltk::Rect::with_size(top_row_x, TOP_ROW_Y_OFFSET, len, 1),
                    is_enabled,
                    *e,
                    idx,
                    (*s).to_string(),
                );
                top_row_x += len + 2;
                item
            })
            .collect();

        // calculate layout for whole window
        let mut func_width: i32 = edit_functions.iter().map(|item| item.layout.width()).sum();
        func_width += edit_functions.len() as i32 * 2 + 2;
        let total_width = func_width.max(dna.simplified.len() as i32 + 2);
        let total_height = 16;
        let layout = rltk::Rect::with_size(
            (game::consts::SCREEN_WIDTH / 2) - (total_width / 2),
            (game::consts::SCREEN_HEIGHT / 2) - (total_height / 2),
            total_width,
            total_height,
        );

        let layout_top_row = rltk::Rect::with_size(layout.x1, layout.y1 + TOP_ROW_Y_OFFSET, 0, 1);
        for item in &mut edit_functions {
            item.layout = item.layout.add(layout_top_row);
        }

        let gene_items =
            Self::build_gene_items(&dna, layout.x1 + 1, layout.y1 + MID_ROW_Y_OFFSET + 1);

        Self {
            layout,
            plasmid_charges: charges,
            selected_gene: 0,
            gene_selection_locked: false,
            selected_function: 0,
            state: EditingState::ChooseFunction,
            player_dna: dna,
            edit_functions,
            gene_items,
        }
    }

    fn build_gene_items(dna: &Dna, start_x: i32, y: i32) -> Vec<GeneItem> {
        let mut x = start_x;
        let cyan = ui::palette().hud_fg_dna_processor;
        let magenta = ui::palette().hud_fg_dna_actuator;
        let yellow = ui::palette().hud_fg_dna_sensor;
        dna.simplified
            .iter()
            .enumerate()
            .map(|(idx, i)| {
                let col: ui::Rgba = match i.trait_family {
                    TraitFamily::Sensing => cyan,
                    TraitFamily::Processing => magenta,
                    TraitFamily::Actuating => yellow,
                    TraitFamily::Junk(_) => ui::Rgba::new(100, 100, 100, 255), // TODO: coloring
                    TraitFamily::Ltr => ui::Rgba::new(255, 255, 255, 255),     // TODO: coloring
                };
                let item = GeneItem::new(rltk::Rect::with_size(x, y, 1, 1), idx, col);
                x += 1;
                item
            })
            .collect()
    }

    pub fn display(self, game_state: &mut State, ctx: &mut rltk::BTerm) -> game::RunState {
        // 1. render everything
        self.render(game_state, ctx);

        // 2. read user input and process
        self.read_input(game_state, ctx)
    }

    #[allow(clippy::too_many_lines)]
    fn render(&self, game_state: &mut State, ctx: &mut rltk::BTerm) {
        ctx.set_active_console(game::consts::HUD_CON);
        ctx.cls();
        let mut draw_batch = rltk::DrawBatch::new();
        let hud_fg = ui::palette().hud_fg;
        let hud_fg_hl = ui::palette().hud_fg_highlight;
        let hud_guide = ui::palette().hud_fg_highlight;
        let hud_bg_active = ui::palette().hud_bg_active;
        let hud_fg_inactive = ui::palette().hud_fg_inactive;
        let hud_fg_border = ui::palette().hud_fg_border;
        let hud_bg = ui::palette().hud_bg;
        // draw window border
        draw_batch.fill_region(
            self.layout,
            rltk::ColorPair::new(hud_fg, hud_bg),
            rltk::to_cp437(' '),
        );
        draw_batch.draw_hollow_box(self.layout, rltk::ColorPair::new(hud_fg_border, hud_bg));

        // draw title
        draw_batch.print_color(
            rltk::Point::new(self.layout.x1 + 2, self.layout.y1),
            " Genome Manipulation ",
            rltk::ColorPair::new(hud_fg_border, hud_bg),
        );

        if self.state == EditingState::ChooseFunction {
            draw_batch.fill_region(
                rltk::Rect::with_size(
                    self.layout.x1 + 1,
                    self.layout.y1 + TOP_ROW_Y_OFFSET + 1,
                    self.layout.width() - 2,
                    1,
                ),
                rltk::ColorPair::new(hud_fg, hud_bg_active),
                rltk::to_cp437(' '),
            );
        } else if self.state == EditingState::ChooseGene {
            draw_batch.fill_region(
                rltk::Rect::with_size(
                    self.layout.x1 + 1,
                    self.layout.y1 + MID_ROW_Y_OFFSET + 1,
                    self.layout.width() - 2,
                    1,
                ),
                rltk::ColorPair::new(hud_fg, hud_bg_active),
                rltk::to_cp437(' '),
            );
        }

        // draw 'functions'
        draw_batch.print_color(
            rltk::Point::new(self.layout.x1 + 1, self.layout.y1 + TOP_ROW_Y_OFFSET),
            "Functions",
            rltk::ColorPair::new(hud_fg, hud_bg),
        );

        // draw 'DNA'
        draw_batch.print_color(
            rltk::Point::new(self.layout.x1 + 1, self.layout.y1 + MID_ROW_Y_OFFSET),
            "DNA",
            rltk::ColorPair::new(hud_fg, hud_bg),
        );

        for item in &self.edit_functions {
            let bg_col = if self.state == EditingState::ChooseFunction {
                hud_bg_active
            } else if self.state == item.state {
                hud_fg_hl
            } else {
                hud_bg
            };

            let fg_col = if !item.is_enabled {
                hud_fg_inactive
            } else if self.state == item.state {
                hud_fg
            } else if self.selected_function == item.idx {
                hud_fg_hl
            } else {
                hud_fg
            };

            draw_batch.fill_region(
                rltk::Rect::with_size(
                    item.layout.x1,
                    item.layout.y1,
                    item.layout.width(),
                    item.layout.height(),
                ),
                rltk::ColorPair::new(fg_col, bg_col),
                rltk::to_cp437(' '),
            );
            draw_batch.print_color(
                rltk::Point::new(item.layout.x1 + 1, item.layout.y1),
                (item.idx + 1).to_string(),
                rltk::ColorPair::new(fg_col, bg_col),
            );
            draw_batch.print_color(
                rltk::Point::new(item.layout.x1 + 3, item.layout.y1),
                item.title.to_string(),
                rltk::ColorPair::new(fg_col, bg_col),
            );
        }

        for item in &self.gene_items {
            let c: char = if item.gene_idx % 2 == 0 { '►' } else { '◄' };

            let bg_color = if item.gene_idx == self.selected_gene {
                hud_fg_hl
            } else if self.state == EditingState::ChooseGene || self.state == EditingState::Move {
                hud_bg_active
            } else {
                hud_bg
            };

            draw_batch.print_color(
                item.layout.center(),
                c,
                rltk::ColorPair::new(item.color, bg_color),
            );
        }

        // draw line between gene and info box
        let item_layout = self.gene_items.get(self.selected_gene).unwrap().layout;
        let connect_start = rltk::Point::new(item_layout.x1, item_layout.y1 + 1);
        let connect_end =
            rltk::Point::new(self.layout.x1 + 1, self.layout.y1 + MID_ROW_Y_OFFSET + 2);

        if connect_start.x == connect_end.x {
            draw_batch.print_color(connect_end, "│", rltk::ColorPair::new(hud_guide, hud_bg));
        } else {
            draw_batch.print_color(
                rltk::Point::new(connect_start.x, connect_end.y),
                "┘",
                rltk::ColorPair::new(hud_guide, hud_bg),
            );
            draw_batch.print_color(connect_end, "┌", rltk::ColorPair::new(hud_guide, hud_bg));
            for i in 1..(connect_start.x - connect_end.x) {
                draw_batch.print_color(
                    rltk::Point::new(connect_end.x + i, connect_end.y),
                    "─",
                    rltk::ColorPair::new(hud_guide, hud_bg),
                );
            }
        }

        // draw genome info box
        if let Some(gene_item) = self.gene_items.get(self.selected_gene) {
            if let Some(genome) = self.player_dna.simplified.get(gene_item.gene_idx) {
                let col_hl = rltk::ColorPair::new(hud_fg_hl, hud_bg);
                let col_guide = rltk::ColorPair::new(hud_guide, hud_bg);
                let color = rltk::ColorPair::new(hud_fg, hud_bg);
                let name_header = "trait name:";
                let family_header = "trait family:";
                let action_header = "action:";
                let attribute_header = "attribute:";
                let code_header = "genetic code:";
                let trait_name: String = if TraitAttribute::Receptor == genome.attribute {
                    format!("{}-({})", genome.trait_name, genome.position)
                } else {
                    genome.trait_name.clone()
                };
                draw_batch.print_color(
                    rltk::Point::new(connect_end.x, connect_end.y + 1),
                    "├",
                    col_guide,
                );
                draw_batch.print_color(
                    rltk::Point::new(connect_end.x, connect_end.y + 2),
                    "├",
                    col_guide,
                );
                draw_batch.print_color(
                    rltk::Point::new(connect_end.x, connect_end.y + 3),
                    "├",
                    col_guide,
                );
                draw_batch.print_color(
                    rltk::Point::new(connect_end.x, connect_end.y + 4),
                    "├",
                    col_guide,
                );
                draw_batch.print_color(
                    rltk::Point::new(connect_end.x, connect_end.y + 5),
                    "└",
                    col_guide,
                );
                let spacing = [name_header, family_header, action_header, attribute_header]
                    .iter()
                    .map(|v| v.len())
                    .max()
                    .unwrap() as i32
                    + 3;
                draw_batch.print_color(
                    rltk::Point::new(connect_end.x + 2, connect_end.y + 1),
                    name_header,
                    col_hl,
                );
                draw_batch.print_color(
                    rltk::Point::new(connect_end.x + spacing, connect_end.y + 1),
                    trait_name,
                    color,
                );
                draw_batch.print_color(
                    rltk::Point::new(connect_end.x + 2, connect_end.y + 2),
                    family_header,
                    col_hl,
                );
                draw_batch.print_color(
                    rltk::Point::new(connect_end.x + spacing, connect_end.y + 2),
                    genome.trait_family,
                    color,
                );
                draw_batch.print_color(
                    rltk::Point::new(connect_end.x + 2, connect_end.y + 3),
                    action_header,
                    col_hl,
                );
                if let Some(action) = &genome.action {
                    draw_batch.print_color(
                        rltk::Point::new(connect_end.x + spacing, connect_end.y + 3),
                        action.get_identifier(),
                        color,
                    );
                } else {
                    draw_batch.print_color(
                        rltk::Point::new(connect_end.x + spacing, connect_end.y + 3),
                        "none",
                        color,
                    );
                }
                draw_batch.print_color(
                    rltk::Point::new(connect_end.x + 2, connect_end.y + 4),
                    attribute_header,
                    col_hl,
                );
                draw_batch.print_color(
                    rltk::Point::new(connect_end.x + spacing, connect_end.y + 4),
                    format!("{:#?}", genome.attribute),
                    color,
                );
                draw_batch.print_color(
                    rltk::Point::new(connect_end.x + 2, connect_end.y + 5),
                    code_header,
                    col_hl,
                );

                if let Some(item) = self.gene_items.get(self.selected_gene) {
                    if let Some(g_trait) = self.player_dna.simplified.get(item.gene_idx) {
                        let gene_bits: Vec<u8> =
                            game_state.gene_library.dna_from_trait_refs(&[g_trait]);
                        let gene_str: String = gene_bits
                            .iter()
                            .map(|b| format!("{:08b}", b))
                            .collect::<String>();

                        draw_batch.print_color(
                            rltk::Point::new(connect_end.x + spacing, connect_end.y + 5),
                            format!("{:#?}", gene_str),
                            color,
                        );
                    }
                }
            }
        }

        // draw controls info
        let info_x = self.layout.x1 + 1;
        let info_y = self.layout.y2 - 3;
        draw_batch.print_color(
            rltk::Point::new(info_x, info_y),
            "↑/↓ - flip between functions/DNA",
            rltk::ColorPair::new(hud_fg, hud_bg),
        );
        draw_batch.print_color(
            rltk::Point::new(info_x, info_y + 1),
            "←/→ - choose function/gene",
            rltk::ColorPair::new(hud_fg, hud_bg),
        );
        draw_batch.print_color(
            rltk::Point::new(info_x, info_y + 2),
            "return - use function",
            rltk::ColorPair::new(hud_fg, hud_bg),
        );

        draw_batch.submit(game::consts::HUD_CON_Z).unwrap();
    }

    #[allow(clippy::too_many_lines)]
    fn read_input(mut self, game_state: &mut State, ctx: &mut rltk::BTerm) -> game::RunState {
        // wait for user input
        // a) keyboard input
        // if we have a key activity, process and return immediately
        if let Some(key) = ctx.key {
            match key {
                rltk::VirtualKeyCode::Up => {
                    if self.state == EditingState::ChooseGene {
                        self.state = EditingState::ChooseFunction;
                    }
                }
                rltk::VirtualKeyCode::Down => {
                    if self.state == EditingState::ChooseFunction {
                        self.state = EditingState::ChooseGene;
                    }
                }
                rltk::VirtualKeyCode::Key1 => return self.do_action(game_state, 0),
                rltk::VirtualKeyCode::Key2 => return self.do_action(game_state, 1),
                rltk::VirtualKeyCode::Key3 => return self.do_action(game_state, 2),
                rltk::VirtualKeyCode::Key4 => return self.do_action(game_state, 3),
                rltk::VirtualKeyCode::Key5 => return self.do_action(game_state, 4),
                rltk::VirtualKeyCode::Key6 => return self.do_action(game_state, 5),
                rltk::VirtualKeyCode::Key7 => return self.do_action(game_state, 6),
                rltk::VirtualKeyCode::Key8 => return self.do_action(game_state, 7),
                rltk::VirtualKeyCode::Key9 => return self.do_action(game_state, 8),
                rltk::VirtualKeyCode::Key0 => return self.do_action(game_state, 9),
                rltk::VirtualKeyCode::Left => match self.state {
                    EditingState::Move => {
                        // if selected is leftmost, then do nothing
                        // otherwise take out of the vector and insert at idx-1
                        if self.selected_gene > 0 {
                            // self.gene_items[idx].gene_idx = idx + 1;
                            // self.gene_items[idx + 1].gene_idx = idx;
                            // self.gene_items.swap(idx, idx + 1);
                            self.player_dna
                                .simplified
                                .swap(self.selected_gene, self.selected_gene - 1);
                            self.selected_gene -= 1;
                            self.regenerate_dna(game_state);
                        }
                    }
                    EditingState::ChooseFunction => {
                        let mut new_idx = self.selected_function;
                        while new_idx > 0 {
                            new_idx -= 1;
                            if let Some(item) = self.edit_functions.get(new_idx) {
                                if item.is_enabled {
                                    self.selected_function = new_idx;
                                    break;
                                }
                            }
                        }
                    }
                    EditingState::ChooseGene => {
                        if self.selected_gene > 0 {
                            self.selected_gene -= 1;
                        }
                    }
                    _ => {}
                },
                rltk::VirtualKeyCode::Right => match self.state {
                    EditingState::Move => {
                        // if selected is rightmost, then do nothing
                        // otherwise take out of the vector and insert at idx-1
                        if self.selected_gene < self.gene_items.len() {
                            // self.gene_items[idx].gene_idx = idx + 1;
                            // self.gene_items[idx + 1].gene_idx = idx;
                            // self.gene_items.swap(idx, idx + 1);
                            self.player_dna
                                .simplified
                                .swap(self.selected_gene, self.selected_gene + 1);
                            self.selected_gene += 1;
                            self.regenerate_dna(game_state);
                        }
                    }
                    EditingState::ChooseFunction => {
                        let mut new_idx = self.selected_function;
                        while new_idx < self.edit_functions.len() {
                            new_idx += 1;
                            if let Some(item) = self.edit_functions.get(new_idx) {
                                if item.is_enabled {
                                    self.selected_function = new_idx;
                                    break;
                                }
                            }
                        }
                    }
                    EditingState::ChooseGene => {
                        self.selected_gene =
                            usize::min(self.selected_gene + 1, self.gene_items.len() - 1);
                    }
                    _ => {}
                },
                rltk::VirtualKeyCode::Return => {
                    // use dummy value, this function will call itself with the correct value.
                    let function_idx: usize = self.selected_function;
                    return self.do_action(game_state, function_idx);
                }
                rltk::VirtualKeyCode::Escape => return game::RunState::CheckInput,
                _ => {}
            }
            return game::RunState::GenomeEditing(self);
        }

        // b) mouse input

        // b.1) check whether we're hovering a function and update the selected function idx
        let hovered_function = self
            .edit_functions
            .iter()
            .find(|i| i.layout.point_in_rect(ctx.mouse_point()) && i.is_enabled)
            .map(|item| item.idx);

        if let Some(idx) = hovered_function {
            // update active index
            self.selected_function = idx;
            // self.selected_function = Some(idx);
            if self.state == EditingState::ChooseGene {
                self.state = EditingState::ChooseFunction;
            }
            // if we have a function and the mouse is clicked, then perform the function
            if ctx.left_click {
                return self.do_action(game_state, idx);
            }
        }

        // b.2) Check whether we're hovering a gene item.
        //      - if we are in state `Move`, move the selected gene to the hovered position
        //      - if we are in state `ChooseGene`, only update the gene if it is not locked

        let hovered_gene = self
            .gene_items
            .iter()
            .find(|i| i.layout.point_in_rect(ctx.mouse_point()))
            .map(|item| item.gene_idx);

        if let Some(target_idx) = hovered_gene {
            match self.state {
                EditingState::Move => {
                    let step: i32 = if target_idx < self.selected_gene {
                        -1
                    } else {
                        1
                    };
                    let mut next_idx = self.selected_gene;
                    while next_idx != target_idx {
                        self.player_dna
                            .simplified
                            .swap(next_idx, (next_idx as i32 + step) as usize);
                        next_idx = (next_idx as i32 + step) as usize;
                        self.selected_gene = next_idx;
                        self.regenerate_dna(game_state);
                    }
                }
                EditingState::ChooseGene => {
                    if !self.gene_selection_locked {
                        self.selected_gene = target_idx;
                    }
                }
                _ => {
                    self.state = EditingState::ChooseGene;
                }
            }

            if ctx.left_click {
                match self.state {
                    // If we're moving, finalise it now.
                    EditingState::Move => self.state = EditingState::ChooseGene,
                    EditingState::ChooseGene => {
                        self.gene_selection_locked = !self.gene_selection_locked;
                    }
                    _ => {
                        self.state = EditingState::ChooseGene;
                        if !self.gene_selection_locked {
                            self.selected_gene = target_idx;
                        }
                    }
                }
            }
        }

        game::RunState::GenomeEditing(self)
    }

    fn do_action(mut self, game_state: &mut State, active_idx: usize) -> game::RunState {
        if let Some(item) = self.edit_functions.get(active_idx) {
            self.selected_function = item.idx;
            match item.state {
                EditingState::Move => {
                    if self.state == EditingState::Move {
                        // finalise move action
                        self.state = EditingState::ChooseFunction;
                        self.decrease_charge();
                    } else {
                        // start move action
                        self.state = EditingState::Move;
                    }
                }
                EditingState::Cut => {
                    self.player_dna.simplified.remove(self.selected_gene);
                    self.selected_gene = if self.selected_gene == 0 {
                        0
                    } else {
                        self.selected_gene - 1
                    };
                    self.decrease_charge();
                    self.regenerate_dna(game_state);
                    self.state = EditingState::ChooseFunction;
                }
                EditingState::FlipBit => {
                    if let Some(item) = self.gene_items.get(self.selected_gene) {
                        if let Some(g_trait) = self.player_dna.simplified.get(item.gene_idx) {
                            let mut gene_bits: Vec<u8> =
                                game_state.gene_library.dna_from_trait_refs(&[g_trait]);
                            let random_bit = game_state.rng.gen_range(0..gene_bits.len());
                            gene_bits[random_bit] ^= game_state.rng.random_bit();
                            let new_dna: Dna = game_state
                                .gene_library
                                .dna_to_traits(self.player_dna.dna_type, &gene_bits)
                                .3;
                            if let Some(new_repr) = new_dna.simplified.get(0) {
                                self.player_dna.simplified[self.selected_gene] = new_repr.clone();
                                self.decrease_charge();
                                self.regenerate_dna(game_state);
                            }
                        }
                    }
                    self.state = EditingState::ChooseFunction;
                }
                EditingState::Duplicate => {
                    if let Some(item) = self.gene_items.get(self.selected_gene) {
                        let mut trait_clone: Option<GeneticTrait> = None;
                        if let Some(g_trait) = self.player_dna.simplified.get(item.gene_idx) {
                            trait_clone = Some(g_trait.clone());
                        }

                        if let Some(t) = trait_clone {
                            self.player_dna.simplified.insert(self.selected_gene + 1, t);
                            self.decrease_charge();
                            self.regenerate_dna(game_state);
                        }
                    }
                    self.state = EditingState::ChooseFunction;
                }
                EditingState::Done => {
                    // apply changed genome to player
                    self.state = EditingState::Done;
                    return game::RunState::GenomeEditing(self);
                }
                _ => {}
            }
        }
        game::RunState::GenomeEditing(self)
    }

    /// Decrease the plasmid charge and update the UI accordingly
    fn decrease_charge(&mut self) {
        if self.plasmid_charges > 0 {
            self.plasmid_charges -= 1;
            if self.plasmid_charges == 0 {
                self.edit_functions
                    .iter_mut()
                    .filter(|f| f.state != EditingState::Done)
                    .for_each(|f| f.is_enabled = false);
                // set selected function to the last one; "Done" which is now the only one possible.
                self.selected_function = self.edit_functions.len() - 1;
            }
        } else {
            panic!("attempting to decrease plasmid changes below zero!");
        }
    }

    /// Re-build the player dna from the current simplified representation.
    fn regenerate_dna(&mut self, game_state: &mut State) {
        let bit_vec = game_state
            .gene_library
            .dna_from_traits(self.player_dna.simplified.as_slice());
        let new_dna = game_state
            .gene_library
            .dna_to_traits(self.player_dna.dna_type, &bit_vec);
        self.player_dna = new_dna.3;
        self.gene_items = Self::build_gene_items(
            &self.player_dna,
            self.layout.x1 + 1,
            self.layout.y1 + MID_ROW_Y_OFFSET + 1,
        );
    }
}

pub fn try_create(state: &mut State, objects: &mut ObjectStore) -> Option<GenomeEditor> {
    if let Some(ref mut player) = objects[state.player_idx] {
        let genome_editor = GenomeEditor::new(player.dna.clone(), 1);
        Some(genome_editor)
    } else {
        None
    }
}
