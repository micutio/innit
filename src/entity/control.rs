use crate::game::game_objects::GameObjects;
use crate::game::game_state::GameState;
use crate::entity::action::Action;
use crate::entity::object::Object;
use crate::entity::player::PlayerCtrl;
#[cfg(not(target_arch = "wasm32"))]
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub enum Controller {
    Npc(Box<dyn Ai>),
    Player(PlayerCtrl),
}

#[cfg_attr(not(target_arch = "wasm32"), typetag::serde(tag = "type"))]
pub trait Ai: AiClone + Debug {
    fn act(
        &mut self,
        state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> Box<dyn Action>;
}

pub trait AiClone {
    fn clone_ai(&self) -> Box<dyn Ai>;
}

impl<T> AiClone for T
where
    T: Ai + Clone + 'static,
{
    fn clone_ai(&self) -> Box<dyn Ai> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Ai> {
    fn clone(&self) -> Self {
        self.clone_ai()
    }
}
