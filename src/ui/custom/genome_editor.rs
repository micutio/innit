//! The genome editor provides the facilities for the player to manipulate their own DNA, provided
//! they have a plasmid that allows this.

use crate::core::game_state::GameState;
use crate::entity::genetics::{Dna, TraitFamily};
use crate::game::{RunState, HUD_CON};
use crate::ui::color_palette::ColorPalette;
use crate::util::modulus;
use rltk::{to_cp437, ColorPair, DrawBatch, Point, Rect, Rltk, VirtualKeyCode};
use std::fmt;
use std::fmt::{Display, Formatter};

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
}

impl Display for GenomeEditingState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            GenomeEditingState::ChooseGene => {
                write!(f, "ChooseGene")
            }
            GenomeEditingState::ChooseFunction => {
                write!(f, "ChooseFunction")
            }
            GenomeEditingState::Move => {
                write!(f, "Move")
            }
            GenomeEditingState::Cut => {
                write!(f, "Cut")
            }
            GenomeEditingState::FlipBit => {
                write!(f, "Flip a Bit")
            }
            GenomeEditingState::Duplicate => {
                write!(f, "Duplicate")
            }
            GenomeEditingState::Done => {
                write!(f, "Done")
            }
        }
    }
}

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
    color: ColorPair,
}

impl GeneItem {
    fn new(layout: Rect, gene_idx: usize, color: ColorPair) -> Self {
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
        let top_row_y: i32 = 6;
        let mut mid_row_x: i32 = 11;
        let mid_row_y: i32 = 14;

        use GenomeEditingState::*;
        let edit_functions: Vec<EditFunction> = [Move, Cut, FlipBit, Duplicate, Done]
            .iter()
            .enumerate()
            .zip(["Move", "Cut", "Mutate", "Duplicate", "Confirm", "Cancel"].iter())
            .map(|((idx, e), s)| {
                let len: i32 = (s.len() + 3) as i32;
                let item = EditFunction::new(
                    Rect::with_size(top_row_x, top_row_y, len, 2),
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
                    ColorPair::new(col, cp.bg_dna),
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
        draw_batch.fill_region(
            self.layout,
            ColorPair::new(palette.fg_hud, palette.bg_hud),
            to_cp437(' '),
        );
        draw_batch.draw_hollow_box(self.layout, ColorPair::new(palette.fg_hud, palette.bg_hud));
        // for (index, item) in self.items.iter().enumerate() {
        //     let color = if index == self.selection {
        //         ColorPair::new(palette.fg_hud, palette.bg_hud_selected)
        //     } else {
        //         ColorPair::new(palette.fg_hud, palette.bg_hud)
        //     };
        //     draw_batch.print_color(item.top_left_corner(), &item.text, color);
        // }

        // TODO: Render top row, middle row and bottom info field
        for item in &self.edit_functions {
            draw_batch.fill_region(
                item.layout,
                ColorPair::new(palette.fg_hud, palette.bg_hud_content),
                to_cp437(' '),
            );
            draw_batch.print_color(
                Point::new(item.layout.x1 + 1, item.layout.y1 + 1),
                item.idx.to_string(),
                ColorPair::new(palette.fg_hud_highlight, palette.bg_hud_content),
            );
            draw_batch.print_color(
                Point::new(item.layout.x1 + 3, item.layout.y1 + 1),
                item.title.to_string(),
                ColorPair::new(palette.fg_hud, palette.bg_hud_content),
            );
        }

        for item in &self.gene_items {
            let c: char = if modulus(item.gene_idx, 2) == 0 {
                '►'
            } else {
                '◄'
            };
            draw_batch.print_color(item.layout.center(), c, item.color);
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
                VirtualKeyCode::Key1 => self.do_action(game_state, 0),
                VirtualKeyCode::Key2 => self.do_action(game_state, 1),
                VirtualKeyCode::Key3 => self.do_action(game_state, 2),
                VirtualKeyCode::Key4 => self.do_action(game_state, 3),
                VirtualKeyCode::Key5 => self.do_action(game_state, 4),
                VirtualKeyCode::Key6 => self.do_action(game_state, 5),
                VirtualKeyCode::Key7 => self.do_action(game_state, 6),
                VirtualKeyCode::Key8 => self.do_action(game_state, 7),
                VirtualKeyCode::Key9 => self.do_action(game_state, 8),
                VirtualKeyCode::Key0 => self.do_action(game_state, 9),
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
                        self.hovered_function = usize::max(self.hovered_function - 1, 0);
                    }
                    GenomeEditingState::ChooseGene => {
                        self.hovered_gene = usize::max(self.hovered_gene - 1, 0);
                    }
                    _ => {}
                },
                VirtualKeyCode::Right => match self.state {
                    GenomeEditingState::Move => {
                        unimplemented!()
                        // if let Some(idx) = self.selected_gene {
                        // if let Some(item1) = self.player_dna.simplified.get(idx) {
                        //     item1.
                        // }
                        // }
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
                        if let Some(_) = self.selected_gene {
                            self.selected_gene = None;
                        } else {
                            self.selected_gene = Some(self.hovered_gene);
                        }
                    }
                    ChooseFunction => {
                        if let Some(idx) = self.selected_function {
                            self.do_action(game_state, idx);
                        }
                    }
                    _ => {}
                },
                VirtualKeyCode::Escape => return RunState::CheckInput,
                _ => return RunState::GenomeEditing(self),
            }
        }

        // b) mouse input
        // if we have a mouse activity, check first for clicks, then for hovers
        if let Some(item) = self
            .edit_functions
            .iter()
            .find(|i| i.layout.point_in_rect(ctx.mouse_point()))
        {
            // update active index
            self.hovered_function = item.idx;
            self.state = ChooseFunction;
            if ctx.left_click {
                self.selected_function = Some(item.idx);
                // TODO: implement activation of functions
            }
        }

        if let Some(item) = self
            .gene_items
            .iter()
            .find(|i| i.layout.point_in_rect(ctx.mouse_point()))
        {
            // update active index
            self.hovered_gene = item.gene_idx;
            self.state = ChooseGene;
            if ctx.left_click {
                self.selected_gene = Some(item.gene_idx);
                // TODO: implement activation of functions
            }
        }

        RunState::GenomeEditing(self)
    }

