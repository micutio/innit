use crate::entity::object::{self, Object};
use crate::entity::{act, ai, control, genetics};
use crate::game::{self, ObjectStore, State};
use crate::raws;
use crate::util::rng::RngExtended;
use crate::world_gen::WorldGen;

use casim::ca::{coord_to_idx, Neighborhood, Simulation, VON_NEUMAN_NEIGHBORHOOD};

const CA_CYCLES: i32 = 150;

/// The organics world generator attempts to create organ-like environments e.g., long snaking
/// blood vessels, branching fractal-like lungs, spongy tissue and more.
// TODO: Rename into game::world_gen_ca::CaWorldGenerator and extract ca construction into dedicated file
//       once we have more than one CA variant.
pub struct CaBased {
    player_start: (i32, i32),
    ca_cycle_count: i32,
    ca: Option<Simulation<CaCell>>,
}

impl CaBased {
    pub fn new() -> Self {
        CaBased {
            player_start: (0, 0),
            ca_cycle_count: 0,
            ca: None,
        }
    }
}

impl WorldGen for CaBased {
    // Idea: use level to scale length of dna of generated entities
    fn make_world(
        &mut self,
        state: &mut State,
        objects: &mut ObjectStore,
        spawns: &[raws::spawn::Spawn],
        object_templates: &[raws::object_template::ObjectTemplate],
    ) -> game::RunState {
        // step 1: create ca, if not already there
        if self.ca.is_none() {
            self.ca = Some(make_ca(state));
            self.player_start = (game::WORLD_WIDTH / 2, game::WORLD_HEIGHT / 2);
        }

        // step 2: use cellular automaton to fill in and smooth out
        while self.ca_cycle_count < CA_CYCLES {
            if let Some(ca) = &mut self.ca {
                ca.step();
                // update positions assigned with `true` to floor tiles
                for (idx, cell) in ca.cells().into_iter().enumerate() {
                    if let Some(Some(tile)) = objects.get_vector_mut().get_mut(idx + 1) {
                        if let Some(t) = &mut tile.tile {
                            // TODO: Create constants for morphogen cutoffs and min morphogens
                            t.morphogen = cell.morphogen;
                            if t.morphogen < 0.3 {
                                tile.physics.is_blocking = false;
                                tile.physics.is_blocking_sight = false;
                                tile.visual.glyph = '·';
                                tile.visual.name = "empty tile".into();
                                tile.control = None;
                            } else {
                                tile.physics.is_blocking = true;
                                tile.physics.is_blocking_sight = true;
                                tile.visual.glyph = '◘';
                                tile.visual.name = "wall tile".into();
                                tile.control = Some(control::Controller::Npc(Box::new(ai::AiTile)));
                            }
                        }
                    }
                }
            }
            self.ca_cycle_count += 1;
            if game::innit_env().is_debug_mode {
                return game::RunState::WorldGen;
            }
        }

        // world gen done, now insert objects
        place_objects(state, objects, spawns, object_templates);
        game::RunState::Ticking
    }

    fn get_player_start_pos(&self) -> (i32, i32) {
        self.player_start
    }
}

/// Cell type for the cellular automaton that's used to generate the world.
/// The cellular automaton is based on forest fire mechanics.
/// The cell state is either forested (`GREEN`) or empty. Fire will only propagate between green
/// areas. The final shape of the world is then determined by the 'burnt' areas which reduce the
/// morphogen values of the affected cell and their neighborhood.
/// The attribute **morphogen** is a gradient between burnt and untouched cells. It ranges from 0.0
/// (burnt) to 1.0 (untouched) and determines whether a cell is to be populated by a wall tile upon
/// world creation.
/// The morphogen also allows the world shape to slightly fluctuate over time by introducing a tiny
/// random component in the process of dying and regenerating world cells.
#[derive(Clone, Debug, Default)]
struct CaCell {
    state: CellState,
    burn_count: i32,
    morphogen: f64,
}

#[derive(Clone, Debug)]
enum CellState {
    EMPTY,
    GREEN,
    BURNING,
    BURNT,
}

impl Default for CellState {
    fn default() -> Self {
        Self::EMPTY
    }
}

