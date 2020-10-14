use std::ops::{Index, IndexMut};

use crate::core::world::world_gen::Tile;
use crate::entity::genetics::{GeneLibrary, GENE_LEN};
use crate::entity::object::Object;
use crate::game::{WORLD_HEIGHT, WORLD_WIDTH};
use crate::player::PLAYER;
use crate::util::game_rng::GameRng;

/// The game object struct contains all game objects, including
/// * player character
/// * non-player character
/// * world tiles
/// * items
/// and offers methods to deal with them in an orderly fashion.
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct GameObjects {
    num_world_tiles: usize,
    obj_vec: Vec<Option<Object>>,
}

impl GameObjects {
    pub fn new() -> Self {
        let num_world_tiles = (WORLD_WIDTH * WORLD_HEIGHT) as usize;
        let obj_vec = Vec::new();
        // obj_vec.push(None);
        // obj_vec.resize_with(num_world_tiles + 1, || None);

        GameObjects {
            num_world_tiles,
            obj_vec,
        }
    }

    // TODO: Add function that returns iterator over all tiles.
    pub fn get_tile_at(&mut self, x: usize, y: usize) -> &mut Option<Object> {
        // offset by one because player is the first object
        &mut self.obj_vec[(y * (WORLD_WIDTH as usize) + x) + 1]
    }

    /// Allocate enough space in the object vector to fit the player and all world tiles.
    pub fn blank_world(&mut self) {
        assert!(self.obj_vec.is_empty());
        self.obj_vec.push(None);
        self.obj_vec.resize_with(self.num_world_tiles + 1, || None);
        for y in 0..WORLD_HEIGHT {
            for x in 0..WORLD_WIDTH {
                // debug!("placing tile at ({}, {})", x, y);
                self.obj_vec[((y as usize) * (WORLD_WIDTH as usize) + (x as usize)) + 1]
                    .replace(Tile::wall(x, y));
            }
        }
    }

    pub fn set_tiles_dna(&mut self, game_rng: &mut GameRng, gene_library: &GeneLibrary) {
        for y in 0..WORLD_HEIGHT {
            for x in 0..WORLD_WIDTH {
                // debug!("setting tile dna at ({}, {})", x, y);
                if let Some(tile) =
                    &mut self.obj_vec[((y as usize) * (WORLD_WIDTH as usize) + (x as usize)) + 1]
                {
                    let (sensors, processors, actuators, dna) =
                        gene_library.new_genetics(game_rng, GENE_LEN);
                    tile.change_genome(sensors, processors, actuators, dna);
                }
            }
        }
    }

    pub fn set_player(&mut self, object: Object) {
        match &mut self.obj_vec[PLAYER] {
            Some(player) => {
                panic!(
                    "Error: trying to replace the player {:?} with {:?}",
                    player, object
                );
            }
            None => {
                debug!("setting player object {:?}", object);
                self.obj_vec[PLAYER].replace(object);
            }
        }
    }

    pub fn push(&mut self, object: Object) {
        trace!("adding {} to game objects", object.visual.name);
        self.obj_vec.push(Some(object));
    }

    pub fn extract(&mut self, index: usize) -> Option<Object> {
        match self.obj_vec.get_mut(index) {
            Some(item) => match item.take() {
                Some(object) => {
                    // debug!(
                    //     "extract object {} @ index {}",
                    //     object.visual.name, index
                    // );
                    Some(object)
                }
                None => None,
            },
            None => panic!(" Error: invalid index {}", index),
        }
    }

    pub fn replace(&mut self, index: usize, object: Object) {
        let item = self.obj_vec.get_mut(index);
        match item {
            Some(obj) => {
                // debug!("replace object {} @ index {}", object.visual.name, index);
                obj.replace(object);
            }
            None => {
                panic!(
                    "Error: object {} with given index {} does not exist!",
                    object.visual.name, index
                );
            }
        }
    }

    /// Check whether there is an objects blocking access to the given world coordinate
    pub fn is_blocked(&self, x: i32, y: i32) -> bool {
        self.obj_vec
            .iter()
            .flatten()
            .any(|object| object.physics.is_blocking && object.pos() == (x, y))
    }

    pub fn is_blocked_by_object(&self, x: i32, y: i32) -> bool {
        self.get_non_tiles()
            .iter()
            .find(|obj| match obj {
                Some(o) => o.x == x && o.y == y,
                None => false,
            })
            .is_some()
    }

    pub fn get_num_objects(&self) -> usize {
        self.obj_vec.len()
    }

    pub fn get_vector(&self) -> &Vec<Option<Object>> {
        &self.obj_vec
    }

    /// Return a Vec slice with all tiles in the world.
    pub fn get_tiles(&self) -> &[Option<Object>] {
        let start: usize = 1;
        let end: usize = WORLD_HEIGHT as usize * WORLD_WIDTH as usize;
        &self.obj_vec[start..end]
    }

    /// Return a Vec slice with all objects that are not tiles in the world.
    pub fn get_non_tiles(&self) -> &[Option<Object>] {
        let start: usize = WORLD_HEIGHT as usize * WORLD_WIDTH as usize;
        &self.obj_vec[start..]
    }
}

impl Index<usize> for GameObjects {
    type Output = Option<Object>;

    fn index(&self, i: usize) -> &Self::Output {
        let item = self.obj_vec.get(i);
        match item {
            Some(obj_option) => obj_option,
            None => panic!("[GameObjects::index] Error: invalid index {}", i),
        }
    }
}

impl IndexMut<usize> for GameObjects {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        let item = self.obj_vec.get_mut(i);
        match item {
            Some(obj_option) => obj_option,
            None => panic!("[GameObjects::index] Error: invalid index {}", i),
        }
    }
}
