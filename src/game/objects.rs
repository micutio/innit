use crate::entity::{genetics, Object};
use crate::game;
use crate::game::world_gen;
use crate::game::Position;
use crate::rand::Rng;
use crate::util::rng;

use rltk;
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
pub struct ObjectStore {
    world_tile_count: usize,
    occupation: Vec<usize>,
    objects: Vec<Option<Object>>,
}

impl ObjectStore {
    pub fn new() -> Self {
        let world_tile_count = (game::consts::WORLD_WIDTH * game::consts::WORLD_HEIGHT) as usize;
        let mut occupation: Vec<usize> = Vec::with_capacity(world_tile_count);
        occupation.resize_with(world_tile_count, || 0);
        let mut objects: Vec<Option<Object>> = Vec::with_capacity(world_tile_count * 2);
        objects.resize_with(world_tile_count * 2, || None);

        ObjectStore {
            world_tile_count,
            occupation,
            objects,
        }
    }

    pub fn get_tile_at(&mut self, x: i32, y: i32) -> &mut Option<Object> {
        // offset by one because player is the first object
        &mut self.objects[(y as usize * (game::consts::WORLD_WIDTH as usize) + x as usize) + 1]
    }

    /// Allocate enough space in the object vector to fit the player and all world tiles.
    pub fn blank_world(&mut self) {
        self.objects.clear();
        self.objects.resize_with(self.world_tile_count + 1, || None);
        for y in 0..game::consts::WORLD_HEIGHT {
            for x in 0..game::consts::WORLD_WIDTH {
                // debug!("placing tile at ({}, {})", x, y);
                self.objects
                    [((y as usize) * (game::consts::WORLD_WIDTH as usize) + (x as usize)) + 1]
                    .replace(world_gen::Tile::wall(x, y, false));
            }
        }
    }

    pub fn _set_tile_dna_random(
        &mut self,
        rng: &mut rng::GameRng,
        gene_library: &genetics::GeneLibrary,
    ) {
        for y in 0..game::consts::WORLD_HEIGHT {
            for x in 0..game::consts::WORLD_WIDTH {
                // debug!("setting tile dna at ({}, {})", x, y);
                if let Some(tile) = &mut self.objects
                    [((y as usize) * (game::consts::WORLD_WIDTH as usize) + (x as usize)) + 1]
                {
                    let (sensors, processors, actuators, dna) = gene_library.new_genetics(
                        rng,
                        genetics::DnaType::Nucleus,
                        false,
                        genetics::GENOME_LEN,
                    );
                    tile.set_genome(sensors, processors, actuators, dna);
                }
            }
        }
    }

