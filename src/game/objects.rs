use crate::entity::genetics::{DnaType, GeneLibrary, GENOME_LEN};
use crate::entity::object::Object;
use crate::game::game_env::PLAYER;
use crate::game::position::Position;
use crate::game::world_gen::Tile;
use crate::game::{WORLD_HEIGHT, WORLD_WIDTH};
use crate::rand::Rng;
use crate::util::rng::GameRng;
use rltk::{Algorithm2D, BaseMap, Point};
use std::ops::{Index, IndexMut};

#[cfg(not(target_arch = "wasm32"))]
use serde::{Deserialize, Serialize};

/// The game object struct contains all game objects, including
/// * player character
/// * non-player character
/// * world tiles
/// * items
/// and offers methods to deal with them in an orderly fashion.
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[derive(Default, Debug)]
pub struct GameObjects {
    world_tile_count: usize,
    obj_vec: Vec<Option<Object>>,
}

impl GameObjects {
    pub fn new() -> Self {
        let world_tile_count = (WORLD_WIDTH * WORLD_HEIGHT) as usize;
        let obj_vec = Vec::new();
        // obj_vec.push(None);
        // obj_vec.resize_with(num_world_tiles + 1, || None);

        GameObjects {
            world_tile_count,
            obj_vec,
        }
    }

    pub fn get_tile_at(&mut self, x: i32, y: i32) -> &mut Option<Object> {
        // offset by one because player is the first object
        &mut self.obj_vec[(y as usize * (WORLD_WIDTH as usize) + x as usize) + 1]
    }

    /// Allocate enough space in the object vector to fit the player and all world tiles.
    pub fn blank_world(&mut self) {
        assert!(self.obj_vec.is_empty());
        self.obj_vec.push(None);
        self.obj_vec.resize_with(self.world_tile_count + 1, || None);
        for y in 0..WORLD_HEIGHT {
            for x in 0..WORLD_WIDTH {
                // debug!("placing tile at ({}, {})", x, y);
                self.obj_vec[((y as usize) * (WORLD_WIDTH as usize) + (x as usize)) + 1]
                    .replace(Tile::wall(x, y, false));
            }
        }
    }

    pub fn _set_tile_dna_random(&mut self, rng: &mut GameRng, gene_library: &GeneLibrary) {
        for y in 0..WORLD_HEIGHT {
            for x in 0..WORLD_WIDTH {
                // debug!("setting tile dna at ({}, {})", x, y);
                if let Some(tile) =
                    &mut self.obj_vec[((y as usize) * (WORLD_WIDTH as usize) + (x as usize)) + 1]
                {
                    let (sensors, processors, actuators, dna) =
                        gene_library.new_genetics(rng, DnaType::Nucleus, false, GENOME_LEN);
                    tile.set_genome(sensors, processors, actuators, dna);
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
                    let (sensors, processors, actuators, dna) = gene_library.dna_to_traits(
                        DnaType::Nucleus,
                        &gene_library.dna_from_trait_strs(rng, &traits),
                    );
                    tile.set_genome(sensors, processors, actuators, dna);
                    tile.processors.life_elapsed =
                        rng.gen_range(0..tile.processors.life_expectancy);
                    // println!(
                    //     "TILE AGE: {}/{}",
                    //     tile.processors.life_elapsed, tile.processors.life_expectancy
                    // );
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
                trace!("setting player object {:?}", object);
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

    pub fn extract_non_tile_by_pos(&mut self, pos: &Position) -> Option<(usize, Option<Object>)> {
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

    pub fn get_neighborhood_tiles(&self, pos: Position) -> Neighborhood {
        Neighborhood::new(pos, self.get_vector())
    }

    /// Return a Vec slice with all tiles in the world.
    pub fn get_tiles_mut(&mut self) -> &mut [Option<Object>] {
        let start: usize = 1;
        let end: usize = WORLD_HEIGHT as usize * WORLD_WIDTH as usize;
        &mut self.obj_vec[start..end]
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

static VON_NEUMAN_NEIGHBORHOOD: &'static [(i32, i32); 4] = &[(-1, 0), (0, -1), (1, 0), (0, 1)];

pub struct Neighborhood<'a> {
    count: usize,
    cell_pos: Position,
    bounds: &'a [(i32, i32)],
    buffer: &'a [Option<Object>],
}

impl<'a> Neighborhood<'a> {
    fn new(cell_pos: Position, buffer: &'a [Option<Object>]) -> Self {
        Neighborhood {
            count: 0,
            cell_pos,
            bounds: VON_NEUMAN_NEIGHBORHOOD,
            buffer,
        }
    }
}

// Implement `Iterator` for `Neighborhood`.
// The `Iterator` trait only requires a method to be defined for the `next` element.
impl<'a> Iterator for Neighborhood<'a> {
    // We can refer to this type using Self::Item
    type Item = &'a Option<Object>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == self.bounds.len() {
            None
        } else {
            while self.count < self.bounds.len() {
                let x = self.bounds[self.count].0 + self.cell_pos.x;
                let y = self.bounds[self.count].1 + self.cell_pos.y;

                self.count += 1;

                if x >= 0 && x < WORLD_WIDTH && y >= 0 && y < WORLD_HEIGHT {
                    return Some(&self.buffer[position_to_index(x, y)]);
                }
            }
            None
        }
    }
}

/// Convert a Point (x/y) to an array index.
fn position_to_index(x: i32, y: i32) -> usize {
    (y as usize * (WORLD_WIDTH as usize) + x as usize) + (1 as usize)
}

/// Convert an array index to a point. Defaults to an index based on an array
fn _index_to_position(idx: usize) -> Position {
    Position::new(
        (idx - 1) as i32 % WORLD_WIDTH,
        (idx - 1) as i32 / WORLD_WIDTH,
    )
}
