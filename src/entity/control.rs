use crate::core::game_objects::GameObjects;
use crate::entity::action::Action;
use crate::entity::object::Object;
use crate::entity::player::PlayerCtrl;
use std::fmt::Debug;
use crate::core::game_state::GameState;

#[derive(Debug, Serialize, Deserialize)]
pub enum Controller {
    Npc(Box<dyn Ai>),
    Player(PlayerCtrl),
}

#[typetag::serde(tag = "type")]
pub trait Ai: Debug {
    fn act(
        &mut self,
        state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> Box<dyn Action>;
}
