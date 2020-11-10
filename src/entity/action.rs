//! Module Action provides the action interface, which is used to create any kind of action that
//! can be performed by the player or an NPC.
//! Any action is supposed to be assigned to one of the three trait families (sensing, prcessing,
//! actuating) of an object

use tcod::colors;

use std::fmt::Debug;

use crate::core::game_objects::GameObjects;
use crate::core::game_state::{GameState, MessageLog, ObjectProcResult, MsgClass};
use crate::core::position::Position;
use crate::entity::object::Object;
use crate::entity::ai::ForceVirusProduction;
use crate::entity::control::Controller::Npc;

/// Possible target groups are: objects, empty space, anything or self (None).
/// Non-targeted actions will always be applied to the performing object itself.
#[derive(Clone, Debug, PartialEq)]
pub enum TargetCategory {
    Any,
    BlockingObject,
    EmptyObject,
    None,
}

/// Targets can only be adjacent to the object: north, south, east, west or the objects itself.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum Target {
    North,
    South,
    East,
    West,
    Center,
}

impl Target {
    fn to_pos(&self) -> Position {
        match self {
            Target::North => Position::new(0, -1),
            Target::South => Position::new(0, 1),
            Target::East => Position::new(1, 0),
            Target::West => Position::new(-1, 0),
            Target::Center => Position::new(0, 0),
        }
    }

    /// Returns the target direction from acting position p1 to targeted position p2.
    pub fn from_pos(p1: &Position, p2: &Position) -> Target {
        match p1.offset(p2) {
            (0, -1) => Target::North,
            (0, 1) => Target::South,
            (1, 0) => Target::East,
            (-1, 0) => Target::West,
            (0, 0) => Target::Center,
            _ => panic!("calling from_xy on non-adjacent target"),
        }
    }
}

/// Result of performing an action.
/// It can succeed, fail and cause direct consequences.
pub enum ActionResult {
    /// Successfully finished action
    Success { callback: ObjectProcResult },
    /// Failed to perform an action, ideally without any side effect.
    Failure,
    /// Another action happens automatically after this one.
    Consequence { action: Option<Box<dyn Action>> },
}

