/// Module World
///
/// The world generation module contains the trait that all world generators have to implement
/// to be exchangably used to create the game environments.
use tcod::{chars, colors};

use crate::{
    core::game_objects::GameObjects,
    entity::{ai::Ai, object::Object},
};

/// The world generation trait only requests to implement a method that
/// manipulated the world tiles provided in the GameObject struct.
pub trait WorldGen {
    fn make_world(&mut self, game_objects: &mut GameObjects, level: u32);
}

/// The tile is an object component that identifies an object as (mostly) fixed part of the game world.
#[derive(Debug, Serialize, Deserialize)]
pub struct Tile {
    pub explored: bool,
}

impl Tile {
    pub fn empty(x: i32, y: i32) -> Object {
        let mut tile_object = Object::new(
            // block_sight: false,
            // explored: false,
            x,
            y,
            "empty tile",
            chars::UMLAUT,
            colors::BLACK,
            false,
            false,
            false,
        );
        tile_object.tile = Some(Tile { explored: false });
        tile_object.ai = Some(Ai::Basic);
        tile_object
    }

    pub fn wall(x: i32, y: i32) -> Object {
        let mut tile_object = Object::new(
            // block_sight: false,
            // explored: false,
            x,
            y,
            "empty tile",
            '\t',
            colors::BLACK,
            true,
            true,
            false,
        );
        tile_object.tile = Some(Tile { explored: false });
        tile_object.ai = Some(Ai::Basic);
        tile_object
    }
}

pub fn is_explored(tile: &Tile) -> Option<&bool> {
    Some(&tile.explored)
}
