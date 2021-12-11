use crate::entity::action::{self, Action};
use crate::entity::object::Object;
use crate::game::game_objects::GameObjects;
use crate::game::game_state::GameState;
#[cfg(not(target_arch = "wasm32"))]
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub enum Controller {
    Npc(Box<dyn Ai>),
    Player(Player),
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

#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct Player {
    pub primary_action: Box<dyn Action>,
    pub secondary_action: Box<dyn Action>,
    pub quick1_action: Box<dyn Action>,
    pub quick2_action: Box<dyn Action>,
    pub next_action: Option<Box<dyn Action>>,
}

impl Player {
    pub fn new() -> Self {
        Player {
            primary_action: Box::new(action::hereditary::ActPass::default()),
            secondary_action: Box::new(action::hereditary::ActPass::default()),
            quick1_action: Box::new(action::hereditary::ActPass::default()),
            quick2_action: Box::new(action::hereditary::ActPass::default()),
            next_action: None,
        }
    }
}