/// Interface for all actions.
/// They need to be `performable` and have a cost (even if it's 0).
#[typetag::serde(tag = "type")]
pub trait Action: ActionClone + Debug {
    fn perform(
        &self,
        state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult;

    fn set_target(&mut self, t: Target);

    fn set_level(&mut self, lvl: i32);

    fn get_target_category(&self) -> TargetCategory;

    fn get_identifier(&self) -> String;

    fn get_energy_cost(&self) -> i32;

    fn to_text(&self) -> String;
}

pub trait ActionClone {
    fn clone_action(&self) -> Box<dyn Action>;
}

impl<T> ActionClone for T
where
    T: Action + Clone + 'static,
{
    fn clone_action(&self) -> Box<dyn Action> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Action> {
    fn clone(&self) -> Self {
        self.clone_action()
    }
}

/// Dummy action for passing the turn.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pass;

#[typetag::serde]
impl Action for Pass {
    fn perform(
        &self,
        _state: &mut GameState,
        _objects: &mut GameObjects,
        _owner: &mut Object,
    ) -> ActionResult {
        // do nothing
        // duh
        ActionResult::Success {
            callback: ObjectProcResult::NoFeedback,
        }
    }

    fn set_target(&mut self, _target: Target) {}

    fn set_level(&mut self, _lvl: i32) {}

    fn get_target_category(&self) -> TargetCategory {
        TargetCategory::None
    }

    fn get_identifier(&self) -> String {
        "pass".to_string()
    }

    fn get_energy_cost(&self) -> i32 {
        0
    }

    fn to_text(&self) -> String {
        "pass".to_string()
    }
}

/// Move an object
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Move {
    lvl: i32,
    direction: Target,
}

impl Move {
    // TODO: use level
    pub fn new() -> Self {
        Move {
            lvl: 0,
            direction: Target::Center,
        }
    }
}

#[typetag::serde]
impl Action for Move {
    fn perform(
        &self,
        _state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult {
        let target_pos = owner.pos.get_translated(&self.direction.to_pos());
        if !&objects.is_pos_blocked(&target_pos) {
            owner.pos.set(target_pos.x, target_pos.y);
            ActionResult::Success {
                callback: ObjectProcResult::CheckEnterFOV,
            }
        } else {
            info!("object {} blocked!", owner.visual.name);
            ActionResult::Failure // this might cause infinite loops of failure
        }
    }

    fn set_target(&mut self, target: Target) {
        self.direction = target;
    }

    fn set_level(&mut self, lvl: i32) {
        self.lvl = lvl;
    }

    fn get_target_category(&self) -> TargetCategory {
        TargetCategory::EmptyObject
    }

    fn get_identifier(&self) -> String {
        "move".to_string()
    }

    fn get_energy_cost(&self) -> i32 {
        self.lvl
    }

    fn to_text(&self) -> String {
        format!("move to {:?}", self.direction)
    }
}

/// Focus on increased energy production for this turn.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metabolise {
    lvl: i32,
}

impl Metabolise {
    pub fn new() -> Self {
        Metabolise { lvl: 0 }
    }
}

#[typetag::serde]
impl Action for Metabolise {
    fn perform(
        &self,
        _state: &mut GameState,
        _objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult {
        owner.processors.energy += self.lvl;
        ActionResult::Success {
            callback: ObjectProcResult::NoFeedback,
        }
    }

    fn set_target(&mut self, _t: Target) {}

    fn set_level(&mut self, lvl: i32) {
        self.lvl = lvl;
    }

    fn get_target_category(&self) -> TargetCategory {
        TargetCategory::None
    }

    fn get_identifier(&self) -> String {
        "metabolize".to_string()
    }

    fn get_energy_cost(&self) -> i32 {
        0
    }

    fn to_text(&self) -> String {
        "increase metabolism momentarily".to_string()
    }
}

/// Attack another object.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attack {
    lvl: i32,
    target: Target,
}

impl Attack {
    pub fn new() -> Self {
        Attack {
            lvl: 0,
            target: Target::Center,
        }
    }
}

#[typetag::serde]
impl Action for Attack {
    fn perform(
        &self,
        _state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult {
        // get coords of self position plus direction
        // find any objects that are at that position and blocking
        // assert that there is only one available
        // return
        let target_pos: Position = owner.pos.get_translated(&self.target.to_pos());
        let valid_targets: Vec<&Object> = objects
            .get_vector()
            .iter()
            .flatten()
            .filter(|o| o.physics.is_blocking && o.pos.is_equal(&target_pos))
            .collect();

        assert!(valid_targets.len() >= 1);
        if let Some(_target_obj) = valid_targets.first() {
            // TODO: Take damage
            ActionResult::Success {
                callback: ObjectProcResult::CheckEnterFOV,
            }
        } else {
            ActionResult::Failure
        }
    }

    fn set_target(&mut self, target: Target) {
        self.target = target;
    }

    fn set_level(&mut self, lvl: i32) {
        self.lvl = lvl;
    }

    fn get_target_category(&self) -> TargetCategory {
        TargetCategory::BlockingObject
    }

    fn get_identifier(&self) -> String {
        "attack".to_string()
    }

    fn get_energy_cost(&self) -> i32 {
        self.lvl
    }

    fn to_text(&self) -> String {
        format!("attack {:?}", self.target)
    }
}

// TODO: Add actions for
// - attaching to another cell
// - inserting genome into another cell
// - immobilising and manipulating another cell
// - producing stuff

/// A virus' sole purpose is to go forth and multiply.
/// This action corresponds to the virus trait which is located at the beginning of virus DNA.
/// RNA viruses inject their RNA into a host cell and force them to replicate the virus WITHOUT
/// permanently changing the cell's DNA.
/// #[derive(Debug, Serialize, Deserialize, Clone)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InjectVirus {
    lvl: i32,
    target: Target,
}

impl InjectVirus {
    pub fn new(target: Target) -> Self {
        InjectVirus { lvl: 0, target }
    }
}

#[typetag::serde]
impl Action for InjectVirus {
    // TODO: Find a way to get the position of this gene within the dna, to parse the complete
    // virus dna
    fn perform(
        &self,
        _state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult {
        let target_pos: Position = owner.pos.get_translated(&self.target.to_pos());
        if let Some((index, Some(mut target))) = objects.extract_entity_w_index(&target_pos) {
            // check whether the virus can attach to the cell
            // if yes, replace the control and force the cell to produce viruses
            let has_infected = if target
                .processors
                .receptors
                .iter()
                .any(|e| owner.processors.receptors.contains(e)) {
                let original_ai = target.control.take();
                target.control.replace(Npc(Box::new(ForceVirusProduction::new_duration(original_ai, 4))));
                true
            } else {
                false
            };

            // copy target name before moving target back into object vector
            let target_name = target.visual.name.clone();
            objects.replace(index, target);
            if has_infected {
                return ActionResult::Success {callback: ObjectProcResult::Message {msg: format!("{} injected a virus into {}", owner.visual.name, target_name), class: MsgClass::Alert, origin: owner.pos.clone()}};
            }
            ActionResult::Success {callback: ObjectProcResult::NoFeedback}

        } else {
            ActionResult::Success {callback: ObjectProcResult::NoFeedback}        }
    }

    /// NOP, because this action can only be self-targeted.
    fn set_target(&mut self, _t: Target) {}

    fn set_level(&mut self, lvl: i32) {
        self.lvl = lvl;
    }

    fn get_target_category(&self) -> TargetCategory {
        TargetCategory::None
    }

    fn get_identifier(&self) -> String {
        "inject RNA virus".to_string()
    }

    fn get_energy_cost(&self) -> i32 {
        self.lvl
    }

    fn to_text(&self) -> String {
        "inject RNA virus".to_string()
    }
}

/// A virus' sole purpose is to go forth and multiply.
/// This action corresponds to the virus trait which is located at the beginning of virus DNA.
/// Retro viruses convert their RNA into DNA and inject it into the cell for reproduction as well
/// as into the cell's DNA where it can permanently reside and switch between dormant and active.
/// #[derive(Debug, Serialize, Deserialize, Clone)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InjectRetrovirus {
    lvl: i32,
    target: Target,
}

impl InjectRetrovirus {
    pub fn new() -> Self {
        InjectRetrovirus {
            lvl: 0,
            target: Target::Center,
        }
    }
}

#[typetag::serde]
impl Action for InjectRetrovirus {
    // TODO: Find a way to get the position of this gene within the dna, to parse the complete
    //       virus dna.
    fn perform(
        &self,
        state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult {
        let target_pos: Position = owner.pos.get_translated(&self.target.to_pos());
        let feedback= if let Some((index, Some(mut target))) = objects.extract_entity_w_index(&target_pos) {
            // check whether the virus can attach to the cell
            let msg_feedback = if target
                .processors
                .receptors
                .iter()
                .any(|e| owner.processors.receptors.contains(e))
            {
                let mut new_dna = target.dna.raw.clone();
                new_dna.append(&mut owner.dna.raw.clone());
                let (s, p, a, d) = state
                    .gene_library
                    .decode_dna(target.dna.dna_type, new_dna.as_ref());
                target.change_genome(s, p, a, d);

                ObjectProcResult::Message {
                    msg: format!("A virus has infected {}!", target.visual.name),
                    class: MsgClass::Alert,
                    origin: owner.pos.clone(),
                }
            } else {
                ObjectProcResult::Message {
                    msg: format!(
                        "A virus has tried to infect {} but cannot find matching receptor!",
                        target.visual.name
                    ),
                    class: MsgClass::Info,
                    origin: owner.pos.clone(),
                }
            };
            objects.replace(index, target);
            msg_feedback
        } else {
            ObjectProcResult::NoFeedback
        };

        ActionResult::Success {
            callback: feedback,
        }
    }

    /// NOP, because this action can only be self-targeted.
    fn set_target(&mut self, t: Target) {
        self.target = t
    }

    fn set_level(&mut self, lvl: i32) {
        self.lvl = lvl;
    }

    fn get_target_category(&self) -> TargetCategory {
        TargetCategory::None
    }

    fn get_identifier(&self) -> String {
        "inject retrovirus".to_string()
    }

    fn get_energy_cost(&self) -> i32 {
        self.lvl
    }

    fn to_text(&self) -> String {
        "inject retrovirus".to_string()
    }
}

/// A virus' sole purpose is to go forth and multiply.
/// This action corresponds to the virus trait which is located at the beginning of virus DNA.
/// Retro viruses convert their RNA into DNA and inject it into the cell for reproduction as well
/// as into the cell's DNA where it can permanently reside and switch between dormant and active.
/// #[derive(Debug, Serialize, Deserialize, Clone)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProduceVirus {
    lvl: i32,
}

impl ProduceVirus {
    pub fn new() -> Self {
        ProduceVirus { lvl: 0 }
    }
}

#[typetag::serde]
impl Action for ProduceVirus {
    fn perform(
        &self,
        _state: &mut GameState,
        _objects: &mut GameObjects,
        _owner: &mut Object,
    ) -> ActionResult {
        unimplemented!()
    }

    fn set_target(&mut self, _t: Target) {}

    fn set_level(&mut self, lvl: i32) {
        self.lvl = lvl;
    }

    fn get_target_category(&self) -> TargetCategory {
        TargetCategory::None
    }

    fn get_identifier(&self) -> String {
        "produce virus".to_string()
    }

    fn get_energy_cost(&self) -> i32 {
        self.lvl
    }

    fn to_text(&self) -> String {
        "produces virus".to_string()
    }
}
