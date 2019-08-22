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
///
/// **Sensor** - the object's organelle/perception
///
/// - sensing range
///
/// **Processor** - the object's brain
///
/// - reaction time bonus/penalty
///
/// **Actuator** - the object's body
///
/// - membrane integrity a.k.a health points
///
/// Attributes:
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
    // NOTE: Rather use builder pattern here
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

    // TODO: Re-write or delete the methods below!

    // pub fn take_damage(&mut self, damage: i32, game_state: &mut GameState) -> Option<i32> {
    //     // apply damage if possible
    //     if let Some(fighter) = self.fighter.as_mut() {
    //         if damage > 0 {
    //             fighter.hp -= damage;
    //         }
    //     }

    //     // check for death, trigger death callback function
    //     if let Some(fighter) = self.fighter {
    //         if fighter.hp <= 0 {
    //             self.alive = false;
    //             fighter.on_death.callback(self, &mut game_state.log);
    //             return Some(fighter.xp);
    //         }
    //     }
    //     None
    // }

    // pub fn power(&self, _game_state: &GameState) -> i32 {
    //     self.fighter.map_or(0, |f| f.base_power)
    // }

    // pub fn attack(&mut self, target: &mut Object, game_state: &mut GameState) {
    //     // simple formula for attack damage
    //     let damage = self.power(game_state) - target.defense(game_state);
    //     if damage > 0 {
    //         // make the target take some damage
    //         game_state.log.add(
    //             format!(
    //                 "{} attacks {} for {} hit points.",
    //                 self.visual.name, target.visual.name, damage
    //             ),
    //             colors::WHITE,
    //         );
    //         // target.take_damage(damage, messages);
    //         if let Some(xp) = target.take_damage(damage, game_state) {
    //             // yield experience to the player
    //             self.fighter.as_mut().unwrap().xp += xp;
    //         }
    //     } else {
    //         game_state.log.add(
    //             format!(
    //                 "{} attacks {} but it has no effect!",
    //                 self.visual.name, target.visual.name
    //             ),
    //             colors::WHITE,
    //         );
    //     }
    // }

    // pub fn defense(&self, _game_state: &GameState) -> i32 {
    //     self.fighter.map_or(0, |f| f.base_defense)
    // }

    // pub fn max_hp(&self, _game_state: &GameState) -> i32 {
    //     self.fighter.map_or(0, |f| f.base_max_hp)
    // }

    // /// heal by the given amount, without going over the maxmimum
    // pub fn heal(&mut self, game_state: &GameState, amount: i32) {
    //     let max_hp = self.max_hp(game_state);
    //     if let Some(ref mut fighter) = self.fighter {
    //         fighter.hp += amount;
    //         if fighter.hp > max_hp {
    //             fighter.hp = max_hp;
    //         }
    //     }
    // }
}
