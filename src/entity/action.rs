//! Module Action provides the action interface, which is used to create any kind of action that
//! can be performed by the player or an NPC.
//! Any action is supposed to be assigned to one of the three trait families (sensing, prcessing,
//! actuating) of an object

use crate::core::game_objects::GameObjects;
use crate::core::game_state::{GameState, MessageLog, MsgClass, ObjectFeedback};
use crate::core::position::Position;
use crate::entity::ai::{AiForceVirusProduction, AiVirus};
use crate::entity::control::Controller;
use crate::entity::control::Controller::Npc;
use crate::entity::genetics::{DnaType, TraitFamily};
use crate::entity::object::Object;
use crate::ui::color::{self, Color};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

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
    Success { callback: ObjectFeedback },
    /// Failed to perform an action, ideally without any side effect.
    Failure,
    /// Another action happens automatically after this one.
    Consequence {
        callback: ObjectFeedback,
        follow_up: Box<dyn Action>,
    },
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
pub struct ActPass;

#[typetag::serde]
impl Action for ActPass {
    fn perform(
        &self,
        _state: &mut GameState,
        _objects: &mut GameObjects,
        _owner: &mut Object,
    ) -> ActionResult {
        // do nothing
        // duh
        ActionResult::Success {
            callback: ObjectFeedback::NoFeedback,
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
pub struct ActMove {
    lvl: i32,
    direction: Target,
}

impl ActMove {
    // TODO: use level
    pub fn new() -> Self {
        ActMove {
            lvl: 0,
            direction: Target::Center,
        }
    }
}

#[typetag::serde]
impl Action for ActMove {
    fn perform(
        &self,
        _state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult {
        let target_pos = owner.pos.get_translated(&self.direction.to_pos());
        if owner.physics.is_visible {
            debug!(
                "target position {:#?}, blocked: {}",
                target_pos,
                &objects.is_pos_blocked(&target_pos)
            );
        }
        if !&objects.is_pos_blocked(&target_pos) {
            owner.pos.set(target_pos.x, target_pos.y);
            ActionResult::Success {
                callback: ObjectFeedback::Render,
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
pub struct ActMetabolise {
    lvl: i32,
}

impl ActMetabolise {
    pub fn new() -> Self {
        ActMetabolise { lvl: 0 }
    }
}

#[typetag::serde]
impl Action for ActMetabolise {
    fn perform(
        &self,
        _state: &mut GameState,
        _objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult {
        owner.processors.energy += self.lvl;
        ActionResult::Success {
            callback: ObjectFeedback::NoFeedback,
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
pub struct ActAttack {
    lvl: i32,
    target: Target,
}

impl ActAttack {
    pub fn new() -> Self {
        ActAttack {
            lvl: 0,
            target: Target::Center,
        }
    }
}

#[typetag::serde]
impl Action for ActAttack {
    fn perform(
        &self,
        state: &mut GameState,
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

        if let Some(_target_obj) = valid_targets.first() {
            // TODO: Take damage
            ActionResult::Success {
                callback: ObjectFeedback::NoFeedback,
            }
        } else {
            state.log.add("Nothing to attack here", MsgClass::Info);
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
pub struct ActInjectRnaVirus {
    lvl: i32,
    target: Target,
    rna: Vec<u8>,
}

impl ActInjectRnaVirus {
    pub fn new(target: Target, dna: Vec<u8>) -> Self {
        ActInjectRnaVirus {
            lvl: 0,
            target,
            rna: dna,
        }
    }
}

#[typetag::serde]
impl Action for ActInjectRnaVirus {
    // TODO: Find a way to get the position of this gene within the dna, to parse the complete
    // virus dna
    fn perform(
        &self,
        state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult {
        let target_pos: Position = owner.pos.get_translated(&self.target.to_pos());
        if let Some((index, Some(mut target))) = objects.extract_entity_by_pos(&target_pos) {
            // check whether the virus can attach to the object and whether the object is an actual
            // cell and not a plasmid or another virus
            // if yes, replace the control and force the cell to produce viruses
            let has_infected = if target
                .processors
                .receptors
                .iter()
                .any(|e| owner.processors.receptors.contains(e))
                && (target.dna.dna_type == DnaType::Nucleus
                    || target.dna.dna_type == DnaType::Nucleoid)
            {
                if target.is_player()
                    || target.physics.is_visible
                    || owner.is_player()
                    || owner.physics.is_visible
                {
                    state.log.add(
                        format!(
                            "{0} has infected {1} with virus RNA. {1} is forced to produce virions",
                            owner.visual.name, target.visual.name
                        ),
                        MsgClass::Alert,
                    );
                }
                // TODO: Add forced production for retroviruses
                let original_ai = target.control.take();
                let overriding_ai = Npc(Box::new(AiForceVirusProduction::new_duration(
                    original_ai,
                    4,
                    Some(owner.dna.raw.clone()),
                )));
                target.control.replace(overriding_ai);

                // The virus becomes an empty shell after successfully transmitting its dna.
                owner.dna.raw.clear();
                // The virus 'dies' symbolically.
                owner.alive = false;
                // Funny, because it's still debated as to whether viruses are alive to begin.
                // TODO: Handle other death effects, such as change of blocking, symbol and color.

                true
            } else {
                false
            };

            // copy target name before moving target back into object vector
            let target_name = target.visual.name.clone();
            objects.replace(index, target);
            if has_infected {
                if owner.physics.is_visible || owner.is_player() {
                    state.log.add(
                        format!(
                            "{} injected virus RNA into {}",
                            owner.visual.name, target_name
                        ),
                        MsgClass::Alert,
                    );
                    debug!(
                        "{} injected virus RNA into {}",
                        owner.visual.name, target_name
                    );
                }
                return ActionResult::Success {
                    callback: ObjectFeedback::NoFeedback,
                };
            }
            ActionResult::Success {
                callback: ObjectFeedback::NoFeedback,
            }
        } else {
            ActionResult::Success {
                callback: ObjectFeedback::NoFeedback,
            }
        }
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
pub struct ActInjectRetrovirus {
    lvl: i32,
    target: Target,
}

impl ActInjectRetrovirus {
    pub fn _new() -> Self {
        ActInjectRetrovirus {
            lvl: 0,
            target: Target::Center,
        }
    }
}

#[typetag::serde]
impl Action for ActInjectRetrovirus {
    // TODO: Find a way to get the position of this gene within the dna, to parse the complete
    //       virus dna.
    // TODO: Allow for various levels of 'aggression', e.g.: forcing lysis, apoptosis or just
    //       cyclic activity
    fn perform(
        &self,
        state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult {
        let target_pos: Position = owner.pos.get_translated(&self.target.to_pos());
        let feedback: ObjectFeedback =
            if let Some((index, Some(mut target))) = objects.extract_entity_by_pos(&target_pos) {
                // check whether the virus can attach to the object and whether the object is an actual
                // cell and not a plasmid or another virus
                // if yes, replace the control and force the cell to produce viruses
                let msg_feedback = if target
                    .processors
                    .receptors
                    .iter()
                    .any(|e| owner.processors.receptors.contains(e))
                    && (target.dna.dna_type != DnaType::Nucleus
                        || target.dna.dna_type == DnaType::Nucleoid)
                {
                    let mut new_dna = target.dna.raw.clone();
                    new_dna.append(&mut owner.dna.raw.clone());
                    let (s, p, a, d) = state
                        .gene_library
                        .decode_dna(target.dna.dna_type, new_dna.as_ref());
                    target.change_genome(s, p, a, d);

                    // The virus becomes an empty shell after successfully transmitting its dna.
                    owner.dna.raw.clear();
                    // The virus 'dies' symbolically.
                    owner.alive = false;
                    // Funny, because it's still debated as to whether viruses are alive to begin.
                    // TODO: Handle other death effects, such as change of blocking, symbol and color.

                    if owner.physics.is_visible {
                        state.log.add(
                            format!("A virus has infected {}!", target.visual.name),
                            MsgClass::Alert,
                        )
                    }
                    ObjectFeedback::NoFeedback
                } else {
                    if owner.physics.is_visible {
                        state.log.add(
                            format!(
                                "A virus has tried to infect {} but cannot find matching receptor!",
                                target.visual.name
                            ),
                            MsgClass::Alert,
                        )
                    }
                    ObjectFeedback::NoFeedback
                };
                objects.replace(index, target);
                msg_feedback
            } else {
                ObjectFeedback::NoFeedback
            };

        ActionResult::Success { callback: feedback }
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
/// Retroviruses convert their RNA into DNA and inject it into the cell for reproduction as well
/// as into the cell's DNA where it can permanently reside and switch between dormant and active.
///
/// RNA viruses will set the field `virus_rna`, from which viruses will be replicated.
/// If `virus_rna` is `None`, the object will look for a retrovirus sequence within its own dna to
/// use to initialise the new virion.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActProduceVirion {
    lvl: i32,
    virus_rna: Option<Vec<u8>>,
}

impl ActProduceVirion {
    pub fn new(virus_rna: Option<Vec<u8>>) -> Self {
        ActProduceVirion { lvl: 0, virus_rna }
    }
}

#[typetag::serde]
impl Action for ActProduceVirion {
    fn perform(
        &self,
        state: &mut GameState,
        _objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult {
        match &self.virus_rna {
            Some(dna) => {
                debug!("#{} produces virion", owner.visual.name);
                // if owner.physics.is_visible || owner.is_player() {
                state.log.add(
                    format!("{} is forced to produce virions", owner.visual.name),
                    MsgClass::Alert,
                );
                // }
                owner.inventory.items.push(
                    Object::new()
                        .position(owner.pos.x, owner.pos.y)
                        .living(true)
                        .visualize("virus", 'v', Color::from(color::VIRUS))
                        .physical(true, false, false)
                        .genome(0.75, state.gene_library.decode_dna(DnaType::Rna, dna))
                        .control(Controller::Npc(Box::new(AiVirus {}))),
                );
            }
            None => {
                // look for virus dna flanked by LTR markers in our own dna
                let p0 = owner
                    .dna
                    .simplified
                    .iter()
                    .position(|x| x.trait_family == TraitFamily::Ltr);
                let p1 = owner
                    .dna
                    .simplified
                    .iter()
                    .rposition(|x| x.trait_family == TraitFamily::Ltr);
                if let (Some(a), Some(b)) = (p0, p1) {
                    if a != b {
                        let dna_sequence: Vec<String> = owner.dna.simplified[a..=b]
                            .iter()
                            .map(|x| x.trait_name.clone())
                            .collect();
                        let dna_from_seq = state.gene_library.dna_from_traits(&dna_sequence);
                        owner.inventory.items.push(
                            Object::new()
                                .position(owner.pos.x, owner.pos.y)
                                .living(true)
                                .visualize("virus", 'v', Color::from(color::VIRUS))
                                .physical(true, false, false)
                                .genome(
                                    0.75,
                                    state.gene_library.decode_dna(DnaType::Rna, &dna_from_seq),
                                )
                                .control(Controller::Npc(Box::new(AiVirus {}))), // TODO: Separate Ai for retroviruses?
                        );
                    }
                }
            }
        };
        ActionResult::Success {
            callback: ObjectFeedback::NoFeedback,
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
        "produce virus".to_string()
    }

    fn get_energy_cost(&self) -> i32 {
        self.lvl
    }

    fn to_text(&self) -> String {
        "produces virus".to_string()
    }
}
