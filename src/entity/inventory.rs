use crate::entity::object::Object;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Inventory {
    pub items: Vec<Object>,
}

impl Inventory {
    pub fn new() -> Self {
        Inventory { items: vec![] }
    }
}
