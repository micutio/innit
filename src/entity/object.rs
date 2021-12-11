use crate::entity::action;
use crate::entity::action::Action;
use crate::entity::control;
use crate::entity::genetics;
use crate::entity::inventory::Inventory;
use crate::game;
use crate::game::game_state::MessageLog;
use crate::game::position::Position;
use crate::game::GameObjects;
use crate::game::GameState;
use crate::ui::hud;
use crate::world_gen;

use serde::{Deserialize, Serialize};
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
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[derive(Debug, Default)]
pub struct Object {
    // TODO: Add antigen-markers
    pub alive: bool,
    pub gene_stability: f64,
    pub pos: Position,
    pub visual: Visual,
    pub physics: Physics,
    pub tile: Option<world_gen::Tile>,
    pub control: Option<control::Controller>,
    pub dna: genetics::Dna,
    pub sensors: genetics::Sensors,
    pub processors: genetics::Processors,
    pub actuators: genetics::Actuators,
    pub inventory: Inventory,
    pub item: Option<InventoryItem>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Visual {
    pub name: String,
    pub glyph: char,
    pub fg_color: (u8, u8, u8, u8),
    pub bg_color: (u8, u8, u8, u8),
}

impl Visual {
    pub fn new() -> Self {
        Visual {
            name: "unknown".into(),
            glyph: '_',
            fg_color: (255, 255, 255, 255),
            bg_color: (0, 0, 0, 255),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Physics {
    pub is_blocking: bool,
    pub is_blocking_sight: bool,
    pub is_always_visible: bool,
    pub is_visible: bool,
}

impl Physics {
    pub fn new() -> Self {
        Physics {
            is_blocking: false,
            is_blocking_sight: false,
            is_always_visible: false,
            is_visible: false,
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[derive(Debug, Default, Clone)]
pub struct InventoryItem {
    pub description: String,
    pub use_action: Option<Box<dyn Action>>,
}

impl InventoryItem {
    pub fn new<S: Into<String>>(descr: S, use_action: Option<Box<dyn Action>>) -> Self {
        InventoryItem {
            description: descr.into(),
            use_action,
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}] at ({},{}), alive: {}, energy: {}",
            self.visual.name,
            self.visual.glyph,
            self.pos.x,
            self.pos.y,
            self.alive,
            self.processors.energy
        )
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
            dna: genetics::Dna::new(),
            visual: Visual::new(),
            physics: Physics::new(),
            sensors: genetics::Sensors::new(),
            processors: genetics::Processors::new(),
            actuators: genetics::Actuators::new(),
            inventory: Inventory::new(),
            item: None,
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
    pub fn visualize(mut self, name: &str, character: char, fg_color: (u8, u8, u8, u8)) -> Object {
        self.visual.name = name.into();
        self.visual.glyph = character;
        self.visual.fg_color = fg_color;
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
        self.physics.is_visible = is_always_visible;
        self
    }

    /// Set the object's dna and super traits. Part of the builder pattern.
    pub fn genome(
        mut self,
        stability: f64,
        (sensors, processors, actuators, dna): (
            genetics::Sensors,
            genetics::Processors,
            genetics::Actuators,
            genetics::Dna,
        ),
    ) -> Object {
        self.gene_stability = stability;
        self.set_genome(sensors, processors, actuators, dna);

        // debug!("default action: {:#?}", self.default_action);
        self
    }

    /// Turn the object into a collectible item. Part of the builder pattern.
    pub fn inventory_item(mut self, item: InventoryItem) -> Object {
        self.item = Some(item);
        self
    }

    /// Transform the object into a tile. Part of the builder pattern.
    /// Ref. https://www.ncbi.nlm.nih.gov/pmc/articles/PMC3848882/ for overview on chem. gradients.
    pub fn tile_explored(mut self, is_explored: bool) -> Object {
        self.tile = Some(world_gen::Tile {
            is_explored,
            morphogen: 0.0,
        });
        self
    }

    /// Transform the object into an NPC or player. Part of the builder pattern.
    pub fn control(mut self, controller: control::Controller) -> Object {
        self.control = Some(controller);
        self
    }

    pub fn control_opt(mut self, controller: Option<control::Controller>) -> Object {
        self.control = controller;
        self
    }

    /// Turn the object into an item that can be added to the inventory. Part of builder pattern.
    pub fn itemize(mut self, item: Option<InventoryItem>) -> Object {
        self.item = item;
        self
    }

    /// Perform necessary actions when object dies.
    pub fn die(&mut self, _state: &mut GameState, objects: &mut GameObjects) {
        if let Some(_) = self.tile {
            // if this object is a tile, just revert it to a floor tile
            self.physics.is_blocking = false;
            self.physics.is_blocking_sight = false;
            self.visual.glyph = '·';
            self.visual.name = "empty tile".into();
            self.control = None;
        } else {
            self.alive = false;
            // empty inventory into this objects' current position
            for mut o in self.inventory.items.drain(..) {
                o.pos.set(self.pos.x, self.pos.y);
                objects.push(o);
            }
            // take this object out of the world
            if self.is_player() {
                self.visual.name = "your remains".to_string();
            }
        }
    }

    pub fn is_player(&self) -> bool {
        if let Some(control::Controller::Player(_)) = self.control {
            true
        } else {
            false
        }
    }

    /// Transform the object into an NPC or player. Part of the builder pattern.
    pub fn set_control(mut self, controller: control::Controller, log: &mut game::game_state::Log) {
        match controller {
            control::Controller::Npc(_) => {
                if self.is_player() {
                    log.add(
                        format!("You lost control over {}", &self.visual.name),
                        game::game_state::MsgClass::Alert,
                    );
                }
            }
            control::Controller::Player(_) => {
                if let Some(control::Controller::Npc(_)) = self.control {
                    log.add(
                        format!("You gained control over {}", &self.visual.name),
                        game::game_state::MsgClass::Alert,
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
    pub fn set_genome(
        &mut self,
        sensors: genetics::Sensors,
        processors: genetics::Processors,
        actuators: genetics::Actuators,
        dna: genetics::Dna,
    ) {
        self.sensors = sensors;
        self.processors = processors;
        self.actuators = actuators;
        self.dna = dna;

        // update default action
        if let Some(control::Controller::Player(ref mut ctrl)) = &mut self.control {
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

            if let Some(def_action) = self
                .actuators
                .actions
                .iter()
                .find(|a| a.as_ref().get_identifier() == "pick up item")
            {
                ctrl.quick1_action = def_action.clone_action();
                debug!(
                    "{} new quick action: {:#?}",
                    self.visual.name, ctrl.primary_action
                );
            }
        }
    }

    /// Re-generate genetic traits from the raw dna and then re-set all dna-dependent fields.
    pub fn update_genome_from_dna(&mut self, state: &mut GameState) {
        let (new_s, new_p, new_a, new_d) = state
            .gene_library
            .dna_to_traits(self.dna.dna_type, &self.dna.raw);
        self.set_genome(new_s, new_p, new_a, new_d);
    }

    /// Determine and return the next action the object will take.
    pub fn extract_next_action(
        &mut self,
        state: &mut GameState,
        objects: &mut GameObjects,
    ) -> Option<Box<dyn Action>> {
        // Check if this object is ai controlled, and if so, take the ai out of the object before processing.
        let mut controller_opt = self.control.take();
        let next_action: Option<Box<dyn Action>>;
        match controller_opt {
            Some(control::Controller::Npc(ref mut boxed_ai)) => {
                next_action = Some(boxed_ai.act(state, objects, self));
            }
            Some(control::Controller::Player(ref mut player_ctrl)) => {
                next_action = player_ctrl.next_action.take();
            }
            None => next_action = None,
        }

        if self.control.is_none() {
            self.control = controller_opt;
        }
        next_action
    }

    /// Inject the next action this object will take into the object.
    pub fn set_next_action(&mut self, next_action: Option<Box<dyn Action>>) {
        let mut controller = self.control.take();
        if let Some(control::Controller::Player(ref mut ctrl)) = controller {
            ctrl.next_action = next_action;
        }
        self.control = controller;
    }

    pub fn set_primary_action(&mut self, new_primary_action: Box<dyn Action>) {
        let mut controller = self.control.take();
        if let Some(control::Controller::Player(ref mut ctrl)) = controller {
            ctrl.primary_action = new_primary_action;
        }
        self.control = controller;
    }

    pub fn set_secondary_action(&mut self, new_secondary_action: Box<dyn Action>) {
        let mut controller = self.control.take();
        if let Some(control::Controller::Player(ref mut ctrl)) = controller {
            ctrl.secondary_action = new_secondary_action;
        }
        self.control = controller;
    }

    pub fn set_quick1_action(&mut self, new_quick1_action: Box<dyn Action>) {
        let mut controller = self.control.take();
        if let Some(control::Controller::Player(ref mut ctrl)) = controller {
            ctrl.quick1_action = new_quick1_action;
        }
        self.control = controller;
    }

    pub fn set_quick2_action(&mut self, new_quick2_action: Box<dyn Action>) {
        let mut controller = self.control.take();
        if let Some(control::Controller::Player(ref mut ctrl)) = controller {
            ctrl.quick2_action = new_quick2_action;
        }
        self.control = controller;
    }

    pub fn has_next_action(&self) -> bool {
        if let Some(control::Controller::Player(ctrl)) = &self.control {
            ctrl.next_action.is_some()
        } else {
            false
        }
    }

    // NOTE: Consider moving the player-action-related methods into PlayerCtrl.

    pub fn get_primary_action(&self, target: action::Target) -> Box<dyn Action> {
        // Some(def_action.clone())
        if let Some(control::Controller::Player(ctrl)) = &self.control {
            let mut action_clone = ctrl.primary_action.clone();
            action_clone.set_target(target);
            action_clone
        } else {
            Box::new(action::hereditary::ActPass::default())
        }
    }

    pub fn get_secondary_action(&self, target: action::Target) -> Box<dyn Action> {
        // Some(def_action.clone())
        if let Some(control::Controller::Player(ctrl)) = &self.control {
            let mut action_clone = ctrl.secondary_action.clone();
            action_clone.set_target(target);
            action_clone
        } else {
            Box::new(action::hereditary::ActPass::default())
        }
    }

    pub fn get_quick1_action(&self) -> Box<dyn Action> {
        if let Some(control::Controller::Player(ctrl)) = &self.control {
            ctrl.quick1_action.clone()
        } else {
            Box::new(action::hereditary::ActPass::default())
        }
    }

    pub fn get_quick2_action(&self) -> Box<dyn Action> {
        if let Some(control::Controller::Player(ctrl)) = &self.control {
            ctrl.quick2_action.clone()
        } else {
            Box::new(action::hereditary::ActPass::default())
        }
    }

    pub fn match_action(&self, id: &str) -> Option<Box<dyn Action>> {
        self.actuators
            .actions
            .iter()
            .chain(self.processors.actions.iter())
            .chain(self.sensors.actions.iter())
            .find(|a| a.get_identifier().eq(id))
            .cloned()
    }

    pub fn add_to_inventory(&mut self, o: Object) {
        let new_idx = self.inventory.items.len();
        // add item to inventory
        self.inventory.items.push(o);
        // add action to drop it
        self.inventory
            .inv_actions
            .push(Box::new(action::inventory::ActDropItem::new(
                new_idx as i32,
            )));
    }

    pub fn remove_from_inventory(&mut self, index: usize) -> Object {
        self.inventory.items.remove(index)
    }

    pub fn set_dna(&mut self, new_dna: genetics::Dna) {
        self.dna = new_dna;
    }

    /// Retrieve the genetic traits and actions of this object's dna combined with those of all
    /// plasmid-type items in the inventory
    pub fn get_combined_dna(
        &self,
    ) -> Vec<(
        &genetics::Sensors,
        &genetics::Processors,
        &genetics::Actuators,
        &genetics::Dna,
    )> {
        let mut combined_dna = Vec::new();
        // append own dna first
        combined_dna.push((&self.sensors, &self.processors, &self.actuators, &self.dna));
        let mut inventory_dna = self
            .inventory
            .items
            .iter()
            .filter(|o| o.dna.dna_type == genetics::DnaType::Plasmid)
            .map(|o| (&o.sensors, &o.processors, &o.actuators, &o.dna))
            .collect::<Vec<(
                &genetics::Sensors,
                &genetics::Processors,
                &genetics::Actuators,
                &genetics::Dna,
            )>>();
        combined_dna.append(&mut inventory_dna);

        combined_dna
    }

    /// Retrieve the genetic traits and actions of this object's dna combined with those of all
    /// plasmid-type items in the inventory
    pub fn get_combined_simplified_dna(&self) -> Vec<&genetics::GeneticTrait> {
        let mut combined_dna = Vec::new();
        self.dna
            .simplified
            .iter()
            .for_each(|genetic_trait| combined_dna.push(genetic_trait));

        self.inventory
            .items
            .iter()
            .filter(|o| o.dna.dna_type == genetics::DnaType::Plasmid)
            .for_each(|o| {
                o.dna
                    .simplified
                    .iter()
                    .for_each(|genetic_trait| combined_dna.push(genetic_trait));
            });

        combined_dna
    }

    pub fn generate_tooltip(&self, other: &Object) -> hud::ToolTip {
        // show whether both objects have matching receptors
        let receptor_match = if self
            .processors
            .receptors
            .iter()
            .any(|e1| other.processors.receptors.iter().any(|e2| e1.typ == e2.typ))
        {
            "match".to_string()
        } else {
            "no match".to_string()
        };

        // tiles have reduced information (might change in the future) since they are static
        if self.tile.is_some() {
            return if !self.physics.is_blocking {
                hud::ToolTip::no_header(vec![])
            } else {
                let attributes = vec![("receptors:".to_string(), receptor_match)];
                hud::ToolTip::new(self.visual.name.clone(), attributes)
            };
        }

        let header = self.visual.name.clone();
        let attributes: Vec<(String, String)> = vec![
            (
                "position".to_string(),
                format!("{}, {}", self.pos.x, self.pos.y),
            ),
            (
                "hp:".to_string(),
                format!("{}/{}", self.actuators.hp, self.actuators.max_hp).to_string(),
            ),
            (
                "energy:".to_string(),
                format!(
                    "{}/{}",
                    self.processors.energy, self.processors.energy_storage
                )
                .to_string(),
            ),
            (
                "sense range:".to_string(),
                self.sensors.sensing_range.to_string(),
            ),
            ("receptors:".to_string(), receptor_match),
        ];
        hud::ToolTip::new(header, attributes)
    }
}
