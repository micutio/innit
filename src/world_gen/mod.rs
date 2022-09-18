//! The world generation module contains the trait that all world generators have to implement to
//! be changeably used to create the game environments.

pub mod ca;
pub mod rogue;

use crate::entity::{self, ai, control, Object};
use crate::game::objects::ObjectStore;
use crate::game::State;
use crate::raws;
use crate::{game, ui};
use serde::{Deserialize, Serialize};

/// The world generation trait only requests to implement a method that
/// manipulated the world tiles provided in the `GameObject` struct.
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
#[allow(clippy::use_self)]
pub enum TileType {
    Wall,
    Floor,
    Void,
}

impl TileType {
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Wall => "tissue cell",
            Self::Floor => "floor tile",
            Self::Void => "void tile",
        }
    }
}
/// The tile is an object component that identifies an object as (mostly) fixed part of the game
/// world.
#[derive(Debug, Serialize, Deserialize)]
pub struct Tile {
    pub typ: TileType,
    pub morphogen: f64, // growth protein that controls where walls can 'grow'
    pub complement: entity::complement::ComplementProteins,
}

impl Tile {
    pub fn new_wall(x: i32, y: i32, is_visible: bool) -> Object {
        let dbg = game::env().debug_mode;
        let (fg_col, bg_col) = match dbg {
            game::env::GameOption::Enabled => (
                ui::palette().world_fg_wall_fov_true,
                ui::palette().world_bg_wall_fov_true,
            ),
            game::env::GameOption::Disabled => (
                ui::palette().world_fg_wall_fov_false,
                ui::palette().world_bg_wall_fov_false,
            ),
        };
        Object::new()
            .position_xy(x, y)
            .living(true)
            .visualize_bg(TileType::Wall.as_str(), '○', fg_col, bg_col)
            .physical(true, true, is_visible)
            .tile(TileType::Wall)
            .control(control::Controller::Npc(Box::new(ai::AiWallTile)))
    }

    pub fn new_floor(x: i32, y: i32, is_visible: bool) -> Object {
        let dbg = game::env().debug_mode;
        let (fg_col, bg_col) = match dbg {
            game::env::GameOption::Enabled => (
                ui::palette().world_fg_floor_fov_true,
                ui::palette().world_bg_floor_fov_true,
            ),
            game::env::GameOption::Disabled => (
                ui::palette().world_fg_floor_fov_false,
                ui::palette().world_bg_floor_fov_false,
            ),
        };
        Object::new()
            .position_xy(x, y)
            .living(true)
            .visualize_bg(TileType::Floor.as_str(), ' ', fg_col, bg_col)
            .physical(false, false, is_visible)
            .tile(TileType::Floor)
            .control(control::Controller::Npc(Box::new(ai::AiFloorTile)))
    }

    pub fn new_void(x: i32, y: i32, is_visible: bool) -> Object {
        Object::new()
            .position_xy(x, y)
            .living(true)
            .visualize(TileType::Void.as_str(), '█', ui::Rgba::new(0, 0, 0, 255))
            .physical(true, true, is_visible)
            .tile(TileType::Void)
    }
}
