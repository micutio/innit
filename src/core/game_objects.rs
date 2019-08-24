use std::ops::{Index, IndexMut};

use crate::core::world::world_gen::Tile;
use crate::entity::dna::GeneLibrary;
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
    obj_vec:         Vec<Option<Object>>,
}

impl GameObjects {
    pub fn new() -> Self {
        let num_world_tiles = (WORLD_WIDTH * WORLD_HEIGHT) as usize;
        let mut obj_vec = Vec::new();
        // obj_vec.push(None);
        // obj_vec.resize_with(num_world_tiles + 1, || None);

        GameObjects {
            num_world_tiles,
            obj_vec,
        }
    }

    pub fn get_tile_at(&mut self, x: usize, y: usize) -> &mut Option<Object> {
        // offset by one because player is the first object
        &mut self.obj_vec[(y * (WORLD_WIDTH as usize) + x) + 1]
    }

    /// Allocate enough space in the object vector to fit the player and all world tiles.
    pub fn init_world(&mut self, game_rng: &mut GameRng, gene_library: &mut GeneLibrary) {
        assert!(self.obj_vec.is_empty());
        self.obj_vec.push(None);
        self.obj_vec.resize_with(self.num_world_tiles + 1, || None);
        for y in 0..WORLD_HEIGHT {
            for x in 0..WORLD_WIDTH {
                // debug!("placing tile at ({}, {})", x, y);
                self.obj_vec[((y as usize) * (WORLD_WIDTH as usize) + (x as usize)) + 1]
                    .replace(Tile::wall(game_rng, gene_library, x, y));
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

    // /// Remove all objects that are not the player or world tiles.
    // /// NOTE: This means we cannot go back to a dungeon level once we leave it.
    // pub fn truncate(&mut self) {
    //     // PLayer is the first element, remove everything else.
    //     // NOTE: works only if player is the first object!
    //     self.obj_vec.truncate(self.num_world_tiles);
    //     for y in 0..WORLD_HEIGHT {
    //         for x in 0..WORLD_WIDTH {
    //             self.obj_vec[(y as usize) * (WORLD_WIDTH as usize) + (x as usize)]
    //                 .replace(Tile::wall(x, y));
    //         }
    //     }
    // }

    /// Check wether there is an objects blocking access to the given world coordinate
    pub fn is_blocked(&self, x: i32, y: i32) -> bool {
        self.obj_vec
            .iter()
            .flatten()
            .any(|object| object.physics.is_blocking && object.pos() == (x, y))
    }

    pub fn get_num_objects(&self) -> usize {
        self.obj_vec.len()
    }

    pub fn get_vector(&self) -> &Vec<Option<Object>> {
        &self.obj_vec
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
