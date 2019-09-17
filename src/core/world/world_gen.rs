//! The world generation module contains the trait that all world generators have to implement to
//! be exchangably used to create the game environments.
// TODO: WorldGen should offer an API to define spawn and drop tables.

use tcod::{chars, colors};

use crate::core::game_objects::GameObjects;
use crate::entity::ai::Ai;
use crate::entity::genetics::GeneLibrary;
use crate::entity::object::Object;
use crate::game::DEBUG_MODE;
use crate::ui::game_frontend::GameFrontend;
use crate::util::game_rng::GameRng;

/// The world generation trait only requests to implement a method that
/// manipulated the world tiles provided in the GameObject struct.
pub trait WorldGen {
    fn make_world(
        &mut self,
        game_frontend: &mut GameFrontend,
        game_objects: &mut GameObjects,
        game_rng: &mut GameRng,
        gene_library: &mut GeneLibrary,
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
    pub fn empty(x: i32, y: i32) -> Object {
        Object::new()
            .position(x, y)
            .living(true)
            .visualize("empty tile", chars::UMLAUT, colors::WHITE)
            .physical(false, false, DEBUG_MODE)
            .tile_explored(DEBUG_MODE)
            .ai(Ai::Basic)
    }

    pub fn wall(x: i32, y: i32) -> Object {
        Object::new()
            .position(x, y)
            .living(true)
            .visualize("wall tile", '\t', colors::WHITE)
            .physical(true, true, DEBUG_MODE)
            .tile_explored(DEBUG_MODE)
            .ai(Ai::Basic)
    }
}

pub fn is_explored(tile: &Tile) -> Option<&bool> {
    Some(&tile.is_explored)
}
