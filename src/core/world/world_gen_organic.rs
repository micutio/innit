use crate::core::game_objects::GameObjects;
use crate::core::game_state::{from_dungeon_level, Transition};
use crate::core::world::world_gen::{new_monster, Monster, Tile, WorldGen};
use crate::entity::genetics::GeneLibrary;
use crate::entity::object::Position;
use crate::game::{WORLD_HEIGHT, WORLD_WIDTH};
use crate::ui::game_frontend::{blit_consoles, render_objects, GameFrontend};
use crate::util::game_rng::{GameRng, RngExtended};

use std::collections::HashSet;

use crate::core::game_env::GameEnv;
use tcod::console::*;

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
    fn make_world(
        &mut self,
        env: &GameEnv,
        frontend: &mut GameFrontend,
        objects: &mut GameObjects,
        rng: &mut GameRng,
        gene_library: &mut GeneLibrary,
        level: u32,
    ) {
        // step 1: generate foundation pattern
        let mid_x = WORLD_WIDTH / 2;
        let mid_y = WORLD_HEIGHT / 2;
        for y in mid_y - 2..mid_y + 2 {
            for x in mid_x - 2..mid_x + 2 {
                objects
                    .get_tile_at(x as usize, y as usize)
                    .replace(Tile::empty(x, y, env.debug_mode));
                self.player_start = (x, y);
                // println!("#1 flipped {}, {}", x, y);
            }
            visualize_map(env, frontend, objects);
        }

        let mut changed_tiles: HashSet<(i32, i32)> = HashSet::new();
        // step 2: use cellular automaton to fill in and smooth out
        for _ in 0..CA_CYCLES {
            for y in 2..WORLD_HEIGHT - 2 {
                for x in 2..WORLD_WIDTH - 2 {
                    // note whether a cell has changed
                    if update_from_neighbours(objects, rng, x, y) {
                        changed_tiles.insert((x, y));
                    }
                }
            }
            // perform actual update
            for (j, k) in &changed_tiles {
                objects
                    .get_tile_at(*j as usize, *k as usize)
                    .replace(Tile::empty(*j, *k, env.debug_mode));
            }
            changed_tiles.clear();
            visualize_map(env, frontend, objects);
        }

        // world gen done, now insert objects
        place_objects(
            objects,
            rng,
            gene_library,
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

fn visualize_map(env: &GameEnv, game_frontend: &mut GameFrontend, game_objects: &mut GameObjects) {
    if env.debug_mode {
        // let ten_millis = time::Duration::from_millis(100);
        // thread::sleep(ten_millis);

        game_frontend.con.clear();
        render_objects(env, game_frontend, game_objects);
        blit_consoles(game_frontend);
        game_frontend.root.flush();
    }
}

// TODO: If return true, flip to walkable, if false flip back to wall!
fn update_from_neighbours(
    game_objects: &mut GameObjects,
    game_rng: &mut GameRng,
    x: i32,
    y: i32,
) -> bool {
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
            if let Some(neighbour_tile) = &mut game_objects.get_tile_at(nx as usize, ny as usize) {
                if !neighbour_tile.physics.is_blocking {
                    access_count += weight;
                }
            }
        }
    }

    game_rng.flip_with_prob(access_count / 16.0)
}

fn place_objects(
    objects: &mut GameObjects,
    game_rng: &mut GameRng,
    gene_library: &mut GeneLibrary,
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

    let monster_chances = [("virus", 80), ("bacteria", bacteria_chance)];
    let monster_dist = WeightedIndex::new(monster_chances.iter().map(|item| item.1)).unwrap();

    // choose random number of monsters
    let num_monsters = game_rng.gen_range(0, max_monsters + 1);
    for _ in 0..num_monsters {
        // choose random spot for this monster
        let x = game_rng.gen_range(0 + 1, WORLD_WIDTH);
        let y = game_rng.gen_range(0 + 1, WORLD_HEIGHT);

        if !objects.is_pos_occupied(&Position::new(x, y)) {
            let mut monster = match monster_chances[monster_dist.sample(game_rng)].0 {
                "virus" => new_monster(game_rng, &gene_library, Monster::Virus, x, y, level),
                "bacteria" => new_monster(game_rng, &gene_library, Monster::Bacteria, x, y, level),
                _ => unreachable!(),
            };

            monster.alive = true;
            objects.push(monster);
        }
    }
}
