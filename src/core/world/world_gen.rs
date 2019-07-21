/// Module World
///
/// The world contains all structures and methods for terrain/dungeon generation
// external imports

use tcod::chars;
use tcod::colors;


use core::game_objects::GameObjects;

use entity::ai::Ai;

use entity::object::Object;




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

pub trait WorldGen {
    fn make_world(&mut self, objects: &mut GameObjects, level: u32);
}


