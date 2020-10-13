use crate::core::game_objects::GameObjects;
use crate::core::game_state::GameState;
use crate::entity::action::MoveAction;
use crate::entity::genetics::{Actuators, Dna, Processors, Sensors};
use crate::player::PLAYER;

// TODO: test walking in any direction
// TODO: test walking in only one possible direction
// TODO: test blocked by monsters => only can pass
// TODO: extend available actions and tests to include attacking

fn create_minimal_world() -> ((i32, i32), GameState, GameObjects) {
    use crate::entity::ai::RandomAi;
    use crate::entity::object::Object;
    use crate::game::{WORLD_HEIGHT, WORLD_WIDTH};

    // create game state holding game-relevant information
    let level = 1;
    let mut game_state = GameState::new(level);

    // create blank game world
    let mut game_objects = GameObjects::new();
    game_objects.blank_world();

    let (p_x, p_y) = (WORLD_WIDTH / 2, WORLD_HEIGHT / 3);

    let player = Object::new()
        .position(p_x, p_y)
        .living(true)
        // .visualize("player", '@', colors::WHITE)
        .physical(true, false, false)
        .genome((
            Sensors::default(),
            Processors::default(),
            Actuators {
                actions: vec![Box::new(MoveAction::new())],
                hp: 0,
            },
            Dna::default(),
        ))
        .ai(Box::new(RandomAi::new()));

    game_objects.set_player(player);

    ((p_x, p_y), game_state, game_objects)
}

#[test]
fn test_random_ai() {
    use crate::core::world::world_gen::new_monster;
    use crate::core::world::world_gen::Monster;

    let ((p_x, p_y), mut game_state, mut game_objects) = create_minimal_world();

    // test walking in any direction
    if let Some(mut player) = game_objects.extract(PLAYER) {
        if let Some(action) = player.get_next_action(&mut game_objects, &mut game_state.game_rng) {
            assert!(action.get_identifier().contains("move"))
        } else {
            panic!();
        }
        game_objects.replace(PLAYER, player);
    } else {
        panic!();
    }

    // // TODO: Set up monsters
    let virus_north = new_monster(
        &mut game_state.game_rng,
        &game_state.gene_library,
        Monster::Virus,
        p_x,
        p_y - 1,
        0,
    );

    let virus_east = new_monster(
        &mut game_state.game_rng,
        &game_state.gene_library,
        Monster::Virus,
        p_x + 1,
        p_y,
        0,
    );

    let virus_south = new_monster(
        &mut game_state.game_rng,
        &game_state.gene_library,
        Monster::Virus,
        p_x,
        p_y + 1,
        0,
    );

    game_objects.push(virus_north);
    game_objects.push(virus_east);
    game_objects.push(virus_south);

    // test walking in only west direction
    if let Some(mut player) = game_objects.extract(PLAYER) {
        if let Some(action) = player.get_next_action(&mut game_objects, &mut game_state.game_rng) {
            assert_eq!(action.get_identifier(), "move west")
        } else {
            panic!();
        }
        game_objects.replace(PLAYER, player);
    } else {
        panic!();
    }
}
