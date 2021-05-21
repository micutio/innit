use crate::core::game_state::GameState;
use crate::core::world::Monster;
use crate::entity::action::hereditary::ActEditGenome;
use crate::entity::ai::{AiRandom, AiVirus};
use crate::entity::control::Controller;
use crate::entity::genetics::{DnaType, GENE_LEN};
use crate::entity::object::{InventoryItem, Object};
use crate::ui::palette;

/// Struct for spawning objects that requires an internal state.
pub struct Spawn {}

pub(crate) fn new_monster(
    state: &mut GameState,
    monster: Monster,
    x: i32,
    y: i32,
    _level: u32,
) -> Object {
    // append LTR markers
    match monster {
        Monster::Virus => Object::new()
            .position(x, y)
            .living(true)
            .visualize("Virus", 'v', palette().entity_virus)
            .physical(true, false, false)
            // TODO: Pull genome create out of here. It's not the same for every NPC.
            .genome(
                0.75,
                state
                    .gene_library
                    .new_genetics(&mut state.rng, DnaType::Rna, true, GENE_LEN),
            )
            .control(Controller::Npc(Box::new(AiVirus::new()))),
        Monster::Bacteria => Object::new()
            .position(x, y)
            .living(true)
            .visualize("Bacteria", 'b', palette().entity_bacteria)
            .physical(true, false, false)
            .genome(
                0.9,
                state
                    .gene_library
                    .new_genetics(&mut state.rng, DnaType::Nucleoid, false, GENE_LEN),
            )
            .control(Controller::Npc(Box::new(AiRandom::new()))),
        Monster::Plasmid => Object::new()
            .position(x, y)
            .living(true)
            .visualize("Plasmid", 'p', palette().entity_plasmid)
            .physical(false, false, false)
            .inventory_item(InventoryItem::new(
                "Plasmids can transfer DNA between cells and bacteria and help manipulate it.",
                Some(Box::new(ActEditGenome::new())),
            ))
            .genome(
                1.0,
                state.gene_library.new_genetics(
                    &mut state.rng,
                    DnaType::Plasmid,
                    false,
                    _level as usize,
                ),
            ),
    }
}
