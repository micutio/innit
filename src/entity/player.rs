//! Game settings pertaining to the player.
//! This defines player actions, key bindings and more.

use crate::entity::action::{hereditary::ActPass, Action};

#[cfg(not(target_arch = "wasm32"))]
use serde::{Deserialize, Serialize};

pub const PLAYER: usize = 0; // player object reference, index of the object vector
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
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
            primary_action: Box::new(ActPass::default()),
            secondary_action: Box::new(ActPass::default()),
            quick1_action: Box::new(ActPass::default()),
            quick2_action: Box::new(ActPass::default()),
            next_action: None,
        }
    }
}
