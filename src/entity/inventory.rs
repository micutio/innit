use crate::entity::{act::Action, object::Object};

#[cfg(not(target_arch = "wasm32"))]
use serde::{Deserialize, Serialize};

#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[derive(Debug, Default)]
pub struct Inventory {
    /// A list of items contained in this inventory.
    pub items: Vec<Object>,

    /// A list of actions pertaining this inventory, mostly dropping items.
    pub inv_actions: Vec<Box<dyn Action>>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            inv_actions: Vec::new(),
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[derive(Debug, Default, Clone)]
pub struct Item {
    pub description: String,
    pub use_action: Option<Box<dyn Action>>,
}

impl Item {
    pub fn new<S: Into<String>>(descr: S, use_action: Option<Box<dyn Action>>) -> Self {
        Self {
            description: descr.into(),
            use_action,
        }
    }
}
