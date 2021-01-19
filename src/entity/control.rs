use crate::core::game_objects::GameObjects;
use crate::core::game_state::GameState;
use crate::entity::action::Action;
use crate::entity::object::Object;
use crate::entity::player::PlayerCtrl;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

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
