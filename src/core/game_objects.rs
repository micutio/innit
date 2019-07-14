/// Module Game Objects
/// 
/// The game object struct contains all game objects, including
/// * player character
/// * non-player character
/// * world tiles
/// * items
/// and offers methods to deal with them in an orderly fashion.

use entity::object::Object;
use game::{WORLD_WIDTH, WORLD_HEIGHT};
use core::world::Tile;

#[derive(Serialize, Deserialize, Default)]
pub struct GameObjects {
    player_index: usize,
    num_world_tiles: usize,
    obj_vec: Vec<Option<Object>>,
}

impl GameObjects {
    pub fn new() -> Self {
        let num_world_tiles = (WORLD_WIDTH * WORLD_HEIGHT) as usize;
        let player_index = 0;

        let game_objects = GameObjects{
            player_index,
            num_world_tiles,
            obj_vec: Vec::new(),
        };
        game_objects.init_world();
        game_objects
    }

    // pub fn get_vector(&self) -> &Vec<Option<Object>> {
    //     &self.obj_vec
    // }

    // pub fn get_vector_mut(&mut self) -> &mut Vec<Option<Object>> {
    //     &mut self.obj_vec
    // }

    pub fn get_tile_at(&self, x: usize, y: usize) -> &Option<Object> {
        // HACK: offset by one because player is the first object
        &self.obj_vec[(y * (WORLD_WIDTH as usize) + x) + 1]
    }

    /// Allocate enough space in the object vector to fit the player and all world tiles.
    fn init_world(&self) {
        self.obj_vec.push(None);
        self.obj_vec.resize_with(self.num_world_tiles + 1, || None);
        for y in 0..WORLD_HEIGHT {
            for x in 0..WORLD_WIDTH {
                &self.obj_vec[(y as usize) * (WORLD_WIDTH as usize) + (x as usize)].replace(Tile::wall(x, y));
            }
        }
    }

    pub fn set_player(&self, object: Object) {
        match self.obj_vec[self.player_index] {
            Some(player) => {
                panic!("[GameObjects] Error: trying to replace the player");
            }
            None => {
                self.obj_vec[self.player_index].replace(object);
            }
        }
    }

    pub fn push(&mut self, object: Object) {
        println!("pushing object {}", object.visual.name);
        self.obj_vec.push(Some(object));
    }

    pub fn extract(&mut self, index: usize) -> Option<Object> {
        match self.obj_vec.get_mut(index) {
            Some(item) => match item.take() {
                Some(object) => {
                    // println!("extract object {} @ index {}", object.name, index);
                    Some(object)
                }
                None => None,
            },
            None => panic!("[GameObjects::index] Error: invalid index {}", index),
        }
    }

    pub fn replace(&mut self, index: usize, object: Object) {
        let item = self.obj_vec.get_mut(index);
        match item {
            Some(obj) => {
                // println!("replace object {} @ index {}", object.name, index);
                obj.replace(object);
            }
            None => {
                panic!(
                    "[GameObjects::replace] Error: object {} with given index {} does not exist!",
                    object.visual.name, index
                );
            }
        }
    }

    /// Remove all objects that are not the player or world tiles.
    /// NOTE: This means we cannot go back to a dungeon level once we leave it.
    pub fn truncate(&self) {
        // PLayer is the first element, remove everything else.
        // NOTE: works only if player is the first object!
        assert_eq!(&self.obj_vec[self.player_index as usize].unwrap() as *const _, &self.obj_vec[0].unwrap() as *const _);
        self.obj_vec.truncate(self.num_world_tiles);
        for y in 0..WORLD_HEIGHT {
            for x in 0..WORLD_WIDTH {
                &self.obj_vec[(y as usize) * (WORLD_WIDTH as usize) + (x as usize)].replace(Tile::wall(x, y));
            }
        }
    }

    /// Check wether there is an objects blocking access to the given world coordinate
    pub fn is_blocked(&self, x: i32, y: i32) -> bool {
        self
            .obj_vec
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

use std::ops::{Index, IndexMut};

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
