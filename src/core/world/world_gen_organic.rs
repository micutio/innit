use crate::core::game_state::GameState;
use crate::core::position::Position;
use crate::core::world::{Tile, WorldGen};
use crate::core::{game_objects::GameObjects, innit_env};
use crate::entity::action::action_from_string;
use crate::entity::ai::AiPassive;
use crate::entity::ai::AiRandom;
use crate::entity::ai::AiRandomWalk;
use crate::entity::ai::AiVirus;
use crate::entity::control::Controller;
use crate::entity::genetics::DnaType;
use crate::entity::genetics::TraitFamily;
use crate::entity::object::InventoryItem;
use crate::entity::object::Object;
use crate::entity::player::PlayerCtrl;
use crate::game::{RunState, WORLD_HEIGHT, WORLD_WIDTH};
use crate::raws::object_template::DnaTemplate;
use crate::raws::object_template::ObjectTemplate;
use crate::raws::spawn::{from_dungeon_level, Spawn};
use crate::util::game_rng::{GameRng, RngExtended};

use casim::ca::{von_neuman, Simulation};
use std::collections::HashSet;

const CA_CYCLES: i32 = 65;

/// The organics world generator attempts to create organ-like environments e.g., long snaking blood
/// vessels, branching fractal-like lungs, spongy tissue and more.
pub struct OrganicsWorldGenerator {
    player_start: (i32, i32),
    requires_foundation: bool,
    ca_cycle_count: i32,
    ca_x: i32,
    ca_y: i32,
    ca_end_x: i32,
    ca_end_y: i32,
    changed_tiles: HashSet<(i32, i32)>,
}

impl OrganicsWorldGenerator {
    pub fn new() -> Self {
        OrganicsWorldGenerator {
            player_start: (0, 0),
            requires_foundation: true,
            ca_cycle_count: 0,
            ca_x: 2,
            ca_y: 2,
            ca_end_x: WORLD_WIDTH - 2,
            ca_end_y: WORLD_HEIGHT - 2,
            changed_tiles: HashSet::new(),
        }
    }
}

impl WorldGen for OrganicsWorldGenerator {
    // TODO: Use the `level` parameter to scale object properties in some way.
    // Idea: use level to scale length of dna of generated entities
    fn make_world(
        &mut self,
        state: &mut GameState,
        objects: &mut GameObjects,
        spawns: &[Spawn],
        object_templates: &[ObjectTemplate],
    ) -> RunState {
        // step 1: generate foundation pattern
        if self.requires_foundation {
            let mid_x = WORLD_WIDTH / 2;
            let mid_y = WORLD_HEIGHT / 2;
            for y in mid_y - 2..mid_y + 2 {
                for x in mid_x - 2..mid_x + 2 {
                    objects
                        .get_tile_at(x as usize, y as usize)
                        .replace(Tile::empty(x, y, innit_env().debug_mode));
                    self.player_start = (x, y);
                }
            }
        }
        self.requires_foundation = false;

        // step 2: use cellular automaton to fill in and smooth out
        while self.ca_cycle_count < CA_CYCLES {
            while self.ca_y <= self.ca_end_y {
                while self.ca_x <= self.ca_end_x {
                    // note whether a cell has changed
                    if update_from_neighbours(objects, &mut state.rng, self.ca_x, self.ca_y) {
                        self.changed_tiles.insert((self.ca_x, self.ca_y));
                    }
                    self.ca_x += 1;
                }
                self.ca_x = 2;
                self.ca_y += 1;
            }
            self.ca_y = 2;
            // perform actual update
            for (j, k) in &self.changed_tiles {
                objects
                    .get_tile_at(*j as usize, *k as usize)
                    .replace(Tile::empty(*j, *k, innit_env().debug_mode));
            }
            self.ca_cycle_count += 1;
            self.changed_tiles.clear();
            info!("CA cycle {0}", self.ca_cycle_count);
            if innit_env().debug_mode {
                return RunState::WorldGen;
            }
        }

        // world gen done, now insert objects
        place_objects(state, objects, spawns, object_templates);
        RunState::Ticking
    }

    fn get_player_start_pos(&self) -> (i32, i32) {
        self.player_start
    }
}

