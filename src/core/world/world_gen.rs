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
    pub explored: bool,
}

impl Tile {
    pub fn empty(game_rng: &mut GameRng, gene_library: &mut GeneLibrary, x: i32, y: i32) -> Object {
        let dna = gene_library.new_dna(game_rng, 10);
        let (sensors, processors, actuators) = gene_library.decode_dna(&dna);
        let mut tile_object = Object::new(
            // block_sight: false,
            // explored: false,
            x,
            y,
            Vec::new(),
            "empty tile",
            chars::UMLAUT,
            colors::BLACK,
            false,
            false,
            false,
            sensors,
            processors,
            actuators,
            Some(Ai::Basic),
        );
        tile_object.tile = Some(Tile { explored: false });
        tile_object
    }

    pub fn wall(game_rng: &mut GameRng, gene_library: &mut GeneLibrary, x: i32, y: i32) -> Object {
        let dna = gene_library.new_dna(game_rng, 10);
        let (sensors, processors, actuators) = gene_library.decode_dna(&dna);
        let mut tile_object = Object::new(
            // block_sight: false,
            // explored: false,
            x,
            y,
            Vec::new(),
            "empty tile",
            '\t',
            colors::BLACK,
            true,
            true,
            false,
            sensors,
            processors,
            actuators,
            Some(Ai::Basic),
        );
        tile_object.tile = Some(Tile { explored: false });
        tile_object
    }
}

pub fn is_explored(tile: &Tile) -> Option<&bool> {
    Some(&tile.explored)
}
