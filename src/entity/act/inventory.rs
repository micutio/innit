//! This module contains actions that are automatically available to all objects with an inventory.

use crate::entity::act::{self, Action};
use crate::entity::object::Object;
use crate::game;
use crate::game::objects::GameObjects;
use crate::game::game_state::{GameState, MessageLog};

use serde::{Deserialize, Serialize};

/// Pick up an item and store it in the inventory.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PickUpItem;

#[cfg_attr(not(target_arch = "wasm32"), typetag::serde)]
impl Action for PickUpItem {
    fn perform(
        &self,
        state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> act::ActionResult {
        if let Some((index, Some(target_obj))) = objects.extract_item_by_pos(&owner.pos) {
            // do stuff with object
            if target_obj.item.is_some() {
                if owner.inventory.items.len() < owner.actuators.volume as usize {
                    // only add object if it has in item tag
                    state.log.add(
                        format!(
                            "{} picked up a {}",
                            owner.visual.name, &target_obj.visual.name
                        ),
                        game::game_state::MsgClass::Info,
                    );
                    owner.add_to_inventory(target_obj);

                    // keep the object vector neat and tidy
                    objects.get_vector_mut().remove(index);

                    return act::ActionResult::Success {
                        callback: act::ObjectFeedback::NoFeedback,
                    };
                } else {
                    state
                        .log
                        .add("Your inventory is full!", game::game_state::MsgClass::Info);
                }
            }
            //else {
            // otherwise put it back into the world
            //}
            objects.replace(index, target_obj);
            act::ActionResult::Failure
        } else {
            act::ActionResult::Failure
        }
    }

    fn set_target(&mut self, _t: act::Target) {}

    fn set_level(&mut self, _lvl: i32) {}

    fn get_target_category(&self) -> act::TargetCategory {
        act::TargetCategory::None
    }

    fn get_level(&self) -> i32 {
        0
    }

    fn get_identifier(&self) -> String {
        "pick up item".to_string()
    }

    fn get_energy_cost(&self) -> i32 {
        0
    }

    fn to_text(&self) -> String {
        "pick up item".to_string()
    }
}

/// Drop an item from the owner's inventory. The action level determines the item slot.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DropItem {
    lvl: i32,
}

impl DropItem {
    pub fn new(lvl: i32) -> Self {
        DropItem { lvl }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), typetag::serde)]
impl Action for DropItem {
    fn perform(
        &self,
        state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> act::ActionResult {
        // make sure there is an item at slot [self.lvl]
        if owner.inventory.items.len() > self.lvl as usize {
            let mut item: Object = owner.remove_from_inventory(self.lvl as usize);
            state.log.add(
                format!("{} dropped a {}", owner.visual.name, &item.visual.name),
                game::game_state::MsgClass::Info,
            );
            // set the item to be dropped at the same position as the player
            item.pos.set(owner.pos.x, owner.pos.y);
            objects.get_vector_mut().push(Some(item));

            // Remove this action from the inventory.
            owner.inventory.inv_actions.retain(|action| {
                action.get_identifier() != self.get_identifier()
                    || action.get_level() != self.get_level()
            });

            act::ActionResult::Success {
                callback: act::ObjectFeedback::NoFeedback,
            }
        } else {
            act::ActionResult::Failure
        }
    }

    fn set_target(&mut self, _t: act::Target) {}

    fn set_level(&mut self, lvl: i32) {
        self.lvl = lvl;
    }

    fn get_target_category(&self) -> act::TargetCategory {
        act::TargetCategory::None
    }

    fn get_level(&self) -> i32 {
        self.lvl
    }

    fn get_identifier(&self) -> String {
        "drop item".to_string()
    }

    fn get_energy_cost(&self) -> i32 {
        0
    }

    fn to_text(&self) -> String {
        "drop item".to_string()
    }
}
