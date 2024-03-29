use crate::entity::control::Controller;
use crate::entity::genetics::{Actuators, Dna, Processors, Sensors};
use crate::game;
use crate::game::env;
use crate::game::State;
use crate::world_gen::Tile;
use crate::{entity::act::Move, game::objects::ObjectStore};

#[test]
fn test_random_ai() {
    use crate::entity::ai::Virus;
    use crate::entity::genetics::{DnaType, GENOME_LEN};
    use crate::entity::object::Object;
    use crate::game::consts::PLAYER;
    use crate::ui::palette;

    let ((p_x, p_y), mut state, mut objects) = _create_minimal_world();

    // test walking in any direction
    #[allow(clippy::option_if_let_else)]
    if let Some(mut player) = objects.extract_by_index(PLAYER) {
        if let Some(action) = player.extract_next_action(&mut state, &mut objects) {
            println!("move test '{}'", &action.to_text());
            assert!(action.get_identifier().contains("move"));
        } else {
            panic!();
        }
        objects.replace(PLAYER, player);
    } else {
        panic!();
    }

    let virus_color = palette().entity_virus;

    // Set up monsters
    let virus_north = Object::new()
        .position_xy(p_x, p_y - 1)
        .living(true)
        .visualize("Virus", 'v', virus_color)
        .physical(true, false, false)
        .genome(
            0.75,
            state
                .gene_library
                .new_genetics(&mut state.rng, DnaType::Rna, true, GENOME_LEN),
        )
        .control(Controller::Npc(Box::new(Virus::new())));

    let virus_east = Object::new()
        .position_xy(p_x + 1, p_y)
        .living(true)
        .visualize("Virus", 'v', virus_color)
        .physical(true, false, false)
        .genome(
            0.75,
            state
                .gene_library
                .new_genetics(&mut state.rng, DnaType::Rna, true, GENOME_LEN),
        )
        .control(Controller::Npc(Box::new(Virus::new())));

    let virus_south = Object::new()
        .position_xy(p_x, p_y + 1)
        .living(true)
        .visualize("Virus", 'v', virus_color)
        .physical(true, false, false)
        .genome(
            0.75,
            state
                .gene_library
                .new_genetics(&mut state.rng, DnaType::Rna, true, GENOME_LEN),
        )
        .control(Controller::Npc(Box::new(Virus::new())));

    objects.push(virus_north);
    objects.push(virus_east);
    objects.push(virus_south);

    // test walking in only west direction
    #[allow(clippy::option_if_let_else)]
    if let Some(mut player) = objects.extract_by_index(PLAYER) {
        if let Some(action) = player.extract_next_action(&mut state, &mut objects) {
            assert_eq!(action.to_text(), "move to West");
        } else {
            panic!();
        }
        objects.replace(PLAYER, player);
    } else {
        panic!();
    }

    let virus_west = Object::new()
        .position_xy(p_x - 1, p_y)
        .living(true)
        .visualize("Virus", 'v', virus_color)
        .physical(true, false, false)
        .genome(
            0.75,
            state
                .gene_library
                .new_genetics(&mut state.rng, DnaType::Rna, true, GENOME_LEN),
        )
        .control(Controller::Npc(Box::new(Virus::new())));

    objects.push(virus_west);

    // test no walk possible
    #[allow(clippy::option_if_let_else)]
    if let Some(mut player) = objects.extract_by_index(PLAYER) {
        if let Some(action) = player.extract_next_action(&mut state, &mut objects) {
            assert_eq!(action.to_text(), "pass");
        } else {
            panic!();
        }
        objects.replace(PLAYER, player);
    } else {
        panic!();
    }
}

fn _create_minimal_world() -> ((i32, i32), State, ObjectStore) {
    use crate::entity::ai::RandomAction;
    use crate::entity::object::Object;
    use crate::game::consts::{WORLD_HEIGHT, WORLD_WIDTH};

    // create game state holding game-relevant information
    let level = 1;
    let state = State::new(level);

    // create blank game world
    let mut objects = ObjectStore::new();
    objects.blank_circle_world();

    let (p_x, p_y) = (WORLD_WIDTH / 2, WORLD_HEIGHT / 3);

    let debug_mode = matches!(env().debug_mode, game::env::GameOption::Enabled);

    // make tiles near the player walkable
    objects
        .get_tile_at_mut(p_x, p_y)
        .replace(Tile::new_floor(p_x, p_y, debug_mode));
    objects
        .get_tile_at_mut(p_x + 1, p_y)
        .replace(Tile::new_floor(p_x + 1, p_y, debug_mode));
    objects
        .get_tile_at_mut(p_x - 1, p_y)
        .replace(Tile::new_floor(p_x - 1, p_y, debug_mode));
    objects
        .get_tile_at_mut(p_x, p_y - 1)
        .replace(Tile::new_floor(p_x, p_y - 1, debug_mode));
    objects
        .get_tile_at_mut(p_x, p_y + 1)
        .replace(Tile::new_floor(p_x, p_y + 1, debug_mode));

    let player = Object::new()
        .position_xy(p_x, p_y)
        .living(true)
        // .visualize("player", '@', colors::WHITE)
        .physical(true, false, false)
        .genome(
            1.0,
            (
                Sensors::default(),
                Processors::default(),
                Actuators {
                    actions: vec![Box::new(Move::new())],
                    max_hp: 1,
                    hp: 1,
                    volume: 1,
                },
                Dna::default(),
            ),
        )
        .control(Controller::Npc(Box::new(RandomAction::new())));

    objects.set_player(player);

    ((p_x, p_y), state, objects)
}
