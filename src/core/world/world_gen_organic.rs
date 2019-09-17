use core::game_objects::GameObjects;
use core::world::world_gen::WorldGen;
use entity::genetics::GeneLibrary;
use ui::game_frontend::GameFrontend;
use util::game_rng::GameRng;

/// The organics world generator attempts to create organ-like environments e.g., long snaking blood
/// vessels, branching fractal-like lungs, spongy tissue and more.
pub struct OrganicsWorldGenerator {
    player_start: (i32, i32),
}

impl WorldGen for OrganicsWorldGenerator {
    fn make_world(
        &mut self,
        game_frontend: &mut GameFrontend,
        game_objects: &mut GameObjects,
        game_rng: &mut GameRng,
        gene_library: &mut GeneLibrary,
        level: u32,
    ) {
    }

    fn get_player_start_pos(&self) -> (i32, i32) {
        self.player_start
    }
}
