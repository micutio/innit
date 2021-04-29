use crate::core::game_state::{from_dungeon_level, GameState, Transition};
use crate::core::position::Position;
use crate::core::world::world_gen::{new_monster, Monster, Tile, WorldGen};
use crate::core::{game_objects::GameObjects, innit_env};
use crate::game::{WORLD_HEIGHT, WORLD_WIDTH};
use crate::util::game_rng::{GameRng, RngExtended};
use std::collections::HashSet;

const CA_CYCLES: i32 = 45;

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
    // TODO: Use the `level` parameter to scale object properties in some way.
    // Idea: use level to scale length of dna of generated entities
    fn make_world(&mut self, state: &mut GameState, objects: &mut GameObjects, level: u32) {
        // step 1: generate foundation pattern
        let mid_x = WORLD_WIDTH / 2;
        let mid_y = WORLD_HEIGHT / 2;
        for y in mid_y - 2..mid_y + 2 {
            for x in mid_x - 2..mid_x + 2 {
                objects
                    .get_tile_at(x as usize, y as usize)
                    .replace(Tile::empty(x, y, innit_env().debug_mode));
                self.player_start = (x, y);
                // println!("#1 flipped {}, {}", x, y);
            }
        }

        let mut changed_tiles: HashSet<(i32, i32)> = HashSet::new();
        // step 2: use cellular automaton to fill in and smooth out
        for _ in 0..CA_CYCLES {
            for y in 2..WORLD_HEIGHT - 2 {
                for x in 2..WORLD_WIDTH - 2 {
                    // note whether a cell has changed
                    if update_from_neighbours(objects, &mut state.rng, x, y) {
                        changed_tiles.insert((x, y));
                    }
                }
            }
            // perform actual update
            for (j, k) in &changed_tiles {
                objects
                    .get_tile_at(*j as usize, *k as usize)
                    .replace(Tile::empty(*j, *k, innit_env().debug_mode));
            }
            changed_tiles.clear();
        }

        // world gen done, now insert objects
        place_objects(
            state,
            objects,
            level,
            Transition {
                level: 6,
                value: 500,
            },
        );
    }

    fn get_player_start_pos(&self) -> (i32, i32) {
        self.player_start
    }
}

fn update_from_neighbours(objects: &mut GameObjects, rng: &mut GameRng, x: i32, y: i32) -> bool {
    let directions = [
        // (-1, -1),
        (-1, 0, 4.0),
        // (-1, 1),
        (0, -1, 1.0),
        (0, 1, 1.0),
        // (1, -1),
        (1, 0, 4.0),
        // (1, 1),
    ];

    let mut access_count: f64 = 0.0;
    for (i, j, weight) in directions.iter() {
        let nx = x + i;
        let ny = y + j;
        if nx >= 2 && nx <= (WORLD_WIDTH - 2) && ny >= 2 && ny <= (WORLD_HEIGHT - 2) {
            if let Some(neighbour_tile) = &mut objects.get_tile_at(nx as usize, ny as usize) {
                if !neighbour_tile.physics.is_blocking {
                    access_count += weight;
                }
            }
        }
    }

    rng.flip_with_prob(access_count / 16.0)
}

fn place_objects(
    state: &mut GameState,
    objects: &mut GameObjects,
    level: u32,
    transition: Transition,
) {
    use rand::distributions::WeightedIndex;
    use rand::prelude::*;

    // TODO: Pull spawn tables out of here and pass as parameters in make_world().
    let max_monsters = from_dungeon_level(
        &[
            Transition {
                level: 1,
                value: 200,
            },
            Transition {
                level: 4,
                value: 300,
            },
            transition,
        ],
        level,
    );

    // monster random table
    let bacteria_chance = from_dungeon_level(
        &[
            Transition {
                level: 3,
                value: 15,
            },
            Transition {
                level: 5,
                value: 30,
            },
            Transition {
                level: 7,
                value: 60,
            },
        ],
        level,
    );

    let monster_chances = [
        (Monster::Virus, 60),
        (Monster::Bacteria, bacteria_chance),
        (Monster::Plasmid, 40),
    ];
    let monster_dist = WeightedIndex::new(monster_chances.iter().map(|item| item.1)).unwrap();

    // choose random number of monsters
    let num_monsters = state.rng.gen_range(0..max_monsters);
    for _ in 0..num_monsters {
        // choose random spot for this monster
        let x = state.rng.gen_range(0 + 1..WORLD_WIDTH);
        let y = state.rng.gen_range(0 + 1..WORLD_HEIGHT);

        if !objects.is_pos_occupied(&Position::new(x, y)) {
            let monster_type = monster_chances[monster_dist.sample(&mut state.rng)].0;
            let mut monster = new_monster(state, monster_type, x, y, level);
            monster.alive = true;
            objects.push(monster);
        }
    }
}
