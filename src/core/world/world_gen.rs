//! The world generation module contains the trait that all world generators have to implement to
//! be exchangably used to create the game environments.

use tcod::{chars, colors};

use crate::core::game_objects::GameObjects;
use crate::entity::ai::Ai;
use crate::entity::dna::GeneLibrary;
use crate::entity::object::Object;
use crate::util::game_rng::GameRng;

/// The world generation trait only requests to implement a method that
/// manipulated the world tiles provided in the GameObject struct.
pub trait WorldGen {
    fn make_world(
        &mut self,
        game_objects: &mut GameObjects,
        game_rng: &mut GameRng,
        gene_library: &mut GeneLibrary,
        level: u32,
    );
}

/// The tile is an object component that identifies an object as (mostly) fixed part of the game
/// world.
#[derive(Debug, Serialize, Deserialize)]
pub struct Tile {
    pub is_explored: bool,
}

impl Tile {
    pub fn empty(x: i32, y: i32) -> Object {
        Object::new()
            .position(x, y)
            .living(true)
            .visualize("empty tile", chars::UMLAUT, colors::BLACK)
            .physical(false, false, false)
            .tile_explored(false)
            .ai(Ai::Basic)
    }

    pub fn wall(x: i32, y: i32) -> Object {
        Object::new()
            .position(x, y)
            .living(true)
            .visualize("wall tile", '\t', colors::BLACK)
            .physical(true, true, false)
            .tile_explored(false)
            .ai(Ai::Basic)
    }
}

pub fn is_explored(tile: &Tile) -> Option<&bool> {
    Some(&tile.is_explored)
}
