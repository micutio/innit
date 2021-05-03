//! The world generation module contains the trait that all world generators have to implement to
//! be changeably used to create the game environments.
// TODO: WorldGen should offer an API to define spawn and drop tables.
use crate::entity::control::Controller;
use crate::entity::genetics::{DnaType, GENE_LEN};
use crate::entity::object::Object;
use crate::{core::game_objects::GameObjects, entity::object::InventoryItem};
use crate::{core::game_state::GameState, entity::action::hereditary::ActEditGenome};
use crate::{
    entity::ai::{AiRandom, AiVirus},
    ui::palette,
};
use serde::{Deserialize, Serialize};

/// The world generation trait only requests to implement a method that
/// manipulated the world tiles provided in the GameObject struct.
pub trait WorldGen {
    fn make_world(&mut self, state: &mut GameState, objects: &mut GameObjects, level: u32);

    fn get_player_start_pos(&self) -> (i32, i32);
}

/// The tile is an object component that identifies an object as (mostly) fixed part of the game
/// world.
#[derive(Debug, Serialize, Deserialize)]
pub struct Tile {
    pub is_explored: bool,
}

impl Tile {
    pub fn empty(x: i32, y: i32, is_visible: bool) -> Object {
        Object::new()
            .position(x, y)
            .living(true)
            .visualize("empty tile", '·', (255, 255, 255))
            .physical(false, false, is_visible)
            .tile_explored(is_visible)
        // .control(Controller::Npc(Box::new(AiPassive::new())))
    }

    pub fn wall(x: i32, y: i32, is_visible: bool) -> Object {
        Object::new()
            .position(x, y)
            .living(true)
            .visualize("wall tile", '◘', (255, 255, 255))
            .physical(true, true, is_visible)
            .tile_explored(is_visible)
        // .control(Controller::Npc(Box::new(AiPassive::new())))
    }
}

/// For use in lambdas.
pub fn is_explored(tile: &Tile) -> Option<&bool> {
    Some(&tile.is_explored)
}

#[derive(Clone, Copy)]
pub enum Monster {
    Bacteria,
    Virus,
    Plasmid,
}

pub fn new_monster(state: &mut GameState, monster: Monster, x: i32, y: i32, _level: u32) -> Object {
    // append LTR markers
    match monster {
        Monster::Virus => Object::new()
            .position(x, y)
            .living(true)
            .visualize("virus", 'v', palette().virus)
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
            .visualize("bacteria", 'b', palette().bacteria)
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
            .visualize("Plasmid", 'p', palette().plasmid)
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
