//! Game settings pertaining to the player.
//! This defines player actions, key bindings and more.

use crate::entity::action::{Action, PassAction};

pub const PLAYER: usize = 0; // player object reference, index of the object vector
#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerCtrl {
    pub primary_action: Box<dyn Action>,
    pub secondary_action: Box<dyn Action>,
    pub quick1_action: Box<dyn Action>,
    pub quick2_action: Box<dyn Action>,
    pub next_action: Option<Box<dyn Action>>,
}

impl PlayerCtrl {
    pub fn new() -> Self {
        PlayerCtrl {
            primary_action: Box::new(PassAction),
            secondary_action: Box::new(PassAction),
            quick1_action: Box::new(PassAction),
            quick2_action: Box::new(PassAction),
            next_action: None,
        }
    }
}
