//! The genome editor provides the facilities for the player to manipulate their own DNA, provided
//! they have a plasmid that allows this.

use crate::core::game_state::GameState;
use crate::entity::genetics::{Dna, TraitFamily};
use crate::game::{RunState, HUD_CON, SCREEN_WIDTH};
use crate::ui::color::Color;
use crate::ui::color_palette::ColorPalette;
use crate::util::modulus;
use rltk::{to_cp437, ColorPair, DrawBatch, Point, Rect, Rltk, VirtualKeyCode};
use std::fmt::Formatter;

/// Determines which features of the editor are enabled.
#[derive(Debug)]
pub enum PlasmidFeatureSet {
    Reduce,  // moving and cutting
    InPlace, // editing without moving or extending, like flipping bits
    Extend,  // copy and adding new
}

#[derive(Debug, Clone, Copy)]
pub enum GenomeEditingState {
    ChooseGene,
    ChooseFunction,
    Move,
    Cut,
    FlipBit,
    Duplicate,
    Done,
    Cancel,
}

// impl Display for GenomeEditingState {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         match self {
//             GenomeEditingState::ChooseGene => {
//                 write!(f, "ChooseGene")
//             }
//             GenomeEditingState::ChooseFunction => {
//                 write!(f, "ChooseFunction")
//             }
//             GenomeEditingState::Move => {
//                 write!(f, "Move")
//             }
//             GenomeEditingState::Cut => {
//                 write!(f, "Cut")
//             }
//             GenomeEditingState::FlipBit => {
//                 write!(f, "Flip a Bit")
//             }
//             GenomeEditingState::Duplicate => {
//                 write!(f, "Duplicate")
//             }
//             GenomeEditingState::Done => {
//                 write!(f, "Done")
//             }
//         }
//     }
// }

#[derive(Debug)]
struct EditFunction {
    layout: Rect,
    state: GenomeEditingState,
    idx: usize,
    title: String,
}

