use tcod::colors::Color;
use tcod::console::*;

use crate::core::game_state::{GameState, MessageLog};
use crate::core::world::world_gen::Tile;
use crate::entity::action::*;
use crate::entity::ai::Ai;
use crate::entity::dna::{ActionPrototype, Actuators, Processors, Sensors};

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
// TODO: Use builder pattern to construct new objects.
#[derive(Debug, Serialize, Deserialize)]
pub struct Object {
    pub x:           i32,
    pub y:           i32,
    pub alive:       bool,
    pub energy:      i32, // could be changed into some pseudo-progress like allowed DNA length
    pub dna:         Vec<u8>,
    pub visual:      Visual,
    pub physics:     Physics,
    pub actions:     Vec<ActionPrototype>,
    pub tile:        Option<Tile>,
    pub sensors:     Sensors,
    pub processors:  Processors,
    pub actuators:   Actuators,
    pub next_action: Option<Box<dyn Action>>,
    pub ai:          Option<Ai>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Visual {
    pub name:      String,
    pub character: char,
    pub color:     Color,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Physics {
    pub is_blocking:       bool,
    pub is_blocking_sight: bool,
    pub is_always_visible: bool,
}

impl Object {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        x: i32,
        y: i32,
        dna: Vec<u8>,
        name: &str,
        character: char,
        color: Color,
        is_blocking: bool,
        is_blocking_sight: bool,
        is_always_visible: bool,
        sensors: Sensors,
        processors: Processors,
        actuators: Actuators,
        ai: Option<Ai>,
    ) -> Self {
        let visual = Visual {
            name: name.into(),
            character,
            color,
        };

        let physics = Physics {
            is_blocking,
            is_blocking_sight,
            is_always_visible,
        };

        Object {
            x,
            y,
            alive: false,
            energy: 0,
            dna,
            visual,
            physics,
            actions: Vec::new(),
            tile: None,
            sensors,
            processors,
            actuators,
            next_action: None,
            ai,
        }
    }

    pub fn pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    /// Set the color and then draw the char that represents this object at its position.
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.visual.color);
        con.put_char(self.x, self.y, self.visual.character, BackgroundFlag::None);
    }

    /// calculate distance to another object
    pub fn distance_to(&self, other: &Object) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        ((dx.pow(2) + dy.pow(2)) as f32).sqrt()
    }

    /// return distance to given coordinates
    pub fn distance(&self, x: i32, y: i32) -> f32 {
        (((x - self.x).pow(2) + (y - self.y).pow(2)) as f32).sqrt()
    }

    pub fn get_next_action(&mut self) -> Option<Box<dyn Action>> {
        match &self.ai {
            Some(_) => {
                // TODO: Call ai function to figure out next action!
                let pass = PassAction;
                Some(Box::new(pass))
            }
            None => self.next_action.take(),
        }
    }

    pub fn set_next_action(&mut self, next_action: Option<Box<dyn Action>>) {
        self.next_action = next_action;
    }
}
