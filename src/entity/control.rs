use crate::core::game_objects::GameObjects;
use crate::entity::action::{Action, PassAction};
use crate::entity::object::Object;
use crate::entity::player::PlayerCtrl;
use crate::util::game_rng::GameRng;
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize)]
pub enum Controller {
    Npc(Box<dyn Ai>),
    Player(PlayerCtrl),
}

#[typetag::serde(tag = "type")]
pub trait Ai: Debug {
    fn act(
        &self,
        object: &mut Object,
        game_objects: &mut GameObjects,
        game_rng: &mut GameRng,
    ) -> Box<dyn Action>;
}
