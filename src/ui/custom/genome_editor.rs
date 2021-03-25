//! The genome editor provides the facilities for the player to manipulate their own DNA, provided
//! they have a plasmid that allows this.

use crate::entity::genetics::Dna;
use crate::game::{RunState, HUD_CON};
use crate::ui::color_palette::ColorPalette;
use rltk::{to_cp437, ColorPair, DrawBatch, Point, Rect, Rltk, VirtualKeyCode};
use std::fmt;
use std::fmt::{Display, Formatter};

/// Determines which features of the editor are enabled.
#[derive(Debug)]
pub enum GenomeEditorFeatureSet {
    Reduce,  // moving and cutting
    InPlace, // editing without moving or extending, like flipping bits
    Extend,  // copy and adding new
}

#[derive(Debug)]
pub enum GenomeEditingState {
    Idle,
    Move,
    Cut,
    FlipBit,
    Copy,
    Add,
    Done,
}

impl Display for GenomeEditingState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            GenomeEditingState::Idle => {
                write!(f, "")
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
            GenomeEditingState::Copy => {
                write!(f, "Copy")
            }
            GenomeEditingState::Add => {
                write!(f, "Add new")
            }
            GenomeEditingState::Done => {
                write!(f, "Done")
            }
        }
    }
}

#[derive(Debug)]
struct GeneItem {
    layout: Rect,
    /// position of the represented gene within the genome
    gene_idx: usize,
    title: String,
}

#[derive(Debug)]
struct EditFunction {
    layout: Rect,
    state: GenomeEditingState,
    title: String,
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
    features: GenomeEditorFeatureSet,
    active_item: usize,
    pub current_state: GenomeEditingState,
    pub player_dna: Dna,
    clipboard: Option<GeneItem>,
    edit_functions: Vec<EditFunction>,
    gene_items: Vec<GeneItem>,
}

impl GenomeEditor {
    // required functions
    // - constructor
    // - render items
    // - methods for each edit function
    //   - move, cut, flip bit, copy, add new

    /// Creates a new instance of the editor. DNA is parsed into items that can be cycled through.
    pub fn new(dna: Dna, layout: Rect, features: GenomeEditorFeatureSet) -> Self {
        let top_row_start: Point = Point::new(11, 6);
        let genome_row_start: Point = Point::new(11, 14);
        let mut edit_functions: Vec<EditFunction> = vec![];
        let mut gene_items: Vec<GeneItem> = vec![];

        use GenomeEditingState::*;
        for item in vec![Idle, Move, Cut, FlipBit, Copy, Add, Done] {}

        GenomeEditor {
            layout,
            features,
            active_item: 0,
            current_state: GenomeEditingState::Idle,
            player_dna: dna,
            clipboard: None,
            edit_functions, // TODO: Create items and insert here!
            gene_items,     // TODO: Create items and insert here!
        }
    }

    pub fn display(self, ctx: &mut Rltk, palette: &ColorPalette) -> RunState {
        // 1. render everything
        self.render(ctx, palette);

        // 2. read user input
        if let Some(key) = ctx.key {
            return match key {
                VirtualKeyCode::Escape => RunState::CheckInput,
                _ => RunState::GenomeEditing(self),
            };
        }

        // 3. make adjustments to genome

        // 4. package editor back up and return it
        RunState::GenomeEditing(self)
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
        }

        draw_batch.submit(6000).unwrap();
    }
}
