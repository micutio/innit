//! This module contains actions that are automatically available to all objects with an inventory.

use crate::{
    core::{
        game_objects::GameObjects,
        game_state::{GameState, MessageLog, MsgClass, ObjectFeedback},
    },
    entity::{
        action::{Action, ActionResult, Target, TargetCategory},
        object::Object,
    },
};
use serde::{Deserialize, Serialize};

/// Pick up an item and store it in the inventory.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActPickUpItem;

#[typetag::serde]
impl Action for ActPickUpItem {
    fn perform(
        &self,
        state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult {
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
                        MsgClass::Info,
                    );
                    owner.add_to_inventory(state, target_obj);

                    // keep the object vector neat and tidy
                    objects.get_vector_mut().remove(index);

                    return ActionResult::Success {
                        callback: ObjectFeedback::NoFeedback,
                    };
                } else {
                    state.log.add("Your inventory is full!", MsgClass::Info);
                }
            }
            //else {
            // otherwise put it back into the world
            //}
            objects.replace(index, target_obj);
            ActionResult::Failure
        } else {
            ActionResult::Failure
        }
    }

    fn set_target(&mut self, _t: Target) {}

    fn set_level(&mut self, _lvl: i32) {}

    fn get_target_category(&self) -> TargetCategory {
        TargetCategory::None
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
pub struct ActDropItem {
    lvl: i32,
}

impl ActDropItem {
    pub fn new(lvl: i32) -> Self {
        ActDropItem { lvl }
    }
}

#[typetag::serde]
impl Action for ActDropItem {
    fn perform(
        &self,
        state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult {
        // make sure there is an item at slot [self.lvl]
        if owner.inventory.items.len() > self.lvl as usize {
            let mut item: Object = owner.remove_from_inventory(state, self.lvl as usize);
            state.log.add(
                format!("{} dropped a {}", owner.visual.name, &item.visual.name),
                MsgClass::Info,
            );
            // set the item to be dropped at the same position as the player
            item.pos.set(owner.pos.x, owner.pos.y);
            objects.get_vector_mut().push(Some(item));

            // Remove this action from the inventory.
            owner.inventory.inv_actions.retain(|action| {
                action.get_identifier() != self.get_identifier()
                    || action.get_level() != self.get_level()
            });

            ActionResult::Success {
                callback: ObjectFeedback::NoFeedback,
            }
        } else {
            ActionResult::Failure
        }
    }

    fn set_target(&mut self, _t: Target) {}

    fn set_level(&mut self, lvl: i32) {
        self.lvl = lvl;
    }

    fn get_target_category(&self) -> TargetCategory {
        TargetCategory::None
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
