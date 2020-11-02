use crate::core::game_objects::GameObjects;
use crate::entity::action::{Action, PassAction};
use crate::entity::object::Object;
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerCtrl {
    pub primary_action: Box<dyn Action>,
    pub secondary_action: Box<dyn Action>,
    pub quick1_action: Box<dyn Action>,
    pub quick2_action: Box<dyn Action>,
    pub next_action: Option<Box<dyn Action>>,
}

impl PlayerCtrl {
    fn new() -> Self {
        PlayerCtrl {
            primary_action: Box::new(PassAction),
            secondary_action: Box::new(PassAction),
            quick1_action: Box::new(PassAction),
            quick2_action: Box::new(PassAction),
            next_action: None,
        }
    }
}
