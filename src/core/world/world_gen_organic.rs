use crate::core::game_state::GameState;
use crate::core::world::WorldGen;
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

use casim::ca::{coord_to_idx, Neighborhood, Simulation, VON_NEUMAN_NEIGHBORHOOD};
use rand::Rng;

const CA_CYCLES: i32 = 500;
const GROWTH_PROTEIN_CUTOFF: f64 = 0.1;
const BASE_ELEVATION: f64 = 500.0;

/// The organics world generator attempts to create organ-like environments e.g., long snaking blood
/// vessels, branching fractal-like lungs, spongy tissue and more.
pub struct OrganicsWorldGenerator {
    player_start: (i32, i32),
    ca_cycle_count: i32,
    ca: Option<Simulation<ErosionCell>>,
}

impl OrganicsWorldGenerator {
    pub fn new() -> Self {
        OrganicsWorldGenerator {
            player_start: (0, 0),
            ca_cycle_count: 0,
            ca: None,
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
        // step 1: create ca, if not already there
        if self.ca.is_none() {
            self.ca = Some(make_ca(state));
            self.player_start = (WORLD_WIDTH / 2, WORLD_HEIGHT / 2);
        }

        // step 2: use cellular automaton to fill in and smooth out
        while self.ca_cycle_count < CA_CYCLES {
            info!("CA cycle {0}", self.ca_cycle_count);
            if let Some(ca) = &mut self.ca {
                ca.step();
                // update positions assigned with `true` to floor tiles
                for (idx, cell) in ca.cells().into_iter().enumerate() {
                    if let Some(Some(tile)) = objects.get_vector_mut().get_mut(idx) {
                        if let Some(t) = &mut tile.tile {
                            t.growth_protein = f64::max(cell.elevation / BASE_ELEVATION, 0.0);
                            if (t.growth_protein < GROWTH_PROTEIN_CUTOFF) {
                                println!(
                                    "growth protein: {}, true elevation: {}",
                                    t.growth_protein,
                                    cell.elevation / BASE_ELEVATION
                                );
                            }

                            if t.growth_protein < GROWTH_PROTEIN_CUTOFF {
                                tile.physics.is_blocking = false;
                                tile.physics.is_blocking_sight = false;
                                tile.visual.glyph = '·';
                                tile.visual.name = "empty tile".into();
                            } else {
                                tile.physics.is_blocking = true;
                                tile.physics.is_blocking_sight = true;
                                tile.visual.glyph = '◘';
                                tile.visual.name = "wall tile".into();
                            }
                        }
                    }
                }
            }
            self.ca_cycle_count += 1;
            if innit_env().is_debug_mode {
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

#[derive(Clone, Debug, Default)]
struct ErosionCell {
    pub elevation: f64,
    pub water_lvl: f64,
    pub is_drain: bool,
}

/// Create a cellular automaton from the tiles of the game world.
fn make_ca(state: &mut GameState) -> Simulation<ErosionCell> {
    // init cells
    let mut cells = vec![ErosionCell::default(); (WORLD_WIDTH * WORLD_HEIGHT) as usize];
    let mid_x = WORLD_WIDTH / 2;
    let mid_y = WORLD_HEIGHT / 2;
    let min_elevation = BASE_ELEVATION * 0.75;
    let max_elevation = BASE_ELEVATION;
    let dist_to_max =
        f64::sqrt((WORLD_WIDTH * WORLD_WIDTH) as f64 + (WORLD_HEIGHT * WORLD_HEIGHT) as f64);
    for y in 0..WORLD_HEIGHT {
        for x in 0..WORLD_WIDTH {
            let idx = coord_to_idx(WORLD_WIDTH, x, y);
            if i32::abs(mid_y - y) < 1 && i32::abs(mid_x - x) < 1 {
                // turn center cells into drains
                cells[idx].is_drain = true;
                cells[idx].elevation = -100_000_000.0;
            } else {
                // all other cells are getting semi-random elevation
                let dist_to_mid = f64::sqrt(
                    f64::powf((mid_x - x) as f64, 2.0) + f64::powf((mid_y - y) as f64, 2.0),
                );
                let adjusted_max_elev = dist_to_mid * 2.0;
                let adjusted_min_elev = dist_to_mid;
                // cells[idx].elevation = state.rng.gen_range(adjusted_min_elev..adjusted_max_elev);
                cells[idx].elevation = state.rng.gen_range(min_elevation..max_elevation);
                cells[idx].is_drain = false;
                // cells[idx].elevation = 100.0;
                println!(
                    "({},{}) elevation {} from {:?}",
                    x,
                    y,
                    cells[idx].elevation,
                    adjusted_min_elev..adjusted_max_elev
                );
            }
        }
    }

    // define transition function
    let mut rng = GameRng::new_from_u64_seed(0);
    let trans_fn = move |cell: &mut ErosionCell, neigh_it: Neighborhood<ErosionCell>| {
        // let t_count = neigh_it.into_iter().filter(|n| **n).count();
        const SOIL_HARDNESS: f64 = 0.99;
        const RAINFALL_PROB: f64 = 0.1;
        if !cell.is_drain {
            // add rain
            if rng.flip_with_prob(RAINFALL_PROB) {
                cell.water_lvl += 1.0;
            }
        }

        if !cell.is_drain && cell.water_lvl > 0.0 {
            // perform flow
            // 1. find neighbour with minimal elevation
            if let Some(target) = neigh_it.min_by_key(|n| (n.elevation + n.water_lvl) as i32) {
                let mut flow_amount =
                    0.5 * (cell.elevation + cell.water_lvl - target.elevation - target.water_lvl);
                if flow_amount > 0.0 {
                    // first erode
                    let erosion = flow_amount * (1.0 - SOIL_HARDNESS);
                    cell.elevation -= erosion;
                    // now erosion has changed the amount of flow needed to equalise the water level
                    // recalculate flow amount
                    flow_amount = 0.5
                        * (cell.elevation + cell.water_lvl - target.elevation - target.water_lvl)
                            as f64;
                    cell.water_lvl -= flow_amount;
                } else {
                    // this cell is the lowest cell
                    let erosion = -flow_amount * (1.0 - SOIL_HARDNESS);
                    // now erosion has changed the amount of flow needed to equalise the water level
                    // recalculate flow amount
                    flow_amount = 0.5
                        * ((target.elevation - erosion) + target.water_lvl
                            - cell.elevation
                            - cell.water_lvl);
                    cell.water_lvl += flow_amount;
                }
            }
        }

        // reset drains
        if cell.is_drain {
            cell.water_lvl = 0.0;
            cell.elevation = -100_000_00.0;
        }
    };

    Simulation::from_cells(
        WORLD_WIDTH,
        WORLD_HEIGHT,
        trans_fn,
        VON_NEUMAN_NEIGHBORHOOD,
        cells,
    )
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
    let max_monsters = 50;

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
        let tile = objects
            .get_tiles()
            .iter()
            .flatten()
            .filter(|t| !t.physics.is_blocking)
            .choose(&mut state.rng);

        if let Some(t) = tile {
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
                    .position(t.pos.x, t.pos.y)
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
