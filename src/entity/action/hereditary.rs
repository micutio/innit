//! This module contains all actions that can be acquired via genes.

use crate::core::game_objects::GameObjects;
use crate::core::game_state::{GameState, MessageLog, MsgClass, ObjectFeedback};
use crate::core::innit_env;
use crate::core::position::Position;
use crate::entity::action::{Action, ActionResult, Target, TargetCategory};
use crate::entity::ai::{AiForceVirusProduction, AiTile, AiVirus};
use crate::entity::control::Controller;
use crate::entity::genetics::DnaType;
use crate::entity::genetics::TraitFamily;
use crate::entity::object::Object;
use crate::entity::player::PlayerCtrl;
use crate::ui::particle::ParticleBuilder;
use crate::ui::{palette, register_particle, register_particles};
use serde::{Deserialize, Serialize};

/// Dummy action for passing the turn.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActPass {
    force_redraw: bool,
}

impl ActPass {
    pub fn update() -> Self {
        ActPass { force_redraw: true }
    }
}

impl Default for ActPass {
    fn default() -> Self {
        ActPass {
            force_redraw: false,
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), typetag::serde)]
impl Action for ActPass {
    fn perform(
        &self,
        _state: &mut GameState,
        _objects: &mut GameObjects,
        _owner: &mut Object,
    ) -> ActionResult {
        // play a little particle effect
        // let fg = palette().hud_fg_dna_sensor;
        // let bg = palette().world_bg_ground_fov_true;
        // Disable particle effect for now. It's a bit spammy.
        // if owner.physics.is_visible {
        //     register_particle(owner.pos.into(), fg, bg, 'Z', 250.0);
        // }

        let callback = if self.force_redraw {
            ObjectFeedback::Render
        } else {
            ObjectFeedback::NoFeedback
        };

        ActionResult::Success { callback }
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

#[cfg_attr(not(target_arch = "wasm32"), typetag::serde)]
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
            let callback = if owner.physics.is_visible {
                ObjectFeedback::Render
            } else {
                ObjectFeedback::NoFeedback
            };
            ActionResult::Success { callback }
        } else {
            // object cannot move because it's blocked
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

    fn get_level(&self) -> i32 {
        self.lvl
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
pub struct ActRepairStructure {
    lvl: i32,
}

impl ActRepairStructure {
    pub fn new() -> Self {
        ActRepairStructure { lvl: 0 }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), typetag::serde)]
impl Action for ActRepairStructure {
    fn perform(
        &self,
        _state: &mut GameState,
        _objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult {
        owner.actuators.hp = i32::min(owner.actuators.hp + (self.lvl * 2), owner.actuators.max_hp);
        if owner.physics.is_visible {
            register_particle(
                owner.pos,
                (50, 255, 50, 180),
                palette().col_transparent,
                owner.visual.glyph,
                450.0,
            )
        }
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

    fn get_level(&self) -> i32 {
        self.lvl
    }

    fn get_identifier(&self) -> String {
        "repair".to_string()
    }

    fn get_energy_cost(&self) -> i32 {
        self.lvl
    }

    fn to_text(&self) -> String {
        "repair cell structure".to_string()
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

#[cfg_attr(not(target_arch = "wasm32"), typetag::serde)]
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
        let valid_target: Option<&mut Object> = objects
            .get_vector_mut()
            .iter_mut()
            .flatten()
            .find(|o| o.physics.is_blocking && o.pos.is_equal(&target_pos));

        match valid_target {
            Some(t) => {
                // deal damage
                t.actuators.hp -= self.lvl;
                debug!("target hp: {}/{}", t.actuators.hp, t.actuators.max_hp);
                if owner.physics.is_visible {
                    state.log.add(
                        format!(
                            "{} attacked {} for {} damage",
                            &owner.visual.name, &t.visual.name, self.lvl
                        ),
                        MsgClass::Info,
                    );
                    // show particle effect
                    register_particle(
                        t.pos,
                        (200, 10, 10, 180),
                        palette().col_transparent,
                        'x',
                        250.0,
                    )
                }

                ActionResult::Success {
                    callback: ObjectFeedback::NoFeedback,
                }
            }
            None => {
                if owner.is_player() {
                    state.log.add("Nothing to attack here", MsgClass::Info);
                }
                ActionResult::Failure
            }
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

    fn get_level(&self) -> i32 {
        self.lvl
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

#[cfg_attr(not(target_arch = "wasm32"), typetag::serde)]
impl Action for ActInjectRnaVirus {
    // TODO: Find a way to get the position of this gene within the dna, to parse the complete
    //       virus dna
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
                let original_ai = target.control.take();
                let overriding_ai =
                    Controller::Npc(Box::new(AiForceVirusProduction::new_duration(
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

    fn get_level(&self) -> i32 {
        self.lvl
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

#[cfg_attr(not(target_arch = "wasm32"), typetag::serde)]
impl Action for ActInjectRetrovirus {
    // TODO: Allow for various levels of 'aggression', e.g.: forcing lysis, apoptosis or just
    //       cyclic activity
    fn perform(
        &self,
        state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult {
        let target_pos: Position = owner.pos.get_translated(&self.target.to_pos());
        if let Some((index, Some(mut target))) = objects.extract_entity_by_pos(&target_pos) {
            // check whether the virus can attach to the object
            // cell and not a plasmid or another virus
            // if yes, replace the control and force the cell to produce viruses

            if target.dna.dna_type == DnaType::Nucleus || target.dna.dna_type == DnaType::Nucleoid {
                // FAIL: target is not an actual cell, merely another virus or plasmid
                if owner.physics.is_visible {
                    state.log.add(
                        format!(
                            "A virus has tried to infect {} but it is not a cell!",
                            target.visual.name
                        ),
                        MsgClass::Info,
                    );
                    // play a little particle effect
                    let fg = palette().col_acc3;
                    let bg = palette().col_transparent;
                    register_particle(owner.pos.into(), fg, bg, '?', 150.0);
                }
            } else if owner.processors.receptors.is_empty() {
                // this virus must have receptors
                if owner.physics.is_visible {
                    state.log.add(
                        format!(
                            "A virus has tried to infect {} but cannot find matching receptor!",
                            target.visual.name
                        ),
                        MsgClass::Info,
                    );
                    // play a little particle effect
                    let fg = palette().col_acc3;
                    let bg = palette().col_transparent;
                    register_particle(owner.pos.into(), fg, bg, '?', 150.0);
                }
            } else if target
                .processors
                .receptors
                .iter()
                .any(|e1| owner.processors.receptors.iter().any(|e2| e1.typ == e2.typ))
            {
                // target and  owner must have matching receptor
                let mut new_dna = target.dna.raw.clone();
                new_dna.append(&mut owner.dna.raw.clone());
                let (s, p, a, d) = state
                    .gene_library
                    .dna_to_traits(target.dna.dna_type, new_dna.as_ref());
                target.set_genome(s, p, a, d);

                // The virus becomes an empty shell after successfully transmitting its dna.
                owner.dna.raw.clear();
                // The virus 'dies' symbolically...
                owner.alive = false;
                // ..because it's still debated as to whether viruses are alive to begin with.
                if owner.physics.is_visible {
                    state.log.add(
                        format!("A virus has infected {}!", target.visual.name),
                        MsgClass::Alert,
                    );
                    // play a little particle effect
                    let fg = palette().hud_fg_bar_health;
                    let bg = palette().col_transparent;
                    register_particle(owner.pos.into(), fg, bg, target.visual.glyph, 350.0);
                }
            }
            objects.replace(index, target);
        }

        ActionResult::Success {
            callback: ObjectFeedback::NoFeedback,
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

    fn get_level(&self) -> i32 {
        self.lvl
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

#[cfg_attr(not(target_arch = "wasm32"), typetag::serde)]
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
                assert!(!dna.is_empty());
                if owner.physics.is_visible || owner.is_player() {
                    state.log.add(
                        format!("{} is forced to produce virions", owner.visual.name),
                        MsgClass::Alert,
                    );
                }
                owner.inventory.items.push(
                    Object::new()
                        .position(owner.pos.x, owner.pos.y)
                        .living(true)
                        .visualize("virus", 'v', palette().entity_virus)
                        .physical(true, false, false)
                        .genome(0.75, state.gene_library.dna_to_traits(DnaType::Rna, dna))
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
                        let dna_from_seq =
                            state.gene_library.dna_from_traits(&owner.dna.simplified);
                        owner.inventory.items.push(
                            Object::new()
                                .position(owner.pos.x, owner.pos.y)
                                .living(true)
                                .visualize("virus", 'v', palette().entity_virus)
                                .physical(true, false, false)
                                .genome(
                                    0.75,
                                    state
                                        .gene_library
                                        .dna_to_traits(DnaType::Rna, &dna_from_seq),
                                )
                                .control(Controller::Npc(Box::new(AiVirus {}))),
                            // TODO: Separate Ai for retroviruses?
                        );
                    }
                }
            }
        };
        ActionResult::Success {
            callback: ObjectFeedback::Render,
        }
    }

    fn set_target(&mut self, _t: Target) {}

    fn set_level(&mut self, lvl: i32) {
        self.lvl = lvl;
    }

    fn get_level(&self) -> i32 {
        self.lvl
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActEditGenome {
    lvl: i32,
}

impl ActEditGenome {
    pub fn new() -> Self {
        ActEditGenome { lvl: 0 }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), typetag::serde)]
impl Action for ActEditGenome {
    fn perform(
        &self,
        _state: &mut GameState,
        _objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult {
        let callback = if owner.is_player() {
            ObjectFeedback::GenomeManipulator
        } else {
            ObjectFeedback::NoFeedback
        };
        ActionResult::Success { callback }
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
        "Manipulate Genome".to_string()
    }

    fn get_energy_cost(&self) -> i32 {
        self.lvl
    }

    fn to_text(&self) -> String {
        "Manipulate Genome".to_string()
    }
}

/// Ability for a cell to trigger its own killswitch. It can also trigger someone else's killswitch
/// provided that cell also has a killswitch and a matching receptor.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActKillSwitch {
    target: Target,
    lvl: i32,
}

impl ActKillSwitch {
    pub fn new() -> Self {
        ActKillSwitch {
            target: Target::Center,
            lvl: 0,
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), typetag::serde)]
impl Action for ActKillSwitch {
    fn perform(
        &self,
        state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult {
        match self.target {
            Target::Center => {
                owner.die(state, objects);
                let callback = if owner.physics.is_visible {
                    ObjectFeedback::Render
                } else {
                    ObjectFeedback::NoFeedback
                };
                // play a little particle effect
                if owner.physics.is_visible {
                    let fg = (255, 10, 90, 180);
                    let bg = palette().col_transparent;
                    register_particle(owner.pos.into(), fg, bg, '◘', 500.0);
                }
                ActionResult::Success { callback }
            }

            _ => {
                let t_pos: Position = owner.pos.get_translated(&self.target.to_pos());
                if let Some((index, Some(mut target))) = objects.extract_entity_by_pos(&t_pos) {
                    // kill switches of other cells can only be activated if they have both
                    // a) the killswitch gene
                    // b) a matching receptor that the owner can use to connect to the target
                    let has_killswitch = target
                        .dna
                        .simplified
                        .iter()
                        .any(|d| d.trait_name == "Kill Switch");
                    let has_matching_receptor = target
                        .processors
                        .receptors
                        .iter()
                        .any(|e1| owner.processors.receptors.iter().any(|e2| e1.typ == e2.typ));
                    if has_killswitch && has_matching_receptor {
                        target.die(state, objects);
                        if target.physics.is_visible {
                            let fg = (255, 10, 90, 180);
                            let bg = palette().col_transparent;
                            register_particle(target.pos.into(), fg, bg, '◘', 500.0);
                        }
                    }

                    let callback = if !target.alive && target.physics.is_visible {
                        ObjectFeedback::Render
                    } else {
                        ObjectFeedback::NoFeedback
                    };

                    objects.replace(index, target);

                    ActionResult::Success { callback }
                } else {
                    ActionResult::Failure
                }
            }
        }
    }

    fn set_target(&mut self, t: Target) {
        self.target = t;
    }

    fn set_level(&mut self, lvl: i32) {
        self.lvl = lvl;
    }

    fn get_target_category(&self) -> TargetCategory {
        TargetCategory::Any
    }

    fn get_level(&self) -> i32 {
        self.lvl
    }

    fn get_identifier(&self) -> String {
        "killswitch".to_string()
    }

    fn get_energy_cost(&self) -> i32 {
        self.lvl
    }

    fn to_text(&self) -> String {
        format!("killswitch {:?}", self.target)
    }
}

/// Reproduction of a cell by duplicating the genome and creating a quasi identical object, can be
/// affected by mutation. The target here can be any empty tile to place the child object.
/// In case of a tile cell replicating, it will replace the empty tile with a wall tile instead.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActBinaryFission {
    target: Target,
    lvl: i32,
}

impl ActBinaryFission {
    pub fn new() -> Self {
        ActBinaryFission {
            target: Target::Center,
            lvl: 0,
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), typetag::serde)]
impl Action for ActBinaryFission {
    fn perform(
        &self,
        state: &mut GameState,
        objects: &mut GameObjects,
        owner: &mut Object,
    ) -> ActionResult {
        // If the acting cell is a tile, turn a floor tile into a wall tile and insert a copy of
        // this one's (mutated) genome.
        let target_pos: Position = owner.pos.get_translated(&self.target.to_pos());
        let is_pos_available = !objects.is_pos_occupied(&target_pos);

        if is_pos_available {
            let child_obj = match objects.get_tile_at(target_pos.x, target_pos.y) {
                Some(t) => {
                    if owner.tile.is_some() && owner.physics.is_blocking {
                        if !t.physics.is_blocking {
                            // turn into wall
                            t.physics.is_blocking = true;
                            t.physics.is_blocking_sight = true;
                            t.visual.glyph = '◘';
                            t.visual.name = "wall tile".into();
                            t.control = Some(Controller::Npc(Box::new(AiTile)));
                            // insert (mutated) genome
                            t.set_dna(owner.dna.clone());
                            t.update_genome_from_dna(state);
                            t.processors.life_elapsed = 0;

                            // play a little particle effect
                            if t.physics.is_visible {
                                // cover up the new cell as long as the creation particles play
                                register_particle(
                                    owner.pos,
                                    (0, 0, 0, 0),
                                    (0, 0, 0, 0),
                                    ' ',
                                    600.0,
                                );
                                let fg = owner.visual.fg_color;
                                let bg = palette().col_transparent;
                                register_particles(
                                    ParticleBuilder::new(owner.pos.into(), fg, bg, '◘', 600.0)
                                        .with_moving_to(t.pos.into())
                                        .with_end_color((180, 255, 180, 180), bg),
                                )
                            }

                            // return prematurely because we don't need to insert anything new into
                            // the object vector
                            return ActionResult::Success {
                                callback: ObjectFeedback::NoFeedback,
                            };
                        } else {
                            None
                        }
                    } else {
                        // create a new object
                        let child_ctrl = match &owner.control {
                            Some(ctrl) => match ctrl {
                                Controller::Npc(ai) => Some(Controller::Npc(ai.clone())),
                                Controller::Player(_) => {
                                    Some(Controller::Player(PlayerCtrl::new()))
                                }
                            },
                            None => None,
                        };
                        let mut child = Object::new()
                            .position(t.pos.x, t.pos.y)
                            .living(true)
                            .visualize(
                                owner.visual.name.as_str(),
                                owner.visual.glyph,
                                owner.visual.fg_color,
                            )
                            .genome(
                                owner.gene_stability,
                                state
                                    .gene_library
                                    .dna_to_traits(owner.dna.dna_type, &owner.dna.raw),
                            )
                            .control_opt(child_ctrl)
                            .living(true);
                        child.physics.is_visible = t.physics.is_visible;
                        // play a little particle effect
                        if child.physics.is_visible {
                            let fg = (10, 255, 180, 180);
                            let bg = palette().col_transparent;
                            register_particle(child.pos.into(), fg, bg, '◘', 500.0);
                        }
                        Some(child)
                    }
                }
                None => None,
            };

            // finally place the 'child' cell into the world
            if let Some(child) = child_obj {
                let callback = if child.physics.is_visible && !innit_env().is_debug_mode {
                    ObjectFeedback::Render
                } else {
                    ObjectFeedback::NoFeedback
                };
                objects.push(child);

                ActionResult::Success { callback }
            } else {
                ActionResult::Failure
            }
        } else {
            ActionResult::Failure
        }
    }

    fn set_target(&mut self, t: Target) {
        self.target = t;
    }

    fn set_level(&mut self, lvl: i32) {
        self.lvl = lvl;
    }

    fn get_target_category(&self) -> TargetCategory {
        TargetCategory::EmptyObject
    }

    fn get_level(&self) -> i32 {
        self.lvl
    }

    fn get_identifier(&self) -> String {
        "bin. fission".to_string()
    }

    fn get_energy_cost(&self) -> i32 {
        self.lvl
    }

    fn to_text(&self) -> String {
        format!("binary fission into {:?}", self.target)
    }
}