impl EditFunction {
    fn new(layout: Rect, state: GenomeEditingState, idx: usize, title: String) -> Self {
        EditFunction {
            layout,
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
    plasmid_features: PlasmidFeatureSet,
    /// How many operations can be performed with the editor.
    /// Plasmids are consumed when using the editor to avoid infinite amounts of gene editing.
    plasmid_charges: usize,
    hovered_gene: usize,
    selected_gene: Option<usize>,
    hovered_function: usize,
    selected_function: Option<usize>,
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
    pub fn new(
        dna: Dna,
        layout: Rect,
        charges: usize,
        features: PlasmidFeatureSet,
        cp: &ColorPalette,
    ) -> Self {
        let mut top_row_x: i32 = 11;
        let top_row_y: i32 = 7;
        let mut mid_row_x: i32 = 11;
        let mid_row_y: i32 = 10;

        use GenomeEditingState::*;
        let edit_functions: Vec<EditFunction> = [Move, Cut, FlipBit, Duplicate, Done, Cancel]
            .iter()
            .enumerate()
            .zip(["Move", "Cut", "Mutate", "Duplicate", "Confirm", "Cancel"].iter())
            .map(|((idx, e), s)| {
                let len: i32 = (s.len() + 3) as i32;
                let item = EditFunction::new(
                    Rect::with_size(top_row_x, top_row_y, len, 0),
                    *e,
                    idx,
                    s.to_string(),
                );
                top_row_x += len + 2;
                item
            })
            .collect();

        let gene_items: Vec<GeneItem> = dna
            .simplified
            .iter()
            .enumerate()
            .map(|(idx, i)| {
                let col: (u8, u8, u8) = match i.trait_family {
                    TraitFamily::Sensing => cp.cyan,
                    TraitFamily::Processing => cp.magenta,
                    TraitFamily::Actuating => cp.yellow,
                    TraitFamily::Junk => (100, 100, 100), // TODO
                    TraitFamily::Ltr => (255, 255, 255),  // TODO
                };
                let item = GeneItem::new(
                    Rect::with_size(mid_row_x, mid_row_y, 1, 1),
                    idx,
                    Color::from(col),
                );
                mid_row_x += 1;
                item
            })
            .collect();

        GenomeEditor {
            layout,
            plasmid_features: features,
            plasmid_charges: charges,
            hovered_gene: 0,
            selected_gene: None,
            hovered_function: 0,
            selected_function: None,
            state: GenomeEditingState::ChooseGene,
            player_dna: dna,
            clipboard: None,
            edit_functions, // TODO: Create items and insert here!
            gene_items,     // TODO: Create items and insert here!
        }
    }

    pub fn display(
        self,
        game_state: &mut GameState,
        ctx: &mut Rltk,
        palette: &ColorPalette,
    ) -> RunState {
        // 1. render everything
        // TODO: Implement rendering of gene properties
        self.render(ctx, palette);

        // 2. read user input and process
        self.read_input(game_state, ctx)
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
            Point::new(self.layout.x1 + 1, self.layout.y1 + 1),
            "Functions",
            ColorPair::new(palette.fg_hud, palette.bg_hud),
        );

        // draw 'DNA'
        draw_batch.print_color(
            Point::new(self.layout.x1 + 1, self.layout.y1 + 4),
            "DNA",
            ColorPair::new(palette.fg_hud, palette.bg_hud),
        );

        // TODO: Render top row, middle row and bottom info field
        let function_idx = if let Some(i) = self.selected_function {
            i
        } else {
            self.hovered_function
        };
        for item in &self.edit_functions {
            let bg_col = if item.idx == function_idx {
                palette.bg_hud_selected
            } else {
                palette.bg_hud
            };
            draw_batch.fill_region(
                item.layout,
                ColorPair::new(palette.fg_hud, bg_col),
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

        let gene_idx = if let Some(i) = self.selected_gene {
            i
        } else {
            self.hovered_gene
        };
        for item in &self.gene_items {
            let c: char = if modulus(item.gene_idx, 2) == 0 {
                '►'
            } else {
                '◄'
            };

            let bg_color = if item.gene_idx == gene_idx {
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
        let item_layout = self.gene_items.get(self.hovered_gene).unwrap().layout;
        let connect_start = Point::new(item_layout.x1, item_layout.y1);
        let connect_end = Point::new(11, 11);

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
        if let Some(gene_item) = self.gene_items.get(self.hovered_gene) {
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

    fn read_input(mut self, game_state: &mut GameState, ctx: &mut Rltk) -> RunState {
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
                VirtualKeyCode::Key1 => return self.do_action(game_state, 0),
                VirtualKeyCode::Key2 => return self.do_action(game_state, 1),
                VirtualKeyCode::Key3 => return self.do_action(game_state, 2),
                VirtualKeyCode::Key4 => return self.do_action(game_state, 3),
                VirtualKeyCode::Key5 => return self.do_action(game_state, 4),
                VirtualKeyCode::Key6 => return self.do_action(game_state, 5),
                VirtualKeyCode::Key7 => return self.do_action(game_state, 6),
                VirtualKeyCode::Key8 => return self.do_action(game_state, 7),
                VirtualKeyCode::Key9 => return self.do_action(game_state, 8),
                VirtualKeyCode::Key0 => return self.do_action(game_state, 9),
                // TODO: implement update of hovered function/gene for left/right keys
                // TODO: implement choice of insertion for left/right keys when in 'move gene' state
                VirtualKeyCode::Left => match self.state {
                    GenomeEditingState::Move => {
                        unimplemented!()
                        // if let Some(idx) = self.selected_gene {
                        // if let Some(item1) = self.player_dna.simplified.get(idx) {
                        //     item1.
                        // }
                        // }
                    }
                    GenomeEditingState::ChooseFunction => {
                        if self.hovered_function > 0 {
                            self.hovered_function -= 1;
                        }
                    }
                    GenomeEditingState::ChooseGene => {
                        if self.hovered_gene > 0 {
                            self.hovered_gene -= 1;
                        }
                    }
                    _ => {}
                },
                VirtualKeyCode::Right => match self.state {
                    GenomeEditingState::Move => {
                        // unimplemented!();
                        // if selected is rightmost, then do nothing
                        // otherwise take out of the vector and insert at idx-1
                        if let Some(idx) = self.selected_gene {
                            self.gene_items[idx].gene_idx = idx + 1;
                            self.gene_items[idx + 1].gene_idx = idx;
                            self.gene_items.swap(idx, idx + 1);
                        }
                    }
                    GenomeEditingState::ChooseFunction => {
                        self.hovered_function =
                            usize::min(self.hovered_function + 1, self.edit_functions.len() - 1);
                    }
                    GenomeEditingState::ChooseGene => {
                        self.hovered_gene =
                            usize::min(self.hovered_gene + 1, self.gene_items.len() - 1);
                    }
                    _ => {}
                },
                VirtualKeyCode::Return => match self.state {
                    ChooseGene => {
                        if self.selected_gene.is_some() {
                            self.selected_gene = None;
                        } else {
                            self.selected_gene = Some(self.hovered_gene);
                        }
                    }
                    ChooseFunction => {
                        self.selected_function = Some(self.hovered_function);
                        if let Some(idx) = self.selected_function {
                            // self.selected_function is being re-assigned in the function call below
                            return self.do_action(game_state, idx);
                        }
                    }
                    // Move => {
                    //     if let Some(idx) = self.selected_function {
                    //         // self.selected_function is being re-assigned in the function call below
                    //         self.do_action(game_state, idx)
                    //     } else {
                    //         RunState::GenomeEditing(self)
                    //     }
                    // }
                    _ => unimplemented!(),
                },
                VirtualKeyCode::Escape => return RunState::CheckInput,
                _ => {}
            }
            return RunState::GenomeEditing(self);
        }

        // b) mouse input
        let hovered_function = if let Some(item) = self
            .edit_functions
            .iter()
            .find(|i| i.layout.point_in_rect(ctx.mouse_point()))
        {
            Some(item.idx)
        } else {
            None
        };

        if let Some(idx) = hovered_function {
            // update active index
            self.hovered_function = idx;
            self.state = ChooseFunction;
            if ctx.left_click {
                return self.do_action(game_state, idx);
            }
        }

        let hovered_gene = if let Some(item) = self
            .gene_items
            .iter()
            .find(|i| i.layout.point_in_rect(ctx.mouse_point()))
        {
            Some(item.gene_idx)
        } else {
            None
        };

        if let Some(idx) = hovered_gene {
            // update active index
            self.hovered_gene = idx;
            self.state = ChooseGene;
            if ctx.left_click {
                self.selected_gene = Some(idx);
            }
        }

        RunState::GenomeEditing(self)
    }

    fn do_action(mut self, game_state: &mut GameState, idx: usize) -> RunState {
        if let Some(item) = self.edit_functions.get(idx) {
            // self.state = item.state;
            // TODO: Trigger function
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
                    }
                    RunState::GenomeEditing(self)
                }
                Cut => {
                    // get currently selected gene and remove it
                    if let Some(selected) = self.selected_gene {
                        self.player_dna.simplified.remove(selected);
                        self.selected_function = None;
                        self.selected_gene = None;
                        self.hovered_gene = usize::min(0, self.hovered_gene - 1);
                        self.plasmid_charges -= 1;
                    }
                    RunState::GenomeEditing(self)
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
                                }
                            }
                        }
                    }
                    RunState::GenomeEditing(self)
                }
                Duplicate => {
                    if let Some(selected) = self.selected_gene {
                        if let Some(item) = self.gene_items.get(selected) {
                            let new_gene = item.clone();
                            self.gene_items.insert(selected + 1, new_gene);
                            self.plasmid_charges -= 1;
                        }
                    }
                    RunState::GenomeEditing(self)
                }
                Done => {
                    // apply changed genome to player
                    RunState::CheckInput
                }
                Cancel => {
                    // discard and return to input listening
                    RunState::CheckInput
                }

                // GenomeEditingState::ChooseGene => {}
                // GenomeEditingState::ChooseFunction => {}
                _ => RunState::GenomeEditing(self),
            }
        } else {
            RunState::GenomeEditing(self)
        }
    }
}
