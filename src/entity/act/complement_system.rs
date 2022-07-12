use serde::{Deserialize, Serialize};

use crate::entity::act::{Action, ActionResult, ObjectFeedback, Target, TargetCategory};
use crate::entity::Object;
use crate::game::{ObjectStore, State};

/// `Empty Tile`-action for updating the complement system depending on other present objects.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateComplementProteins;

#[cfg_attr(not(target_arch = "wasm32"), typetag::serde)]
impl Action for UpdateComplementProteins {
    fn perform(
        &self,
        _state: &mut State,
        objects: &mut ObjectStore,
        owner: &mut Object,
    ) -> ActionResult {
        objects.get_non_tiles().iter().flatten().for_each(|obj| // temporary criterion for now, need to detect membrane receptors instead
            if obj.pos == owner.pos && (obj.visual.name.to_lowercase().contains("virus") || obj.visual.name.to_lowercase().contains("bacteria")) {
                if let Some(tile) = &mut owner.tile {
                    tile.complement.cause_inflammation();
                }
            });

        ActionResult::Success {
            callback: ObjectFeedback::NoFeedback,
        }
    }

    fn set_target(&mut self, _target: Target) {}

    fn set_level(&mut self, _lvl: i32) {}

    fn get_target_category(&self) -> TargetCategory {
        TargetCategory::None
    }

    fn get_level(&self) -> i32 {
        0
    }

    fn get_identifier(&self) -> String {
        "update_complement_proteins".to_string()
    }

    fn get_energy_cost(&self) -> i32 {
        0
    }

    fn to_text(&self) -> String {
        "pass".to_string()
    }
}
