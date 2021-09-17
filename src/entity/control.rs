use crate::core::game_objects::GameObjects;
use crate::core::game_state::GameState;
use crate::entity::action::Action;
use crate::entity::object::Object;
use crate::entity::player::PlayerCtrl;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Controller {
    Npc(Box<dyn Ai>),
    Player(PlayerCtrl),
}

#[typetag::serde(tag = "type")]
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
