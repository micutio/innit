/*!
The genome editor provides the facilities for the player to manipulate their own DNA, provided
they have a plasmid that allows this.
*/

use crate::core::game_state::GameState;
use crate::entity::genetics::{Dna, TraitFamily};
use crate::game::{RunState, HUD_CON, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::ui::color::Color;
use crate::ui::color_palette::ColorPalette;
use crate::util::modulus;
use rltk::{to_cp437, ColorPair, DrawBatch, Point, Rect, Rltk, VirtualKeyCode};
use std::ops::Add;

const TOP_ROW_Y_OFFSET: i32 = 1;
const MID_ROW_Y_OFFSET: i32 = 4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GenomeEditingState {
    ChooseGene,
    ChooseFunction,
    Move,
    Cut,
    FlipBit,
    Duplicate,
    Confirm,
    Cancel,
}

#[derive(Debug)]
struct EditFunction {
    layout: Rect,
    is_enabled: bool,
    state: GenomeEditingState,
    idx: usize,
    title: String,
}

impl EditFunction {
    fn new(
        layout: Rect,
        is_enabled: bool,
        state: GenomeEditingState,
        idx: usize,
        title: String,
    ) -> Self {
        EditFunction {
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
    layout: Rect,
    /// position of the represented gene within the genome
    gene_idx: usize,
    color: Color,
}

impl GeneItem {
    fn new(layout: Rect, gene_idx: usize, color: Color) -> Self {
        GeneItem {
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
    layout: Rect,
    /// The used plasmid determines which operations the genome editor can perform.
    /// Not all plasmids offer the same capabilities.
    /// How many operations can be performed with the editor.
    /// Plasmids are consumed when using the editor to avoid infinite amounts of gene editing.
    plasmid_charges: usize,
    selected_gene: usize,
    gene_selection_locked: bool,
    selected_function: usize,
    pub state: GenomeEditingState,
    pub player_dna: Dna,
    clipboard: Option<GeneItem>,
    edit_functions: Vec<EditFunction>,
    gene_items: Vec<GeneItem>,
}

// TODO: Take into account limited 'charges' of plasmids.
impl GenomeEditor {
    // required functions
    // - constructor
    // - render items
    // - methods for each edit function
    //   - move, cut, flip bit, copy, add new

    /// Creates a new instance of the editor. DNA is parsed into items that can be cycled through.
    pub fn new(dna: Dna, charges: usize, cp: &ColorPalette) -> Self {
        use GenomeEditingState::*;
        let mut top_row_x = 1;
        let enabled_functions: Vec<GenomeEditingState> =
            vec![Move, Cut, FlipBit, Duplicate, Confirm, Cancel];
        let mut edit_functions: Vec<EditFunction> =
            [Move, Cut, FlipBit, Duplicate, Confirm, Cancel]
                .iter()
                .enumerate()
                .zip(["Move", "Cut", "Mutate", "Duplicate", "Confirm", "Cancel"].iter())
                .map(|((idx, e), s)| {
                    let len: i32 = (s.len() + 3) as i32;
                    let is_enabled = enabled_functions.contains(e);
                    let item = EditFunction::new(
                        Rect::with_size(top_row_x, TOP_ROW_Y_OFFSET, len, 1),
                        is_enabled,
                        *e,
                        idx,
                        s.to_string(),
                    );
                    top_row_x += len + 2;
                    item
                })
                .collect();

        // calculate layout for whole window
        let mut func_width: i32 = edit_functions.iter().map(|item| item.layout.width()).sum();
        func_width += edit_functions.len() as i32 * 2 + 2;
        let total_width = func_width.max(dna.simplified.len() as i32 + 2);
        let total_height = 11;
        let layout = Rect::with_size(
            (SCREEN_WIDTH / 2) - (total_width / 2),
            (SCREEN_HEIGHT / 2) - (total_height / 2),
            total_width,
            total_height,
        );

        let layout_top_row = Rect::with_size(layout.x1, layout.y1 + TOP_ROW_Y_OFFSET, 0, 0);
        for mut item in &mut edit_functions {
            item.layout = item.layout.add(layout_top_row);
        }

        let gene_items = GenomeEditor::build_gene_items(
            &dna,
            cp,
            layout.x1 + 1,
            layout.y1 + MID_ROW_Y_OFFSET + 1,
        );

        GenomeEditor {
            layout,
            plasmid_charges: charges,
            selected_gene: 0,
            gene_selection_locked: false,
            selected_function: 0,
            state: GenomeEditingState::ChooseGene,
            player_dna: dna,
            clipboard: None,
            edit_functions,
            gene_items,
        }
    }

    fn build_gene_items(dna: &Dna, cp: &ColorPalette, start_x: i32, y: i32) -> Vec<GeneItem> {
        let mut x = start_x;
        dna.simplified
            .iter()
            .enumerate()
            .map(|(idx, i)| {
                let col: (u8, u8, u8) = match i.trait_family {
                    TraitFamily::Sensing => cp.cyan,
                    TraitFamily::Processing => cp.magenta,
                    TraitFamily::Actuating => cp.yellow,
                    TraitFamily::Junk => (100, 100, 100), // TODO: coloring
                    TraitFamily::Ltr => (255, 255, 255),  // TODO: coloring
                };
                let item = GeneItem::new(Rect::with_size(x, y, 1, 1), idx, Color::from(col));
                x += 1;
                item
            })
            .collect()
    }

    pub fn display(
        self,
        game_state: &mut GameState,
        ctx: &mut Rltk,
        cp: &ColorPalette,
    ) -> RunState {
        // 1. render everything
        // TODO: Implement rendering of gene properties
        self.render(ctx, cp);

        // 2. read user input and process
        self.read_input(game_state, ctx, cp)
    }

    fn render(&self, ctx: &mut Rltk, palette: &ColorPalette) {
        ctx.set_active_console(HUD_CON);
        ctx.cls();
        let mut draw_batch = DrawBatch::new();
        // draw window border
        draw_batch.fill_region(
            self.layout,
            ColorPair::new(palette.fg_hud, palette.bg_hud),
            to_cp437(' '),
        );
        draw_batch.draw_hollow_box(self.layout, ColorPair::new(palette.fg_hud, palette.bg_hud));

        // draw title
        draw_batch.print_color_centered_at(
            Point::new(SCREEN_WIDTH / 2, self.layout.y1),
            " Genome Manipulation ",
            ColorPair::new(palette.fg_hud_highlight, palette.bg_hud),
        );

        // draw 'functions'
        draw_batch.print_color(
            Point::new(self.layout.x1 + 1, self.layout.y1 + TOP_ROW_Y_OFFSET),
            "Functions",
            ColorPair::new(palette.fg_hud, palette.bg_hud),
        );

        // draw 'DNA'
        draw_batch.print_color(
            Point::new(self.layout.x1 + 1, self.layout.y1 + MID_ROW_Y_OFFSET),
            "DNA",
            ColorPair::new(palette.fg_hud, palette.bg_hud),
        );

        for item in &self.edit_functions {
            let bg_col = if !item.is_enabled {
                palette.bg_hud
            } else if item.idx == self.selected_function {
                palette.bg_bar
            } else if item.state == self.state {
                palette.yellow
            } else {
                palette.bg_hud_content
            };

            let fg_col = if item.state == self.state {
                palette.fg_hud_highlight
            } else {
                palette.fg_hud
            };

            draw_batch.fill_region(
                Rect::with_size(
                    item.layout.x1,
                    item.layout.y1,
                    item.layout.width(),
                    item.layout.height() - 1,
                ),
                ColorPair::new(fg_col, bg_col),
                to_cp437(' '),
            );
            draw_batch.print_color(
                Point::new(item.layout.x1 + 1, item.layout.y1),
                item.idx.to_string(),
                ColorPair::new(palette.fg_hud_highlight, bg_col),
            );
            draw_batch.print_color(
                Point::new(item.layout.x1 + 3, item.layout.y1),
                item.title.to_string(),
                ColorPair::new(palette.fg_hud, bg_col),
            );
        }

        for item in &self.gene_items {
            let c: char = if modulus(item.gene_idx, 2) == 0 {
                '►'
            } else {
                '◄'
            };

            let bg_color = if item.gene_idx == self.selected_gene {
                palette.bg_hud_content
            } else if self.gene_selection_locked {
                palette.bg_hud_selected
            } else {
                palette.bg_dna
            };

            draw_batch.print_color(
                item.layout.center(),
                c,
                ColorPair::new(item.color, bg_color),
            );
        }

        // draw line between gene and info box
        let item_layout = self.gene_items.get(self.selected_gene).unwrap().layout;
        let connect_start = Point::new(item_layout.x1, item_layout.y1 + 1);
        let connect_end = Point::new(self.layout.x1 + 1, self.layout.y1 + MID_ROW_Y_OFFSET + 2);

        if connect_start.x == connect_end.x {
            draw_batch.print_color(
                connect_end,
                "│",
                ColorPair::new(palette.fg_hud, palette.bg_hud),
            );
        } else {
            draw_batch.print_color(
                Point::new(connect_start.x, connect_end.y),
                "┘",
                ColorPair::new(palette.fg_hud, palette.bg_hud),
            );
            draw_batch.print_color(
                connect_end,
                "┌",
                ColorPair::new(palette.fg_hud, palette.bg_hud),
            );
            for i in 1..(connect_start.x - connect_end.x) {
                draw_batch.print_color(
                    Point::new(connect_end.x + i, connect_end.y),
                    "─",
                    ColorPair::new(palette.fg_hud, palette.bg_hud),
                );
            }
        }

        // draw genome info box
        // TODO: Draw bit vector at the bottom of this!
        if let Some(gene_item) = self.gene_items.get(self.selected_gene) {
            if let Some(genome) = self.player_dna.simplified.get(gene_item.gene_idx) {
                let highlight = ColorPair::new(palette.fg_hud_highlight, palette.bg_hud);
                let color = ColorPair::new(palette.fg_hud, palette.bg_hud);
                let name_header = "trait name:";
                let family_header = "trait family:";
                let action_header = "action:";
                let attribute_header = "attribute:";
                draw_batch.print_color(Point::new(connect_end.x, connect_end.y + 1), "├", color);
                draw_batch.print_color(Point::new(connect_end.x, connect_end.y + 2), "├", color);
                draw_batch.print_color(Point::new(connect_end.x, connect_end.y + 3), "├", color);
                draw_batch.print_color(Point::new(connect_end.x, connect_end.y + 4), "└", color);
                let spacing = [name_header, family_header, action_header, attribute_header]
                    .iter()
                    .map(|v| v.len())
                    .max()
                    .unwrap() as i32
                    + 3;
                draw_batch.print_color(
                    Point::new(connect_end.x + 2, connect_end.y + 1),
                    name_header,
                    highlight,
                );
                draw_batch.print_color(
                    Point::new(connect_end.x + spacing, connect_end.y + 1),
                    &genome.trait_name,
                    color,
                );
                draw_batch.print_color(
                    Point::new(connect_end.x + 2, connect_end.y + 2),
                    family_header,
                    highlight,
                );
                draw_batch.print_color(
                    Point::new(connect_end.x + spacing, connect_end.y + 2),
                    &genome.trait_family,
                    color,
                );
                draw_batch.print_color(
                    Point::new(connect_end.x + 2, connect_end.y + 3),
                    action_header,
                    highlight,
                );
                if let Some(action) = &genome.action {
                    draw_batch.print_color(
                        Point::new(connect_end.x + spacing, connect_end.y + 3),
                        format!("{}", action.get_identifier(),),
                        color,
                    );
                } else {
                    draw_batch.print_color(
                        Point::new(connect_end.x + spacing, connect_end.y + 3),
                        "none",
                        color,
                    );
                }
                draw_batch.print_color(
                    Point::new(connect_end.x + 2, connect_end.y + 4),
                    attribute_header,
                    highlight,
                );
                draw_batch.print_color(
                    Point::new(connect_end.x + spacing, connect_end.y + 4),
                    format!("{:#?}", genome.attribute),
                    color,
                );
            }
        }

        draw_batch.submit(6000).unwrap();
    }

    fn read_input(
        mut self,
        game_state: &mut GameState,
        ctx: &mut Rltk,
        cp: &ColorPalette,
    ) -> RunState {
        // wait for user input
        // a) keyboard input
        // if we have a key activity, process and return immediately
        use GenomeEditingState::*;
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Up => {
                    if let ChooseGene = self.state {
                        self.state = ChooseFunction
                    }
                }
                VirtualKeyCode::Down => {
                    if let ChooseFunction = self.state {
                        self.state = ChooseGene
                    }
                }
                VirtualKeyCode::Key1 => return self.do_action(game_state, cp, 0),
                VirtualKeyCode::Key2 => return self.do_action(game_state, cp, 1),
                VirtualKeyCode::Key3 => return self.do_action(game_state, cp, 2),
                VirtualKeyCode::Key4 => return self.do_action(game_state, cp, 3),
                VirtualKeyCode::Key5 => return self.do_action(game_state, cp, 4),
                VirtualKeyCode::Key6 => return self.do_action(game_state, cp, 5),
                VirtualKeyCode::Key7 => return self.do_action(game_state, cp, 6),
                VirtualKeyCode::Key8 => return self.do_action(game_state, cp, 7),
                VirtualKeyCode::Key9 => return self.do_action(game_state, cp, 8),
                VirtualKeyCode::Key0 => return self.do_action(game_state, cp, 9),
                VirtualKeyCode::Left => match self.state {
                    GenomeEditingState::Move => {
                        // if selected is leftmost, then do nothing
                        // otherwise take out of the vector and insert at idx-1
                        // TODO: Pull out into own function
                        if self.selected_gene > 0 {
                            // self.gene_items[idx].gene_idx = idx + 1;
                            // self.gene_items[idx + 1].gene_idx = idx;
                            // self.gene_items.swap(idx, idx + 1);
                            self.player_dna
                                .simplified
                                .swap(self.selected_gene, self.selected_gene - 1);
                            self.selected_gene -= 1;
                            self.regenerate_dna(game_state, cp);
                        }
                    }
                    GenomeEditingState::ChooseFunction => {
                        let mut new_idx = self.selected_function;
                        while new_idx > 0 {
                            new_idx -= 1;
                            if let Some(item) = self.edit_functions.get(new_idx) {
                                if item.is_enabled {
                                    self.selected_function = new_idx;
                                }
                            }
                        }
                    }
                    GenomeEditingState::ChooseGene => {
                        if self.selected_gene > 0 {
                            self.selected_gene -= 1;
                        }
                    }
                    _ => {}
                },
                VirtualKeyCode::Right => match self.state {
                    GenomeEditingState::Move => {
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
                            self.regenerate_dna(game_state, cp);
                        }
                    }
                    GenomeEditingState::ChooseFunction => {
                        let mut new_idx = self.selected_function;
                        while new_idx <= self.edit_functions.len() - 1 {
                            new_idx += 1;
                            if let Some(item) = self.edit_functions.get(new_idx) {
                                if item.is_enabled {
                                    self.selected_function = new_idx;
                                }
                            }
                        }
                    }
                    GenomeEditingState::ChooseGene => {
                        self.selected_gene =
                            usize::min(self.selected_gene + 1, self.gene_items.len() - 1);
                    }
                    _ => {}
                },
                VirtualKeyCode::Return => {
                    // use dummy value, this function will call itself with the correct value.
                    return match self.state {
                        ChooseGene => {
                            self.gene_selection_locked != self.gene_selection_locked;
                            RunState::GenomeEditing(self)
                        }
                        _ => self.do_action(game_state, cp, self.selected_function),
                    };
                }
                VirtualKeyCode::Escape => return RunState::CheckInput,
                _ => {}
            }
            return RunState::GenomeEditing(self);
        }

        // b) mouse input
        let hovered_function = if let Some(item) = self
            .edit_functions
            .iter()
            .find(|i| i.layout.point_in_rect(ctx.mouse_point()) && i.is_enabled)
        {
            Some(item.idx)
        } else {
            None
        };

        if let Some(idx) = hovered_function {
            // update active index
            self.selected_function = idx;
            // self.selected_function = Some(idx);
            self.state = ChooseFunction;
            if ctx.left_click {
                return self.do_action(game_state, cp, idx);
            }
        }

        // only look for new selected genes if the selection is not locked
        if !self.gene_selection_locked {
            let hovered_gene = if let Some(item) = self
                .gene_items
                .iter()
                .find(|i| i.layout.point_in_rect(ctx.mouse_point()))
            {
                Some(item.gene_idx)
            } else {
                None
            };

            if let Some(target_idx) = hovered_gene {
                // if in state MOVE, first move and then update hovered gene idx
                // update active index
                // self.hovered_gene = idx;
                match self.state {
                    Move => {
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
                            self.regenerate_dna(game_state, cp);
                        }
                    }
                    _ => {
                        self.state = ChooseGene;
                    }
                }

                if ctx.left_click {
                    match self.state {
                        Move => {}
                        _ => {
                            self.state = ChooseGene;
                            self.selected_gene = Some(target_idx);
                        }
                    }
                }
            }
        }

        RunState::GenomeEditing(self)
    }

    fn do_action(mut self, game_state: &mut GameState, cp: &ColorPalette, idx: usize) -> RunState {
        if let Some(item) = self.edit_functions.get(idx) {
            // self.state = item.state;
            use GenomeEditingState::*;
            match item.state {
                Move => {
                    if let Move = self.state {
                        // finalise move action
                        self.state = ChooseFunction;
                        self.selected_function = None;
                        self.plasmid_charges -= 1;
                    } else {
                        // start move action
                        self.state = Move;
                        self.selected_function = Some(item.idx);
                        if self.selected_gene.is_none() {
                            self.selected_gene = Some(self.selected_gene);
                        }
                    }
                }
                Cut => {
                    // get currently selected gene and remove it
                    if let Some(selected) = self.selected_gene {
                        self.player_dna.simplified.remove(selected);
                        self.selected_function = None;
                        self.selected_gene = None;
                        self.selected_gene = usize::min(0, self.selected_gene - 1);
                        self.plasmid_charges -= 1;
                        self.regenerate_dna(game_state, cp);
                    }
                }
                FlipBit => {
                    if let Some(selected) = self.selected_gene {
                        if let Some(item) = self.gene_items.get(selected) {
                            if let Some(g_trait) = self.player_dna.simplified.get(item.gene_idx) {
                                let gene_bits: Vec<u8> = game_state
                                    .gene_library
                                    .dna_from_traits(&[g_trait.trait_name.to_string()]);
                                let new_dna: Dna = game_state
                                    .gene_library
                                    .decode_dna(self.player_dna.dna_type, &gene_bits)
                                    .3;
                                if let Some(new_repr) = new_dna.simplified.get(0) {
                                    // std::mem::replace(
                                    //     &mut &self.player_dna.simplified[selected],
                                    //     new_repr,
                                    // );
                                    self.player_dna.simplified[selected] = new_repr.clone();
                                    self.selected_function = None;
                                    self.plasmid_charges -= 1;
                                    self.regenerate_dna(game_state, cp);
                                }
                            }
                        }
                    }
                }
                Duplicate => {
                    if let Some(selected) = self.selected_gene {
                        if let Some(item) = self.gene_items.get(selected) {
                            let new_gene = item.clone();
                            self.gene_items.insert(selected + 1, new_gene);
                            self.plasmid_charges -= 1;
                            self.regenerate_dna(game_state, cp);
                        }
                    }
                }
                Confirm => {
                    // apply changed genome to player
                    self.state = Confirm;
                    return RunState::GenomeEditing(self);
                }
                Cancel => {
                    // discard and return to input listening
                    return RunState::CheckInput;
                }

                _ => {}
            }
        }
        RunState::GenomeEditing(self)
    }

    /// Re-build the player dna from the current simplified representation.
    fn regenerate_dna(&mut self, game_state: &mut GameState, cp: &ColorPalette) {
        let simplified: Vec<String> = self
            .player_dna
            .simplified
            .iter()
            .map(|g| g.trait_name.clone())
            .collect();
        let bit_vec = game_state.gene_library.dna_from_traits(&simplified);
        let new_dna = game_state
            .gene_library
            .decode_dna(self.player_dna.dna_type, &bit_vec);
        self.player_dna = new_dna.3;
        self.gene_items = GenomeEditor::build_gene_items(
            &self.player_dna,
            cp,
            self.layout.x1 + 1,
            self.layout.y1 + MID_ROW_Y_OFFSET + 1,
        );
    }
}
