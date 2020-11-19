//! The world generation module contains the trait that all world generators have to implement to
//! be changeably used to create the game environments.
// TODO: WorldGen should offer an API to define spawn and drop tables.

use tcod::colors;

use crate::core::game_objects::GameObjects;
use crate::core::game_state::GameState;
use crate::entity::ai::{AiPassive, AiRandom};
use crate::entity::control::Controller;
use crate::entity::genetics::{DnaType, GENE_LEN};
use crate::entity::object::Object;
use crate::ui::game_frontend::GameFrontend;
use rltk::Rltk;

/// The world generation trait only requests to implement a method that
/// manipulated the world tiles provided in the GameObject struct.
pub trait WorldGen {
    fn make_world(
        &mut self,
        state: &mut GameState,
        ctx: &mut Rltk,
        objects: &mut GameObjects,
        level: u32,
    );

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
            .visualize("empty tile", '\u{fa}', colors::WHITE)
            .physical(false, false, is_visible)
            .tile_explored(is_visible)
            .control(Controller::Npc(Box::new(AiPassive::new())))
    }

    pub fn wall(x: i32, y: i32, is_visible: bool) -> Object {
        Object::new()
            .position(x, y)
            .living(true)
            .visualize("wall tile", '\t', colors::WHITE)
            .physical(true, true, is_visible)
            .tile_explored(is_visible)
            .control(Controller::Npc(Box::new(AiPassive::new())))
    }
}

/// For use in lambdas.
pub fn is_explored(tile: &Tile) -> Option<&bool> {
    Some(&tile.is_explored)
}

pub enum Monster {
    Bacteria,
    Virus,
}

pub fn new_monster(state: &mut GameState, monster: Monster, x: i32, y: i32, _level: u32) -> Object {
    // append LTR markers
    match monster {
        Monster::Virus => Object::new()
            .position(x, y)
            .living(true)
            .visualize("virus", 'v', colors::DESATURATED_GREEN)
            .physical(true, false, false)
            .genome(
                0.75,
                state
                    .gene_library
                    .new_genetics(&mut state.rng, DnaType::Nucleoid, true, GENE_LEN),
            )
            .control(Controller::Npc(Box::new(AiRandom::new()))),
        Monster::Bacteria => Object::new()
            .position(x, y)
            .living(true)
            .visualize("bacteria", 'b', colors::DARKER_GREEN)
            .physical(true, false, false)
            .genome(
                0.9,
                state
                    .gene_library
                    .new_genetics(&mut state.rng, DnaType::Nucleoid, false, GENE_LEN),
            )
            .control(Controller::Npc(Box::new(AiRandom::new()))),
    }
}
