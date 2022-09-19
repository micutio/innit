use crate::entity::{genetics, Object};
use crate::game::world_gen;
use crate::game::Position;
use crate::rand::Rng;
use crate::util::random;
use crate::{game, util};

use bracket_lib::prelude as rltk;
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

        Self {
            world_tile_count,
            occupation,
            objects,
        }
    }

    /// Allocate enough space in the object vector to fit all world tiles and some room to spare.
    pub fn blank_circle_world(&mut self) {
        self.objects.clear();
        self.objects.resize_with(self.world_tile_count * 2, || None);
        // First carve out a circular world of wall tiles.
        self.bresen_circle();
        // Then fill the rest of world with void tiles.
        for y in 0..game::consts::WORLD_HEIGHT {
            for x in 0..game::consts::WORLD_WIDTH {
                let idx = coord_to_idx(game::consts::WORLD_WIDTH, x, y);
                if self.objects[idx].is_none() {
                    self.objects[idx].replace(world_gen::Tile::new_void(x, y, false));
                }
            }
        }
    }

    fn bresen_circle(&mut self) {
        let _timer = util::Timer::new("bresenham circle");
        let center_x = (game::consts::WORLD_WIDTH / 2) - 1;
        let center_y = (game::consts::WORLD_HEIGHT / 2) - 1;
        let center_point = rltk::Point::new(center_x, center_y);

        let max_radius = center_x - 1;
        for r in 0..=max_radius {
            for point in rltk::BresenhamCircleNoDiag::new(center_point, r) {
                let idx = coord_to_idx(game::consts::WORLD_WIDTH, point.x, point.y);
                self.objects[idx].replace(world_gen::Tile::new_wall(point.x, point.y, false));
            }
        }

        // finally turn center point as well
        let idx = coord_to_idx(game::consts::WORLD_WIDTH, center_x, center_y);
        self.objects[idx].replace(world_gen::Tile::new_wall(center_x, center_y, false));
    }

    /// Iterate over all tiles and update their protein levels
    pub fn update_complement_proteins(&mut self) {
        // `update` step
        // - for each tile:
        //   - 1. extract from objects
        //   - 2. update complement proteins from neighbor tiles
        //   - 3. put back into objects

        (0..self.world_tile_count).for_each(|i| {
            let opt_obj = self.extract_by_index(i);
            if let Some(mut tile_obj) = opt_obj {
                if let Some(t) = &mut tile_obj.tile {
                    if matches!(t.typ, world_gen::TileType::Floor) {
                        t.complement.detect_neighbor_concentration(
                            self.get_neighborhood_tiles(tile_obj.pos),
                        );
                    }
                }
                self.replace(i, tile_obj);
            }
        });

        // `apply new values` step
        // - for each tile:
        //   - 1. apply new value
        self.get_tiles_mut().iter_mut().flatten().for_each(|obj| {
            if let Some(t) = &mut obj.tile {
                if matches!(t.typ, world_gen::TileType::Floor) {
                    t.complement.update();
                }
            }
        });
    }

    pub fn get_tile_at(&self, x: i32, y: i32) -> &Option<Object> {
        let idx = coord_to_idx(game::consts::WORLD_WIDTH, x, y);
        &self.objects[idx]
    }

    pub fn get_tile_at_mut(&mut self, x: i32, y: i32) -> &mut Option<Object> {
        let idx = coord_to_idx(game::consts::WORLD_WIDTH, x, y);
        &mut self.objects[idx]
    }

    pub fn _set_tile_dna_random(
        &mut self,
        rng: &mut random::GameRng,
        gene_library: &genetics::GeneLibrary,
    ) {
        for y in 0..game::consts::WORLD_HEIGHT {
            for x in 0..game::consts::WORLD_WIDTH {
                let idx = coord_to_idx(game::consts::WORLD_WIDTH, x, y);
                if let Some(tile) = &mut self.objects[idx] {
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
        rng: &mut random::GameRng,
        traits: &[String],
        gene_library: &genetics::GeneLibrary,
    ) {
        for y in 0..game::consts::WORLD_HEIGHT {
            for x in 0..game::consts::WORLD_WIDTH {
                let idx = coord_to_idx(game::consts::WORLD_WIDTH, x, y);
                if let Some(tile) = &mut self.objects[idx] {
                    let (sensors, processors, actuators, dna) = gene_library.dna_to_traits(
                        genetics::DnaType::Nucleus,
                        &gene_library.dna_from_trait_strs(rng, traits),
                    );
                    tile.set_genome(sensors, processors, actuators, dna);
                    tile.processors.life_elapsed =
                        rng.gen_range(0..tile.processors.life_expectancy);
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
                let idx = coord_to_idx(game::consts::WORLD_WIDTH, object.pos.x(), object.pos.y());
                self.occupation[idx] += 1;
                self.objects[game::consts::PLAYER].replace(object);
            }
        }
    }

    pub fn push(&mut self, object: Object) {
        trace!("adding {} to game objects", object.visual.name);
        // update occupation map, if the object is not a tile
        if object.tile.is_none() && object.physics.is_blocking {
            let idx = coord_to_idx(game::consts::WORLD_WIDTH, object.pos.x(), object.pos.y());
            self.occupation[idx] += 1;
        }
        self.objects.push(Some(object));
    }

    /// Remove an object from the world. If the object is not a tile, the occupation map will be
    /// updated.
    pub fn remove(&mut self, idx: usize, object: &Object) {
        if object.tile.is_none() && object.physics.is_blocking {
            self.occupation[idx] -= 1;
        }
        self.objects.remove(idx);
    }

    pub fn extract_by_index(&mut self, index: usize) -> Option<Object> {
        match self.objects.get_mut(index) {
            Some(item) => item.take(),
            None => panic!(" Error: invalid index {}", index),
        }
    }

    /// Extract a blocking object with the given position and return both the object and its index.
    pub fn extract_blocking_with_idx(&mut self, pos: &Position) -> Option<(usize, Option<Object>)> {
        let idx = coord_to_idx(game::consts::WORLD_WIDTH, pos.x(), pos.y());
        if self.occupation[idx] == 0 {
            Some((idx, self.extract_by_index(idx)))
        } else {
            self.extract_non_tile_by_pos(pos)
        }
    }

    pub fn extract_tile_by_pos(&mut self, pos: &Position) -> Option<(usize, Option<Object>)> {
        self.objects
            .iter()
            .position(|opt| {
                opt.as_ref()
                    .map_or(false, |obj| obj.pos.is_equal(pos) && obj.tile.is_some())
            })
            .map(|i| (i, self.extract_by_index(i)))
    }

    pub fn extract_non_tile_by_pos(&mut self, pos: &Position) -> Option<(usize, Option<Object>)> {
        if let Some(non_tile_idx) = self.get_non_tiles().iter().position(|opt| {
            opt.as_ref()
                .map_or(false, |obj| obj.pos.is_equal(pos) && obj.tile.is_none())
        }) {
            let full_idx = non_tile_idx + self.world_tile_count;
            Some((full_idx, self.extract_by_index(full_idx)))
        } else {
            None
        }
    }

    pub fn extract_item_by_pos(&mut self, pos: &Position) -> Option<(usize, Option<Object>)> {
        self.objects
            .iter()
            .position(|opt| {
                opt.as_ref().map_or(false, |obj| {
                    obj.pos.is_equal(pos)
                        && obj.tile.is_none()
                        && obj.item.is_some()
                        && !obj.physics.is_blocking
                })
                // o.pos.is_equal(pos) && o.tile.is_none() && o.item.is_some() && !o.physics.is_blocking
            })
            .map(|i| (i, self.extract_by_index(i)))
    }

    pub fn replace(&mut self, index: usize, mut object: Object) {
        let item = self.objects.get_mut(index);
        match item {
            Some(obj) => {
                // debug!("replace object {} @ index {}", object.visual.name, index);
                let last_xy = object.pos.update();
                // record position changes of non-tiles for efficient occupation queries
                if object.tile.is_none() && object.physics.is_blocking {
                    if let Some((last_x, last_y)) = last_xy {
                        let last_idx = coord_to_idx(game::consts::WORLD_WIDTH, last_x, last_y);
                        let this_idx =
                            coord_to_idx(game::consts::WORLD_WIDTH, object.pos.x(), object.pos.y());
                        self.occupation[last_idx] -= 1;
                        self.occupation[this_idx] += 1;
                    }
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
        let tile_idx = coord_to_idx(game::consts::WORLD_WIDTH, p.x(), p.y());
        let is_blocking_tile = self.objects[tile_idx]
            .as_ref()
            .map_or(false, |obj| obj.physics.is_blocking);

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

    pub const fn get_vector(&self) -> &Vec<Option<Object>> {
        &self.objects
    }

    pub fn get_vector_mut(&mut self) -> &mut Vec<Option<Object>> {
        &mut self.objects
    }

    /// Return a Vec slice with all tiles in the world.
    pub fn get_tiles(&self) -> &[Option<Object>] {
        &self.objects[..self.world_tile_count]
    }

    pub fn get_neighborhood_tiles(&self, pos: Position) -> Neighborhood {
        Neighborhood::new(pos, self.get_vector())
    }

    /// Return a Vec slice with all tiles in the world.
    pub fn get_tiles_mut(&mut self) -> &mut [Option<Object>] {
        &mut self.objects[..self.world_tile_count]
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
            self.objects[idx]
                .as_ref()
                .map_or(false, |o| o.physics.is_blocking_sight)
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
        coord_to_idx(game::consts::WORLD_WIDTH, pt.x, pt.y)
    }

    /// Convert an array index to a point. Defaults to an index based on an array
    fn index_to_point2d(&self, idx: usize) -> rltk::Point {
        let coord = idx_to_coord(game::consts::WORLD_WIDTH as usize, idx);
        rltk::Point::from_tuple(coord)
    }

    fn dimensions(&self) -> rltk::Point {
        rltk::Point::new(game::consts::WORLD_WIDTH, game::consts::WORLD_HEIGHT)
    }
}

static VON_NEUMAN_NEIGHBORHOOD: &[(i32, i32); 4] = &[(-1, 0), (0, -1), (1, 0), (0, 1)];

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
        if self.count != self.bounds.len() {
            while self.count < self.bounds.len() {
                let x = self.bounds[self.count].0 + self.cell_pos.x();
                let y = self.bounds[self.count].1 + self.cell_pos.y();

                self.count += 1;

                let is_in_x_bounds = (0..game::consts::WORLD_WIDTH).contains(&x);
                let is_in_y_bounds = (0..game::consts::WORLD_HEIGHT).contains(&y);
                if is_in_x_bounds && is_in_y_bounds {
                    let tile_obj = &self.buffer[coord_to_idx(game::consts::WORLD_WIDTH, x, y)];
                    if let Some(obj) = tile_obj {
                        if !obj.is_void() {
                            return Some(tile_obj);
                        }
                    }
                }
            }
        }
        None
    }
}

// Convert coordinate to an index in a vector.
pub const fn coord_to_idx(width: i32, x: i32, y: i32) -> usize {
    (y * width + x) as usize
}

/// Convert an vector index to a point.
pub const fn idx_to_coord(width: usize, idx: usize) -> (i32, i32) {
    let x = idx % width;
    let y = idx / width;
    (x as i32, y as i32)
}
