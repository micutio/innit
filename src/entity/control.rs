use crate::core::game_objects::GameObjects;
use crate::entity::action::Action;
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
        objects: &mut GameObjects,
        rng: &mut GameRng,
    ) -> Box<dyn Action>;
}
