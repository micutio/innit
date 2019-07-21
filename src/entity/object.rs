/// Module Object
///
/// An Object represents the base structure for all entities in the game.
// external imports
use tcod::colors::{self, Color};
use tcod::console::*;

// internal imports
use core::game_state::{GameState, MessageLog};
use core::world::world_gen::Tile;
use entity::action::*;
use entity::ai::Ai;
use entity::fighter::Fighter;

#[derive(Debug, Serialize, Deserialize)]
pub struct Object {
    pub x: i32,
    pub y: i32,
    pub dna: String,
    pub alive: bool,
    pub level: i32, // could be changed into some pseudo-progress like allowed DNA length
    pub visual: Visual,
    pub physics: Physics,
    pub tile: Option<Tile>,
    pub fighter: Option<Fighter>,
    pub ai: Option<Ai>,
    pub attack_action: Option<AttackAction>,
    pub next_action: Option<Box<dyn Action>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Visual {
    pub name: String,
    pub character: char,
    pub color: Color,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Physics {
    pub is_blocking: bool,
    pub is_blocking_sight: bool,
    pub is_always_visible: bool,
}

impl Object {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        x: i32,
        y: i32,
        // dna: &str, // TODO change constructor
        name: &str,
        character: char,
        color: Color,
        is_blocking: bool,
        is_blocking_sight: bool,
        is_always_visible: bool,
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
            dna: "".into(), // dna.into(),
            alive: false,
            level: 1,
            visual,
            physics,
            tile: None,
            fighter: None,
            ai: None,
            attack_action: None,
            next_action: None,
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
    pub fn draw(&self, con: &mut Console) {
        con.set_default_foreground(self.visual.color);
        con.put_char(self.x, self.y, self.visual.character, BackgroundFlag::None);
    }

    pub fn distance_to(&self, other: &Object) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        ((dx.pow(2) + dy.pow(2)) as f32).sqrt()
    }

    /// return distance between some coordinates and this object
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

    pub fn take_damage(&mut self, damage: i32, game_state: &mut GameState) -> Option<i32> {
        // apply damage if possible
        if let Some(fighter) = self.fighter.as_mut() {
            if damage > 0 {
                fighter.hp -= damage;
            }
        }

        // check for death, trigger death callback function
        if let Some(fighter) = self.fighter {
            if fighter.hp <= 0 {
                self.alive = false;
                fighter.on_death.callback(self, &mut game_state.log);
                return Some(fighter.xp);
            }
        }
        None
    }

    pub fn power(&self, _game_state: &GameState) -> i32 {
        self.fighter.map_or(0, |f| f.base_power)
    }

    pub fn attack(&mut self, target: &mut Object, game_state: &mut GameState) {
        // simple formula for attack damage
        let damage = self.power(game_state) - target.defense(game_state);
        if damage > 0 {
            // make the target take some damage
            game_state.log.add(
                format!(
                    "{} attacks {} for {} hit points.",
                    self.visual.name, target.visual.name, damage
                ),
                colors::WHITE,
            );
            // target.take_damage(damage, messages);
            if let Some(xp) = target.take_damage(damage, game_state) {
                // yield experience to the player
                self.fighter.as_mut().unwrap().xp += xp;
            }
        } else {
            game_state.log.add(
                format!(
                    "{} attacks {} but it has no effect!",
                    self.visual.name, target.visual.name
                ),
                colors::WHITE,
            );
        }
    }

    pub fn defense(&self, _game_state: &GameState) -> i32 {
        self.fighter.map_or(0, |f| f.base_defense)
    }

    pub fn max_hp(&self, _game_state: &GameState) -> i32 {
        self.fighter.map_or(0, |f| f.base_max_hp)
    }

    /// heal by the given amount, without going over the maxmimum
    pub fn heal(&mut self, game_state: &GameState, amount: i32) {
        let max_hp = self.max_hp(game_state);
        if let Some(ref mut fighter) = self.fighter {
            fighter.hp += amount;
            if fighter.hp > max_hp {
                fighter.hp = max_hp;
            }
        }
    }
}