    fn do_action(&mut self, game_state: &mut GameState, idx: usize) {
        if let Some(item) = self.edit_functions.get(idx) {
            // self.state = item.state;
            // TODO: Trigger function
            use GenomeEditingState::*;
            let is_success: bool = match item.state {
                Move => {
                    if let Move = self.state {
                        // finalise move action
                        self.state = ChooseFunction;
                        self.selected_function = None;
                        true
                    } else {
                        // start move action
                        self.state = Move;
                        self.selected_function = Some(item.idx);
                        false
                    }
                }
                Cut => {
                    // get currently selected gene and remove it
                    if let Some(selected) = self.selected_gene {
                        self.player_dna.simplified.remove(selected);
                        self.selected_function = None;
                        self.selected_gene = None;
                        self.hovered_gene = usize::min(0, self.hovered_gene - 1);
                        true
                    } else {
                        false
                    }
                }
                FlipBit => {
                    let result: bool = if let Some(selected) = self.selected_gene {
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
                                    std::mem::replace(
                                        &mut &self.player_dna.simplified[selected],
                                        new_repr,
                                    );
                                    self.selected_function = None;
                                    true
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    result
                }
                Duplicate => {
                    if let Some(selected) = self.selected_gene {
                        if let Some(item) = self.gene_items.get(selected) {
                            let new_gene = item.clone();
                            self.gene_items.insert(selected + 1, new_gene);
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                // Done => {}
                // GenomeEditingState::ChooseGene => {}
                // GenomeEditingState::ChooseFunction => {}
                _ => false,
            };

            if is_success {
                self.plasmid_charges -= 1;
            }
        }
    }
}
