//! The genome editor provides the facilities for the player to manipulate their own DNA, provided
//! they have a plasmid that allows this.

use crate::entity::genetics::Dna;
use rltk::Rect;

/// Determines which features of the editor are enabled.
enum FeatureSet {
    Reduce,  // moving and cutting
    InPlace, // editing without moving or extending, like flipping bits
    Extend,  // copy and adding new
}

enum EditingState {
    Idle,
    Move,
    Cut,
    FlipBit,
    Copy,
    Add,
}

struct GeneItem {
    layout: Rect,
    /// position of the represented gene within the genome
    gene_idx: usize,
    title: String,
}

struct EditFunction {
    layout: Rect,
    title: String,
}

struct GenomeEditor {
    layout: Rect,
    features: FeatureSet,
    active_item: usize,
    current_state: EditingState,
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
    fn new(dna: Dna, layout: Rect, features: FeatureSet) -> Self {
        GenomeEditor {
            layout,
            features,
            active_item: 0,
            current_state: EditingState::Idle,
            clipboard: None,
            edit_functions: Vec::new(), // TODO: Create items and insert here!
            gene_items: Vec::new(),     // TODO: Create items and insert here!
        }
    }
}
