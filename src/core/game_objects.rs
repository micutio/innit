use crate::core::game_state::GameState;
use crate::core::position::Position;
use crate::core::world::world_gen::Tile;
use crate::entity::genetics::{DnaType, GeneLibrary, GENE_LEN};
use crate::entity::object::Object;
use crate::entity::player::PLAYER;
use crate::game::{WORLD_HEIGHT, WORLD_WIDTH};
use crate::util::game_rng::GameRng;
use rltk::{Algorithm2D, BaseMap, Point};
use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

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

    pub fn get_tile_at(&mut self, x: usize, y: usize) -> &mut Option<Object> {
        // offset by one because player is the first object
        &mut self.obj_vec[(y * (WORLD_WIDTH as usize) + x) + 1]
    }

    /// Allocate enough space in the object vector to fit the player and all world tiles.
    pub fn blank_world(&mut self, state: &mut GameState) {
        assert!(self.obj_vec.is_empty());
        self.obj_vec.push(None);
        self.obj_vec.resize_with(self.num_world_tiles + 1, || None);
        for y in 0..WORLD_HEIGHT {
            for x in 0..WORLD_WIDTH {
                // debug!("placing tile at ({}, {})", x, y);
                self.obj_vec[((y as usize) * (WORLD_WIDTH as usize) + (x as usize)) + 1]
                    .replace(Tile::wall(x, y, state.env.debug_mode));
            }
        }
    }

    pub fn set_tile_dna_random(&mut self, rng: &mut GameRng, gene_library: &GeneLibrary) {
        for y in 0..WORLD_HEIGHT {
            for x in 0..WORLD_WIDTH {
                // debug!("setting tile dna at ({}, {})", x, y);
                if let Some(tile) =
                    &mut self.obj_vec[((y as usize) * (WORLD_WIDTH as usize) + (x as usize)) + 1]
                {
                    let (sensors, processors, actuators, dna) =
                        gene_library.new_genetics(rng, DnaType::Nucleus, false, GENE_LEN);
                    tile.change_genome(sensors, processors, actuators, dna);
                }
            }
        }
    }

    pub fn set_tile_dna(
        &mut self,
        rng: &mut GameRng,
        traits: Vec<String>,
        gene_library: &GeneLibrary,
    ) {
        for y in 0..WORLD_HEIGHT {
            for x in 0..WORLD_WIDTH {
                if let Some(tile) =
                    &mut self.obj_vec[((y as usize) * (WORLD_WIDTH as usize) + (x as usize)) + 1]
                {
                    let (sensors, processors, actuators, dna) = gene_library.decode_dna(
                        DnaType::Nucleus,
                        &gene_library.dna_from_traits(rng, &traits),
                    );
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

    pub fn extract_by_index(&mut self, index: usize) -> Option<Object> {
        match self.obj_vec.get_mut(index) {
            Some(item) => match item.take() {
                Some(object) => {
                    // debug!("extract object {} @ index {}", object.visual.name, index);
                    Some(object)
                }
                None => None,
            },
            None => panic!(" Error: invalid index {}", index),
        }
    }

    pub fn extract_by_pos(&mut self, pos: &Position) -> Option<(usize, Option<Object>)> {
        if let Some(i) = self
            .obj_vec
            .iter()
            .flatten()
            .position(|o| o.pos.is_equal(pos))
        {
            Some((i, self.extract_by_index(i)))
        } else {
            None
        }
    }

    pub fn extract_tile_by_pos(&mut self, pos: &Position) -> Option<(usize, Option<Object>)> {
        if let Some(i) = self.obj_vec.iter().position(|opt| {
            if let Some(obj) = opt {
                obj.pos.is_equal(pos) && obj.tile.is_some()
            } else {
                false
            }
        }) {
            Some((i, self.extract_by_index(i)))
        } else {
            None
        }
    }

    pub fn extract_entity_by_pos(&mut self, pos: &Position) -> Option<(usize, Option<Object>)> {
        if let Some(i) = self.obj_vec.iter().position(|opt| {
            if let Some(obj) = opt {
                obj.pos.is_equal(pos) && obj.tile.is_none()
            } else {
                false
            }
        }) {
            Some((i, self.extract_by_index(i)))
        } else {
            None
        }
    }

    pub fn extract_item_by_pos(&mut self, pos: &Position) -> Option<(usize, Option<Object>)> {
        if let Some(i) = self.obj_vec.iter().position(|opt| {
            if let Some(obj) = opt {
                obj.pos.is_equal(pos)
                    && obj.tile.is_none()
                    && obj.item.is_some()
                    && !obj.physics.is_blocking
            } else {
                false
            }
            // o.pos.is_equal(pos) && o.tile.is_none() && o.item.is_some() && !o.physics.is_blocking
        }) {
            Some((i, self.extract_by_index(i)))
        } else {
            None
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

    /// Check whether there is an object, tile or not, blocking access to the given world coordinate
    pub fn is_pos_blocked(&self, p: &Position) -> bool {
        self.obj_vec
            .iter()
            .flatten()
            .any(|object| object.physics.is_blocking && object.pos.is_equal(p))
    }

    /// Check whether there is any non-tile object located at the given position.
    /// The position may or may not be blocked.
    pub fn is_pos_occupied(&self, p: &Position) -> bool {
        self.get_non_tiles()
            .iter()
            .flatten()
            .any(|object| object.pos.is_equal(p))
    }

    pub fn get_obj_count(&self) -> usize {
        self.obj_vec.len()
    }

    pub fn get_vector(&self) -> &Vec<Option<Object>> {
        &self.obj_vec
    }

    pub fn get_vector_mut(&mut self) -> &mut Vec<Option<Object>> {
        &mut self.obj_vec
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

impl BaseMap for GameObjects {
    fn is_opaque(&self, idx: usize) -> bool {
        if idx > 0 && idx < self.obj_vec.len() {
            if let Some(o) = &self.obj_vec[idx] {
                o.physics.is_blocking
            } else {
                false
            }
        } else {
            false
        }
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = WORLD_WIDTH as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl Algorithm2D for GameObjects {
    /// Convert a Point (x/y) to an array index.
    fn point2d_to_index(&self, pt: Point) -> usize {
        (pt.y as usize * (WORLD_WIDTH as usize) + pt.x as usize) + (1 as usize)
    }

    /// Convert an array index to a point. Defaults to an index based on an array
    fn index_to_point2d(&self, idx: usize) -> Point {
        Point::new(
            (idx - 1) as i32 % WORLD_WIDTH,
            (idx - 1) as i32 / WORLD_WIDTH,
        )
    }

    fn dimensions(&self) -> Point {
        Point::new(WORLD_WIDTH, WORLD_HEIGHT)
    }
}
