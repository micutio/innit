use tcod::colors::Color;

use crate::core::game_objects::GameObjects;
use crate::core::world::world_gen::Tile;
use crate::entity::action::*;
use crate::entity::ai::Ai;
use crate::entity::genetics::{Actuators, Dna, Processors, Sensors};
use crate::util::game_rng::GameRng;

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
#[derive(Debug, Serialize, Deserialize)]
pub struct Object {
    pub x: i32,
    pub y: i32,
    pub alive: bool,
    pub gene_stability: f64,
    pub dna: Dna,
    pub visual: Visual,
    pub physics: Physics,
    pub sensors: Sensors,
    pub processors: Processors,
    pub actuators: Actuators,
    pub tile: Option<Tile>,
    pub ai: Option<Box<dyn Ai>>,
    next_action: Option<Box<dyn Action>>,
    pub primary_action: Box<dyn Action>,
    pub secondary_action: Box<dyn Action>,
    pub quick1_action: Box<dyn Action>,
    pub quick2_action: Box<dyn Action>,
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
            x: 0,
            y: 0,
            alive: false,
            gene_stability: 1.0,
            dna: Dna::new(),
            visual: Visual::new(),
            physics: Physics::new(),
            sensors: Sensors::new(),
            processors: Processors::new(),
            actuators: Actuators::new(),
            tile: None,
            ai: None,
            next_action: None,
            primary_action: Box::new(PassAction),
            secondary_action: Box::new(PassAction),
            quick1_action: Box::new(PassAction),
            quick2_action: Box::new(PassAction),
        }
    }

    /// Set the object's position in the world. Part of the builder pattern.
    pub fn position(mut self, x: i32, y: i32) -> Object {
        self.x = x;
        self.y = y;
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

    /// Transform the object into an NPC. Part of the builder pattern.
    pub fn ai(mut self, ai: Box<dyn Ai>) -> Object {
        self.ai = Some(ai);
        self
    }

    pub fn metabolize(&mut self) {
        self.processors.energy = min(
            self.processors.energy + self.processors.metabolism,
            self.processors.energy_storage,
        )
    }

    /// Retrieve the current position of the object.
    pub fn pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    /// Set the current position of the object.
    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
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
        if let Some(def_action) = self
            .actuators
            .actions
            .iter()
            .find(|a| a.as_ref().get_identifier() == "move")
        {
            self.primary_action = def_action.clone_action();
            debug!(
                "{} new default action: {:#?}",
                self.visual.name, self.primary_action
            );
        }
    }

    /// Calculate the distance of this object to another object.
    pub fn distance_to(&self, other: &Object) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        ((dx.pow(2) + dy.pow(2)) as f32).sqrt()
    }

    /// Return distance of this object to a given coordinate.
    pub fn distance(&self, x: i32, y: i32) -> f32 {
        (((x - self.x).pow(2) + (y - self.y).pow(2)) as f32).sqrt()
    }

    /// Determine and return the next action the object will take.
    pub fn get_next_action(
        &mut self,
        game_objects: &mut GameObjects,
        game_rng: &mut GameRng,
    ) -> Option<Box<dyn Action>> {
        // Check if this object is ai controlled, and if so, take the ai out of the object before processing.
        if let Some(ai) = self.ai.take() {
            let next_ai_action = ai.act(self, game_objects, game_rng);
            self.ai = Some(ai);
            Some(next_ai_action)
        } else {
            self.next_action.take()
        }
    }

    /// Inject the next action this object will take into the object.
    pub fn set_next_action(&mut self, next_action: Option<Box<dyn Action>>) {
        self.next_action = next_action;
    }

    pub fn set_primary_action(&mut self, new_primary_action: Box<dyn Action>) {
        self.primary_action = new_primary_action;
    }

    pub fn set_secondary_action(&mut self, new_secondary_action: Box<dyn Action>) {
        self.secondary_action = new_secondary_action;
    }

    pub fn set_quick1_action(&mut self, new_quick1_action: Box<dyn Action>) {
        self.quick1_action = new_quick1_action;
    }

    pub fn set_quick2_action(&mut self, new_quick2_action: Box<dyn Action>) {
        self.quick2_action = new_quick2_action;
    }

    pub fn has_next_action(&self) -> bool {
        self.next_action.is_some()
    }

    pub fn get_primary_action(&self, target: Target) -> Box<dyn Action> {
        // Some(def_action.clone())
        let mut action_clone = self.primary_action.clone();
        action_clone.set_target(target);
        action_clone
    }

    pub fn get_secondary_action(&self, target: Target) -> Box<dyn Action> {
        // Some(def_action.clone())
        let mut action_clone = self.secondary_action.clone();
        action_clone.set_target(target);
        action_clone
    }

    pub fn get_quick1_action(&self) -> Box<dyn Action> {
        self.quick1_action.clone()
    }

    pub fn get_quick2_action(&self) -> Box<dyn Action> {
        self.quick2_action.clone()
    }

    pub fn get_all_actions(&self) -> Vec<&Box<dyn Action>> {
        self.actuators
            .actions
            .iter()
            .chain(self.processors.actions.iter())
            .chain(self.sensors.actions.iter())
            .collect()
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}] at ({},{}), alive: {}, energy: {}",
            self.visual.name,
            self.visual.character,
            self.x,
            self.y,
            self.alive,
            self.processors.energy
        )
    }
}
