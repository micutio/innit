use tcod::colors::Color;

use crate::core::game_objects::GameObjects;
use crate::core::game_state::{GameState, MessageLog, Messages, MsgClass};
use crate::core::position::Position;
use crate::core::world::world_gen::Tile;
use crate::entity::action::*;
use crate::entity::control::*;
use crate::entity::genetics::{Actuators, Dna, DnaType, Processors, Sensors};
use crate::entity::inventory::Inventory;

use std::cmp::min;
use std::fmt;

/// An Object represents the base structure for all entities in the game.
/// Most of the object components are organized in their own
///
/// ```Option<ComponentType>```
///
/// fields.
/// The mandatory components _visual_ and _physics_ are relevant to the UI and game core. On the
/// other hand, nearly all optional components are determined by the object's genome, except
/// _next_action_.
///
/// DNA related fields are going to be _sensor_, _processor_ and _actuator_. These contain
/// attributes pertaining to their specific domain as well as performable actions which are
/// influenced or amplified by certain attributes.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Object {
    // TODO: Add plasmids, antigen-markers
    pub alive: bool,
    pub gene_stability: f64,
    pub pos: Position,
    pub visual: Visual,
    pub physics: Physics,
    pub tile: Option<Tile>,
    pub control: Option<Controller>,
    pub dna: Dna,
    pub sensors: Sensors,
    pub processors: Processors,
    pub actuators: Actuators,
    pub inventory: Inventory,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Visual {
    pub name: String,
    pub character: char,
    pub color: Color,
}