fn make_ca(
) -> Simulation<bool, dyn FnMut(&mut bool, &[&bool]), dyn Fn(i32, i32, i32, i32) -> Vec<(i32, i32)>>
{
    let trans_fn = |cell: &mut bool, neighs: &[&bool]| {
        let mut trues: i32 = 0;
        let mut falses: i32 = 0;
        for n in neighs {
            if **n {
                trues += 1;
            } else {
                falses += 1;
            }
        }

        if trues >= falses {
            *cell = true;
        }
        if falses > trues {
            *cell = false;
        }
    };

    let cells = vec![false, true, false, true, false, true, false, true, false];

    assert!(cells.len() == 9);

    let i = Simulation::from_cells(3, 3, trans_fn, von_neuman, cells);
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
    spawns: &[Spawn],
    object_templates: &[ObjectTemplate],
) {
    use rand::distributions::WeightedIndex;
    use rand::prelude::*;

    // TODO: Set monster number per level via transitions.
    let max_monsters = 100;

    let monster_chances: Vec<(&String, u32)> = spawns
        .iter()
        .map(|s| {
            (
                &s.npc,
                from_dungeon_level(&s.spawn_transitions, state.dungeon_level),
            )
        })
        .collect();

    let monster_dist = WeightedIndex::new(monster_chances.iter().map(|item| item.1)).unwrap();

    // choose random number of monsters
    let num_monsters = state.rng.gen_range(0..max_monsters);
    for _ in 0..num_monsters {
        // choose random spot for this monster
        // TODO: Make sure coordinates are accessible
        let x = state.rng.gen_range(0 + 1..WORLD_WIDTH);
        let y = state.rng.gen_range(0 + 1..WORLD_HEIGHT);

        if !objects.is_pos_occupied(&Position::new(x, y)) {
            let npc_type = monster_chances[monster_dist.sample(&mut state.rng)].0;
            // TODO: maybe build an object factory around all this to make it re-usable.
            if let Some(template) = object_templates.iter().find(|t| t.npc.eq(npc_type)) {
                let controller: Option<Controller> = if let Some(ctrl) = &template.controller {
                    match ctrl.as_str() {
                        "player" => Some(Controller::Player(PlayerCtrl::new())),
                        "AiPassive" => Some(Controller::Npc(Box::new(AiPassive))),
                        "AiRandom" => Some(Controller::Npc(Box::new(AiRandom::new()))),
                        "AiRandomWalk" => Some(Controller::Npc(Box::new(AiRandomWalk))),
                        "AiVirus" => Some(Controller::Npc(Box::new(AiVirus::new()))),
                        s => {
                            error! {"Unknown controller type '{}'", s};
                            // Controller::Npc(Box::new(AiPassive))
                            continue;
                        }
                    }
                } else {
                    None
                };

                let raw_dna = match &template.dna_template {
                    DnaTemplate::Random { genome_len } => state.gene_library.new_dna(
                        &mut state.rng,
                        template.dna_type == DnaType::Rna,
                        *genome_len,
                    ),
                    DnaTemplate::Distributed {
                        s_rate,
                        p_rate,
                        a_rate,
                        genome_len,
                    } => state.gene_library.dna_from_distribution(
                        &mut state.rng,
                        &[*s_rate, *p_rate, *a_rate],
                        &[
                            TraitFamily::Sensing,
                            TraitFamily::Processing,
                            TraitFamily::Actuating,
                        ],
                        template.dna_type == DnaType::Rna,
                        *genome_len,
                    ),
                    DnaTemplate::Defined { traits } => state
                        .gene_library
                        .trait_strs_to_dna(&mut state.rng, &traits),
                };

                let inventory_item = if let Some(item) = &template.item {
                    let action_instance = if item.action.is_empty() {
                        None
                    } else {
                        match action_from_string(item.action.as_ref()) {
                            Ok(action) => Some(action.clone()),
                            Err(msg) => {
                                error!("error getting action from string: {}", msg);
                                continue;
                            }
                        }
                    };
                    Some(InventoryItem::new(&item.name, action_instance))
                } else {
                    None
                };

                let new_npc = Object::new()
                    .position(x, y)
                    .living(true)
                    .visualize(template.npc.as_str(), template.glyph, template.color)
                    .physical(
                        template.physics.is_blocking,
                        template.physics.is_blocking_sight,
                        template.physics.is_always_visible,
                    )
                    .control_opt(controller)
                    .genome(
                        template.stability,
                        state
                            .gene_library
                            .dna_to_traits(template.dna_type, &raw_dna),
                    )
                    .itemize(inventory_item);

                objects.push(new_npc);
            } else {
                error!("No object template found for NPC type '{}'", npc_type);
            }
        }
    }
}