    pub fn set_tile_dna(
        &mut self,
        rng: &mut rng::GameRng,
        traits: Vec<String>,
        gene_library: &genetics::GeneLibrary,
    ) {
        for y in 0..game::consts::WORLD_HEIGHT {
            for x in 0..game::consts::WORLD_WIDTH {
                if let Some(tile) = &mut self.objects
                    [((y as usize) * (game::consts::WORLD_WIDTH as usize) + (x as usize)) + 1]
                {
                    let (sensors, processors, actuators, dna) = gene_library.dna_to_traits(
                        genetics::DnaType::Nucleus,
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
        match &mut self.objects[game::consts::PLAYER] {
            Some(player) => {
                panic!(
                    "Error: trying to replace the player {:?} with {:?}",
                    player, object
                );
            }
            None => {
                trace!("setting player object {:?}", object);
                self.objects[game::consts::PLAYER].replace(object);
            }
        }
    }

    pub fn push(&mut self, object: Object) {
        trace!("adding {} to game objects", object.visual.name);
        self.objects.push(Some(object));
    }

    pub fn extract_by_index(&mut self, index: usize) -> Option<Object> {
        match self.objects.get_mut(index) {
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

    pub fn extract_with_idx(&mut self, pos: &Position) -> Option<(usize, Option<Object>)> {
        let idx = position_to_index(pos.x(), pos.y());
        if idx < self.objects.len() {
            Some((idx, self.extract_by_index(idx)))
        } else {
            None
        }
    }

    pub fn extract_tile_by_pos(&mut self, pos: &Position) -> Option<(usize, Option<Object>)> {
        if let Some(i) = self.objects.iter().position(|opt| {
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
        if let Some(non_tile_idx) = self.get_non_tiles().iter().position(|opt| {
            if let Some(obj) = opt {
                obj.pos.is_equal(pos) && obj.tile.is_none()
            } else {
                false
            }
        }) {
            let full_idx = non_tile_idx + self.world_tile_count;
            Some((full_idx, self.extract_by_index(full_idx)))
        } else {
            None
        }
    }

    pub fn extract_item_by_pos(&mut self, pos: &Position) -> Option<(usize, Option<Object>)> {
        if let Some(i) = self.objects.iter().position(|opt| {
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

    pub fn replace(&mut self, index: usize, mut object: Object) {
        let item = self.objects.get_mut(index);
        match item {
            Some(obj) => {
                // debug!("replace object {} @ index {}", object.visual.name, index);
                let is_moved = object.pos.update();
                // record position changes of non-tiles for efficient occupation queries
                if is_moved && object.tile.is_none() {
                    let last_idx = coord_to_idx(
                        game::consts::WORLD_WIDTH,
                        object.pos.last_x(),
                        object.pos.last_y(),
                    );
                    self.occupation[last_idx] -= 1;
                    let this_idx =
                        coord_to_idx(game::consts::WORLD_WIDTH, object.pos.x(), object.pos.y());
                    self.occupation[this_idx] += 1;
                }
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
        let tile_idx = position_to_index(p.x(), p.y());
        let is_blocking_tile = if let Some(obj) = &self.objects[tile_idx] {
            obj.physics.is_blocking
        } else {
            false
        };

        let is_blocking_object = self
            .get_non_tiles()
            .iter()
            .flatten()
            .any(|object| object.physics.is_blocking && object.pos.is_equal(p));

        is_blocking_tile || is_blocking_object
    }

    /// Check whether there is any non-tile object located at the given position.
    /// The position may or may not be blocked.
    pub fn is_pos_occupied(&self, p: &Position) -> bool {
        let idx = coord_to_idx(game::consts::WORLD_WIDTH, p.x(), p.y());
        self.occupation[idx] > 0
    }

    pub fn get_obj_count(&self) -> usize {
        self.objects.len()
    }

    pub fn get_vector(&self) -> &Vec<Option<Object>> {
        &self.objects
    }

    pub fn get_vector_mut(&mut self) -> &mut Vec<Option<Object>> {
        &mut self.objects
    }

    /// Return a Vec slice with all tiles in the world.
    pub fn get_tiles(&self) -> &[Option<Object>] {
        let start: usize = 1;
        &self.objects[start..self.world_tile_count]
    }

    pub fn get_neighborhood_tiles(&self, pos: Position) -> Neighborhood {
        Neighborhood::new(pos, self.get_vector())
    }

    /// Return a Vec slice with all tiles in the world.
    pub fn get_tiles_mut(&mut self) -> &mut [Option<Object>] {
        let start: usize = 1;
        &mut self.objects[start..self.world_tile_count]
    }

    /// Return a Vec slice with all objects that are not tiles in the world.
    pub fn get_non_tiles(&self) -> &[Option<Object>] {
        &self.objects[self.world_tile_count..]
    }
}

impl Index<usize> for ObjectStore {
    type Output = Option<Object>;

    fn index(&self, i: usize) -> &Self::Output {
        let item = self.objects.get(i);
        match item {
            Some(obj_option) => obj_option,
            None => panic!("[GameObjects::index] Error: invalid index {}", i),
        }
    }
}

impl IndexMut<usize> for ObjectStore {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        let item = self.objects.get_mut(i);
        match item {
            Some(obj_option) => obj_option,
            None => panic!("[GameObjects::index] Error: invalid index {}", i),
        }
    }
}

impl rltk::BaseMap for ObjectStore {
    fn is_opaque(&self, idx: usize) -> bool {
        if idx > 0 && idx < self.objects.len() {
            if let Some(o) = &self.objects[idx] {
                o.physics.is_blocking
            } else {
                false
            }
        } else {
            false
        }
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = game::consts::WORLD_WIDTH as usize;
        let p1 = rltk::Point::new(idx1 % w, idx1 / w);
        let p2 = rltk::Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl rltk::Algorithm2D for ObjectStore {
    /// Convert a Point (x/y) to an array index.
    fn point2d_to_index(&self, pt: rltk::Point) -> usize {
        (pt.y as usize * (game::consts::WORLD_WIDTH as usize) + pt.x as usize) + (1 as usize)
    }

    /// Convert an array index to a point. Defaults to an index based on an array
    fn index_to_point2d(&self, idx: usize) -> rltk::Point {
        rltk::Point::new(
            (idx - 1) as i32 % game::consts::WORLD_WIDTH,
            (idx - 1) as i32 / game::consts::WORLD_WIDTH,
        )
    }

    fn dimensions(&self) -> rltk::Point {
        rltk::Point::new(game::consts::WORLD_WIDTH, game::consts::WORLD_HEIGHT)
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
                let x = self.bounds[self.count].0 + self.cell_pos.x();
                let y = self.bounds[self.count].1 + self.cell_pos.y();

                self.count += 1;

                if x >= 0
                    && x < game::consts::WORLD_WIDTH
                    && y >= 0
                    && y < game::consts::WORLD_HEIGHT
                {
                    return Some(&self.buffer[position_to_index(x, y)]);
                }
            }
            None
        }
    }
}

/// Convert a Point (x/y) to an index in the object storage.
fn position_to_index(x: i32, y: i32) -> usize {
    (y as usize * (game::consts::WORLD_WIDTH as usize) + x as usize) + (1 as usize)
}

/// Convert an object storage index to a point.
fn _index_to_position(idx: usize) -> Position {
    let x = (idx - 1) as i32 % game::consts::WORLD_WIDTH;
    let y = (idx - 1) as i32 / game::consts::WORLD_WIDTH;
    Position::from_xy(x, y)
}

// Convert coordinate to an index in a vector.
pub fn coord_to_idx(width: i32, x: i32, y: i32) -> usize {
    (y * width + x) as usize
}

/// Convert an vector index to a point.
pub fn _idx_to_coord(width: usize, idx: usize) -> (i32, i32) {
    let x = idx % width;
    let y = idx / width;
    (x as i32, y as i32)
}