impl Visual {
    pub fn new() -> Self {
        Visual {
            name: "unknown".into(),
            character: '_',
            color: Color {
                r: 255,
                g: 255,
                b: 255,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Physics {
    pub is_blocking: bool,
    pub is_blocking_sight: bool,
    pub is_always_visible: bool,
}

impl Physics {
    pub fn new() -> Self {
        Physics {
            is_blocking: false,
            is_blocking_sight: false,
            is_always_visible: false,
        }
    }
}

impl Object {
    /// The Object constructor uses the builder pattern.
    pub fn new() -> Self {
        Object {
            pos: Position::new(0, 0),
            alive: false,
            gene_stability: 1.0,
            tile: None,
            control: None,
            dna: Dna::new(),
            visual: Visual::new(),
            physics: Physics::new(),
            sensors: Sensors::new(),
            processors: Processors::new(),
            actuators: Actuators::new(),
            inventory: Inventory::new(),
        }
    }

    /// Set the object's position in the world. Part of the builder pattern.
    pub fn position(mut self, x: i32, y: i32) -> Object {
        self.pos = Position::new(x, y);
        self
    }

    /// Set whether this object is alive (true) or dead (false). Part of the builder pattern.
    pub fn living(mut self, alive: bool) -> Object {
        self.alive = alive;
        self
    }

    /// Initialize the visual properties of the object. Part of the builder pattern.
    pub fn visualize(mut self, name: &str, character: char, color: Color) -> Object {
        self.visual.name = name.into();
        self.visual.character = character;
        self.visual.color = color;
        self
    }

    /// Initialize the physical properties of the object. Part of the builder pattern.
    pub fn physical(
        mut self,
        is_blocking: bool,
        is_blocking_sight: bool,
        is_always_visible: bool,
    ) -> Object {
        self.physics.is_blocking = is_blocking;
        self.physics.is_blocking_sight = is_blocking_sight;
        self.physics.is_always_visible = is_always_visible;
        self
    }

    /// Set the object's dna and super traits. Part of the builder pattern.
    pub fn genome(
        mut self,
        stability: f64,
        (sensors, processors, actuators, dna): (Sensors, Processors, Actuators, Dna),
    ) -> Object {
        self.gene_stability = stability;
        self.change_genome(sensors, processors, actuators, dna);

        // debug!("default action: {:#?}", self.default_action);
        self
    }

    /// Transform the object into a tile. Part of the builder pattern.
    pub fn tile_explored(mut self, is_explored: bool) -> Object {
        self.tile = Some(Tile { is_explored });
        self
    }

    /// Transform the object into an NPC or player. Part of the builder pattern.
    pub fn control(mut self, controller: Controller) -> Object {
        self.control = Some(controller);
        self
    }

    /// Perform necessary actions when object dies.
    pub fn die(&mut self, _state: &mut GameState, objects: &mut GameObjects) {
        self.alive = false;
        // empty inventory into this objects' current position
        for mut o in self.inventory.items.drain(..) {
            o.pos.translate(&self.pos);
            objects.push(o);
        }
        // take this object out of the world
        if self.is_player() {
            self.visual.name = "your remains".to_string();
        }
    }

    pub fn is_player(&self) -> bool {
        if let Some(Controller::Player(_)) = self.control {
            true
        } else {
            false
        }
    }

    /// Transform the object into an NPC or player. Part of the builder pattern.
    pub fn set_control(mut self, controller: Controller, log: &mut Messages) {
        match controller {
            Controller::Npc(_) => {
                if self.is_player() {
                    log.add(
                        format!("You lost control over {}", &self.visual.name),
                        MsgClass::Alert,
                    );
                }
            }
            Controller::Player(_) => {
                if let Some(Controller::Npc(_)) = self.control {
                    log.add(
                        format!("You gained control over {}", &self.visual.name),
                        MsgClass::Alert,
                    );
                }
            }
        }
        self.control = Some(controller);
    }

    pub fn metabolize(&mut self) {
        self.processors.energy = min(
            self.processors.energy + self.processors.metabolism,
            self.processors.energy_storage,
        )
    }

    /// Set the object's current dna and resulting super traits.
    pub fn change_genome(
        &mut self,
        sensors: Sensors,
        processors: Processors,
        actuators: Actuators,
        dna: Dna,
    ) {
        self.sensors = sensors;
        self.processors = processors;
        self.actuators = actuators;
        self.dna = dna;

        // update default action
        if let Some(Controller::Player(ref mut ctrl)) = &mut self.control {
            if let Some(def_action) = self
                .actuators
                .actions
                .iter()
                .find(|a| a.as_ref().get_identifier() == "move")
            {
                ctrl.primary_action = def_action.clone_action();
                debug!(
                    "{} new default action: {:#?}",
                    self.visual.name, ctrl.primary_action
                );
            }
        }
    }

    /// Determine and return the next action the object will take.
    pub fn extract_next_action(
        &mut self,
        state: &mut GameState,
        objects: &mut GameObjects,
    ) -> Option<Box<dyn Action>> {
        // Check if this object is ai controlled, and if so, take the ai out of the object before processing.
        let mut controller = self.control.take();
        let next_action;
        match controller {
            Some(Controller::Npc(ref mut boxed_ai)) => {
                next_action = Some(boxed_ai.act(state, objects, self));
            }
            Some(Controller::Player(ref mut player_ctrl)) => {
                next_action = player_ctrl.next_action.take();
            }
            None => next_action = None,
        }
        self.control = controller;
        next_action
    }

    /// Inject the next action this object will take into the object.
    pub fn set_next_action(&mut self, next_action: Option<Box<dyn Action>>) {
        let mut controller = self.control.take();
        if let Some(Controller::Player(ref mut ctrl)) = controller {
            ctrl.next_action = next_action;
        }
        self.control = controller;
    }

    pub fn set_primary_action(&mut self, new_primary_action: Box<dyn Action>) {
        let mut controller = self.control.take();
        if let Some(Controller::Player(ref mut ctrl)) = controller {
            ctrl.primary_action = new_primary_action;
        }
        self.control = controller;
    }

    pub fn set_secondary_action(&mut self, new_secondary_action: Box<dyn Action>) {
        let mut controller = self.control.take();
        if let Some(Controller::Player(ref mut ctrl)) = controller {
            ctrl.secondary_action = new_secondary_action;
        }
        self.control = controller;
    }

    pub fn set_quick1_action(&mut self, new_quick1_action: Box<dyn Action>) {
        let mut controller = self.control.take();
        if let Some(Controller::Player(ref mut ctrl)) = controller {
            ctrl.quick1_action = new_quick1_action;
        }
        self.control = controller;
    }

    pub fn set_quick2_action(&mut self, new_quick2_action: Box<dyn Action>) {
        let mut controller = self.control.take();
        if let Some(Controller::Player(ref mut ctrl)) = controller {
            ctrl.quick2_action = new_quick2_action;
        }
        self.control = controller;
    }

    pub fn has_next_action(&self) -> bool {
        if let Some(Controller::Player(ctrl)) = &self.control {
            ctrl.next_action.is_some()
        } else {
            false
        }
    }

    // TODO: Consider moving the player-action-related methods into PlayerCtrl.

    pub fn get_primary_action(&self, target: Target) -> Box<dyn Action> {
        // Some(def_action.clone())
        if let Some(Controller::Player(ctrl)) = &self.control {
            let mut action_clone = ctrl.primary_action.clone();
            action_clone.set_target(target);
            action_clone
        } else {
            Box::new(Pass)
        }
    }

    pub fn get_secondary_action(&self, target: Target) -> Box<dyn Action> {
        // Some(def_action.clone())
        if let Some(Controller::Player(ctrl)) = &self.control {
            let mut action_clone = ctrl.secondary_action.clone();
            action_clone.set_target(target);
            action_clone
        } else {
            Box::new(Pass)
        }
    }

    pub fn get_quick1_action(&self) -> Box<dyn Action> {
        if let Some(Controller::Player(ctrl)) = &self.control {
            ctrl.quick1_action.clone()
        } else {
            Box::new(Pass)
        }
    }

    pub fn get_quick2_action(&self) -> Box<dyn Action> {
        if let Some(Controller::Player(ctrl)) = &self.control {
            ctrl.quick2_action.clone()
        } else {
            Box::new(Pass)
        }
    }

    // TODO: Take plasmids into account!
    pub fn get_all_actions(&self) -> Vec<&Box<dyn Action>> {
        self.actuators
            .actions
            .iter()
            .chain(self.processors.actions.iter())
            .chain(self.sensors.actions.iter())
            .collect()
    }

    pub fn add_to_inventory(&mut self, state: &mut GameState, o: Object) {
        let reread_dna = o.dna.dna_type == DnaType::Plasmid;
        self.inventory.items.push(o);
        if reread_dna {
            self.reread_dna(state);
        }
    }

    pub fn remove_from_inventory(&mut self, state: &mut GameState, index: usize) {
        let o = self.inventory.items.remove(index);
        if o.dna.dna_type == DnaType::Plasmid {
            self.reread_dna(state);
        }
    }

    /// Resets the sensor, processor and actuator properties and action from the combined dna of
    /// this object and all the plasmid-dna it contains in the inventory
    fn reread_dna(&mut self, state: &mut GameState) {
        let mut combined: Vec<u8> = self
            .inventory
            .items
            .iter()
            .filter(|o| o.dna.dna_type == DnaType::Plasmid)
            .map(|o| o.dna.raw.clone())
            .flatten()
            .collect();
        let mut complete_dna = self.dna.raw.clone();
        complete_dna.append(&mut combined);
        let (s, p, a, d) = state
            .gene_library
            .decode_dna(self.dna.dna_type, &complete_dna);
        self.change_genome(s, p, a, d);
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}] at ({},{}), alive: {}, energy: {}",
            self.visual.name,
            self.visual.character,
            self.pos.x,
            self.pos.y,
            self.alive,
            self.processors.energy
        )
    }
}
