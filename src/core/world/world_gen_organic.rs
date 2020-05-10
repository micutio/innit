

use tcod::console::*;

use crate::core::game_objects::GameObjects;
use crate::core::world::world_gen::{Tile, WorldGen};
use crate::entity::genetics::GeneLibrary;
use crate::game::DEBUG_MODE;
use crate::game::{WORLD_HEIGHT, WORLD_WIDTH};
use crate::ui::game_frontend::{blit_consoles, render_objects, GameFrontend};
use crate::util::game_rng::{GameRng, RngExtended};

const CA_CYCLES: i32 = 2;

/// The organics world generator attempts to create organ-like environments e.g., long snaking blood
/// vessels, branching fractal-like lungs, spongy tissue and more.
pub struct OrganicsWorldGenerator {
    player_start: (i32, i32),
}

impl OrganicsWorldGenerator {
    pub fn new() -> Self {
        OrganicsWorldGenerator {
            player_start: (0, 0),
        }
    }
}

impl WorldGen for OrganicsWorldGenerator {
    fn make_world(
        &mut self,
        game_frontend: &mut GameFrontend,
        game_objects: &mut GameObjects,
        game_rng: &mut GameRng,
        _gene_library: &mut GeneLibrary,
        _level: u32,
    ) {
        // step 1: generate foundation pattern
        for y in 2..WORLD_HEIGHT - 2 {
            for x in 2..WORLD_WIDTH - 2 {
                // debug!("setting tile dna at ({}, {})", x, y);
                let probability = ((f64::from(x) - (f64::from(WORLD_WIDTH) / 2.0)).abs()
                    + (f64::from(y) - (f64::from(WORLD_HEIGHT) / 2.0)).abs())
                    / (f64::from(WORLD_WIDTH + WORLD_HEIGHT));
                println!("#1 probability: {}", probability);
                if game_rng.flip_with_prob((1.0 - probability) / 10.0) {
                    game_objects
                        .get_tile_at(x as usize, y as usize)
                        .replace(Tile::empty(x, y));
                    self.player_start = (x, y);
                    println!("#1 flipped {}, {}", x, y);
                }
            }
            visualize_map(game_frontend, game_objects);
        }

        // step 2: use cellular automaton to fill in and smoothe out
        for y in 2..WORLD_HEIGHT - 2 {
            for x in 2..WORLD_WIDTH - 2 {
                if update_from_neighbors(game_objects, game_rng, x, y) {
                    println!("#2 flipped {}, {}", x, y);
                }
            }
            visualize_map(game_frontend, game_objects);
        }
    }

    fn get_player_start_pos(&self) -> (i32, i32) {
        self.player_start
    }
}

fn visualize_map(game_frontend: &mut GameFrontend, game_objects: &mut GameObjects) {
    if DEBUG_MODE {
        // let ten_millis = time::Duration::from_millis(100);
        // thread::sleep(ten_millis);

        game_frontend.con.clear();
        render_objects(game_frontend, game_objects);
        blit_consoles(game_frontend);
        game_frontend.root.flush();
    }
}

fn update_from_neighbors(
    game_objects: &mut GameObjects,
    game_rng: &mut GameRng,
    x: i32,
    y: i32,
) -> bool {
    let directions = [
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, -1),
        (0, 1),
        (1, -1),
        (1, 0),
        (1, 1),
    ];

    let mut neighbor_count: usize = 0;
    let mut access_count: usize = 0;
    for (i, j) in directions.iter() {
        let nx = x + i;
        let ny = y + j;
        if nx >= 2 && nx <= (WORLD_WIDTH - 2) && ny >= 2 && ny <= (WORLD_HEIGHT - 2) {
            neighbor_count += 1;
            if let Some(neighbor_tile) = &mut game_objects.get_tile_at(nx as usize, ny as usize) {
                if neighbor_tile.physics.is_blocking {
                    access_count += 1;
                }
            }
        }
    }

    let mut has_changed: bool = false;
    println!(
        "#2 probability: {}",
        (access_count as f64 / neighbor_count as f64) as f64
    );
    if game_rng.flip_with_prob((access_count as f64 / neighbor_count as f64) / 10.0 as f64) {
        has_changed = true;
        game_objects
            .get_tile_at(x as usize, y as usize)
            .replace(Tile::empty(x, y));
    }
    has_changed
}
