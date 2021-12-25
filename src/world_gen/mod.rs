//! The world generation module contains the trait that all world generators have to implement to
//! be changeably used to create the game environments.

pub mod ca;
pub mod rogue;

use crate::entity::{ai, control, Object};
use crate::game;
use crate::game::objects::ObjectStore;
use crate::game::State;
use crate::raws;
use serde::{Deserialize, Serialize};

/// The world generation trait only requests to implement a method that
/// manipulated the world tiles provided in the GameObject struct.
pub trait WorldGen {
    /// Populate the world with tiles, objects and the player.
    /// Returns a runstate, which would be either `Runstate::Ticking` or `Runstate::WorldGen` to
    /// allow for intermediate visualisation of the world generation process.
    fn make_world(
        &mut self,
        state: &mut State,
        objects: &mut ObjectStore,
        spawns: &[raws::spawn::Spawn],
        object_templates: &[raws::template::ObjectTemplate],
    ) -> game::RunState;

    /// Returns the position of where the player was placed.
    fn get_player_start_pos(&self) -> (i32, i32);
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    Void,
}

impl TileType {
    pub fn as_str(&self) -> &str {
        match self {
            TileType::Wall => "wall tile",
            TileType::Floor => "floor tile",
            TileType::Void => "void tile",
        }
    }
}
/// The tile is an object component that identifies an object as (mostly) fixed part of the game
/// world.
#[derive(Debug, Serialize, Deserialize)]
pub struct Tile {
    pub typ: TileType,
    pub is_explored: bool,
    pub morphogen: f64, // growth protein that controls where walls can 'grow'
}

impl Tile {
    pub fn new_floor(x: i32, y: i32, is_visible: bool) -> Object {
        Object::new()
            .position_xy(x, y)
            .living(true)
            .visualize(TileType::Floor.as_str(), '·', (255, 255, 255, 255))
            .physical(false, false, is_visible)
            .tile(TileType::Floor, is_visible)
    }

    pub fn new_wall(x: i32, y: i32, is_visible: bool) -> Object {
        Object::new()
            .position_xy(x, y)
            .living(true)
            .visualize(TileType::Wall.as_str(), '◘', (255, 255, 255, 255))
            .physical(true, true, is_visible)
            .tile(TileType::Wall, is_visible)
            .control(control::Controller::Npc(Box::new(ai::AiTile)))
    }
}
