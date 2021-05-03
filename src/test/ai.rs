use crate::core::game_state::GameState;
use crate::core::innit_env;
use crate::core::world::world_gen::Tile;
use crate::entity::control::Controller;
use crate::entity::genetics::{Actuators, Dna, Processors, Sensors};
use crate::{core::game_objects::GameObjects, entity::action::hereditary::ActMove};

#[test]
fn test_random_ai() {
    use crate::core::world::world_gen::new_monster;
    use crate::core::world::world_gen::Monster;
    use crate::entity::player::PLAYER;

    let ((p_x, p_y), mut state, mut objects) = _create_minimal_world();

    // test walking in any direction
    if let Some(mut player) = objects.extract_by_index(PLAYER) {
        if let Some(action) = player.extract_next_action(&mut state, &mut objects) {
            println!("move test '{}'", &action.to_text());
            assert!(action.get_identifier().contains("move"))
        } else {
            panic!();
        }
        objects.replace(PLAYER, player);
    } else {
        panic!();
    }

    // Set up monsters
    let virus_north = new_monster(&mut state, Monster::Virus, p_x, p_y - 1, 0);

    let virus_east = new_monster(&mut state, Monster::Virus, p_x + 1, p_y, 0);

    let virus_south = new_monster(&mut state, Monster::Virus, p_x, p_y + 1, 0);

    objects.push(virus_north);
    objects.push(virus_east);
    objects.push(virus_south);

    // test walking in only west direction
    if let Some(mut player) = objects.extract_by_index(PLAYER) {
        if let Some(action) = player.extract_next_action(&mut state, &mut objects) {
            assert_eq!(action.to_text(), "move to West")
        } else {
            panic!();
        }
        objects.replace(PLAYER, player);
    } else {
        panic!();
    }

    let virus_west = new_monster(&mut state, Monster::Virus, p_x - 1, p_y, 0);

    objects.push(virus_west);

    // test no walk possible
    if let Some(mut player) = objects.extract_by_index(PLAYER) {
        if let Some(action) = player.extract_next_action(&mut state, &mut objects) {
            assert_eq!(action.to_text(), "pass")
        } else {
            panic!();
        }
        objects.replace(PLAYER, player);
    } else {
        panic!();
    }
}

fn _create_minimal_world() -> ((i32, i32), GameState, GameObjects) {
    use crate::entity::ai::AiRandom;
    use crate::entity::object::Object;
    use crate::game::{WORLD_HEIGHT, WORLD_WIDTH};

    // create game state holding game-relevant information
    let level = 1;
    let state = GameState::new(level);

    // create blank game world
    let mut objects = GameObjects::new();
    objects.blank_world();

    let (p_x, p_y) = (WORLD_WIDTH / 2, WORLD_HEIGHT / 3);

    // make tiles near the player walkable
    objects
        .get_tile_at(p_x as usize, p_y as usize)
        .replace(Tile::empty(p_x, p_y, innit_env().debug_mode));
    objects
        .get_tile_at((p_x + 1) as usize, p_y as usize)
        .replace(Tile::empty(p_x + 1, p_y, innit_env().debug_mode));
    objects
        .get_tile_at((p_x - 1) as usize, p_y as usize)
        .replace(Tile::empty(p_x - 1, p_y, innit_env().debug_mode));
    objects
        .get_tile_at(p_x as usize, (p_y - 1) as usize)
        .replace(Tile::empty(p_x, p_y - 1, innit_env().debug_mode));
    objects
        .get_tile_at(p_x as usize, (p_y + 1) as usize)
        .replace(Tile::empty(p_x, p_y + 1, innit_env().debug_mode));

    let player = Object::new()
        .position(p_x, p_y)
        .living(true)
        // .visualize("player", '@', colors::WHITE)
        .physical(true, false, false)
        .genome(
            1.0,
            (
                Sensors::default(),
                Processors::default(),
                Actuators {
                    actions: vec![Box::new(ActMove::new())],
                    max_hp: 1,
                    hp: 1,
                    volume: 1,
                },
                Dna::default(),
            ),
        )
        .control(Controller::Npc(Box::new(AiRandom::new())));

    objects.set_player(player);

    ((p_x, p_y), state, objects)
}