/// Create a cellular automaton from the tiles of the game world.
fn make_ca(state: &mut State) -> Simulation<CaCell> {
    // init cells
    let mut cells = vec![CaCell::default(); (game::WORLD_WIDTH * game::WORLD_HEIGHT) as usize];
    let mid_x = game::WORLD_WIDTH / 2;
    let mid_y = game::WORLD_HEIGHT / 2;
    // let max_dist =
    //     f64::sqrt((game::WORLD_WIDTH * game::WORLD_WIDTH) as f64 + (game::WORLD_HEIGHT * game::WORLD_HEIGHT) as f64);
    let max_dist = i32::max(game::WORLD_WIDTH, game::WORLD_HEIGHT) as f64;
    for y in 0..game::WORLD_HEIGHT {
        for x in 0..game::WORLD_WIDTH {
            let idx = coord_to_idx(game::WORLD_WIDTH, x, y);
            let cell = &mut cells[idx];
            // let dist_to_mid =
            //     f64::sqrt(f64::powf((mid_x - x) as f64, 2.0) + f64::powf((mid_y - y) as f64, 2.0));
            // let morphogen = 1.0 - f64::min((dist_to_mid * 2.0) / max_dist, 0.01);
            let dist_to_x_border = i32::min(x, game::WORLD_WIDTH - x);
            let dist_to_y_border = i32::min(y, game::WORLD_HEIGHT - y);
            let min_dist_border = i32::min(dist_to_x_border, dist_to_y_border);

            let morphogen = (((min_dist_border * 2) as f64 / max_dist) * 0.20) + 0.50;
            let morph_clamped = f64::min(f64::max(morphogen, 0.01), 1.0);
            if state.rng.flip_with_prob(morph_clamped) {
                cell.state = CellState::GREEN;
            } else {
                cell.state = CellState::EMPTY;
            }
            cell.burn_count = 5;
            cell.morphogen = 1.0;
        }
    }

    let mid_idx = coord_to_idx(game::WORLD_WIDTH, mid_x, mid_y);
    cells[mid_idx].state = CellState::BURNING;

    // transition function
    // 1. propagate fire between burning and green cells
    // 2. slowly diffuse morphogen if there is a gradient between the cell and its neighbors
    let trans_fn = move |cell: &mut CaCell, neigh_it: Neighborhood<CaCell>| {
        let mut is_fire_near = false;
        let mut neigh_count = 0;
        let mut neigh_morphogen = 0.0;
        for n in neigh_it {
            neigh_count += 1;
            neigh_morphogen += n.morphogen;
            if let CellState::BURNING = n.state {
                is_fire_near = true;
            }
        }
        if neigh_count < 4 {
            cell.state = CellState::EMPTY;
            return;
        }

        // propagate fire
        match cell.state {
            CellState::GREEN => {
                if neigh_count == 4 && is_fire_near {
                    cell.state = CellState::BURNING;
                }
            }
            CellState::BURNING => {
                cell.burn_count -= 1;
                if cell.burn_count <= 0 {
                    cell.state = CellState::BURNT;
                    cell.morphogen = 0.0;
                }
            }
            _ => {}
        }

        // burnt cells always have zero mophogen content, this is to maintain the shape of the
        // generated world
        if let CellState::BURNT = cell.state {
            cell.morphogen = 0.0
        } else {
            // propagate morphogen
            let avg_morphogen = neigh_morphogen / neigh_count as f64;
            let diffusion = 0.05 * (avg_morphogen - cell.morphogen);
            cell.morphogen += diffusion;
        }
    };

    Simulation::from_cells(
        game::WORLD_WIDTH,
        game::WORLD_HEIGHT,
        trans_fn,
        VON_NEUMAN_NEIGHBORHOOD,
        cells,
    )
}

fn place_objects(
    state: &mut State,
    objects: &mut ObjectStore,
    spawns: &[raws::spawn::Spawn],
    object_templates: &[raws::object_template::ObjectTemplate],
) {
    use rand::distributions::WeightedIndex;
    use rand::prelude::*;

    // TODO: Set npc number per level via transitions.
    let npc_upper_limit = 50;

    let npc_chances: Vec<(&String, u32)> = spawns
        .iter()
        .map(|s| {
            (
                &s.npc,
                raws::spawn::from_dungeon_level(&s.spawn_transitions, state.dungeon_level),
            )
        })
        .collect();

    let npc_dist = WeightedIndex::new(npc_chances.iter().map(|item| item.1)).unwrap();

    // choose random number of npcs
    let npc_count = state.rng.gen_range(0..npc_upper_limit);
    for _ in 0..npc_count {
        // choose random spot for this npc
        let tile = objects
            .get_tiles()
            .iter()
            .flatten()
            .filter(|t| !t.physics.is_blocking)
            .choose(&mut state.rng);

        if let Some(t) = tile {
            let npc_type = npc_chances[npc_dist.sample(&mut state.rng)].0;
            if let Some(template) = object_templates.iter().find(|t| t.npc.eq(npc_type)) {
                // assign controller
                use control::Controller;
                let controller: Option<Controller> = if let Some(ctrl) = &template.controller {
                    match ctrl.as_str() {
                        "player" => Some(Controller::Player(control::Player::new())),
                        "AiPassive" => Some(Controller::Npc(Box::new(ai::AiPassive))),
                        "AiRandom" => Some(Controller::Npc(Box::new(ai::AiRandom::new()))),
                        "AiRandomWalk" => Some(Controller::Npc(Box::new(ai::AiRandomWalk))),
                        "AiVirus" => Some(Controller::Npc(Box::new(ai::AiVirus::new()))),
                        s => {
                            error! {"Unknown controller type '{}'", s};
                            // Controller::Npc(Box::new(AiPassive))
                            continue;
                        }
                    }
                } else {
                    None
                };

                // generate DNA, either probabilistically or from a template
                use raws::object_template::DnaTemplate;
                let raw_dna = match &template.dna_template {
                    DnaTemplate::Random { genome_len } => state.gene_library.dna_from_size(
                        &mut state.rng,
                        template.dna_type == genetics::DnaType::Rna,
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
                            genetics::TraitFamily::Sensing,
                            genetics::TraitFamily::Processing,
                            genetics::TraitFamily::Actuating,
                        ],
                        template.dna_type == genetics::DnaType::Rna,
                        *genome_len,
                    ),
                    DnaTemplate::Defined { traits } => state
                        .gene_library
                        .dna_from_trait_strs(&mut state.rng, &traits),
                };

                // populate inventory if present
                let inventory_item = if let Some(item) = &template.item {
                    let action_instance = if item.action.is_empty() {
                        None
                    } else {
                        match act::action_from_string(item.action.as_ref()) {
                            Ok(action) => Some(action.clone()),
                            Err(msg) => {
                                error!("error getting action from string: {}", msg);
                                continue;
                            }
                        }
                    };
                    Some(object::InventoryItem::new(&item.name, action_instance))
                } else {
                    None
                };

                // finally cobble everything together and insert the new object into the world
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
