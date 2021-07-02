//! The world generation module contains the trait that all world generators have to implement to
//! be changeably used to create the game environments.

pub mod world_gen_organic;
pub mod world_gen_rogue;

use crate::core::game_objects::GameObjects;
use crate::core::game_state::GameState;
use crate::entity::object::Object;
use crate::game::RunState;
use crate::raws::object_template::ObjectTemplate;
use crate::raws::spawn::Spawn;
use serde::{Deserialize, Serialize};

/// The world generation trait only requests to implement a method that
/// manipulated the world tiles provided in the GameObject struct.
pub trait WorldGen {
    fn make_world(
        &mut self,
        state: &mut GameState,
        objects: &mut GameObjects,
        spawns: &[Spawn],
        object_templates: &[ObjectTemplate],
        level: u32,
    ) -> RunState;

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
