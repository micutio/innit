/// Module Object
///
/// An Object represents the base structure for all entities in the game.
// external imports
use tcod::colors::{self, Color};
use tcod::console::*;

// internal imports
use entity::action::*;
use entity::ai::Ai;
use entity::fighter::Fighter;
use game_state::{GameState, MessageLog};

#[derive(Debug, Serialize, Deserialize)]
pub struct Object {
    pub x: i32,
    pub y: i32,
    pub dna: String,
    pub name: String, // move into a UI component
    pub blocks: bool, // move into a physics component
    pub alive: bool,
    pub chr: char,            // move into a UI component
    pub color: Color,         // move into a UI component
    pub always_visible: bool, // move into a physics component
    pub level: i32,           // could be changed into some pseudo-progress like allowed DNA length
    pub fighter: Option<Fighter>,
    pub ai: Option<Ai>,
    pub attack_action: Option<AttackAction>,
    pub next_action: Option<Box<dyn Action>>,
}

impl Object {
    pub fn new(
        x: i32,
        y: i32,
        // dna: &str, // TODO change constructor
        name: &str,
        blocks: bool,
        chr: char,
        color: Color,
    ) -> Self {
        Object {
            x,
            y,
            dna: "".into(), // dna.into(),
            name: name.into(),
            blocks,
            alive: false,
            chr,
            color,
            always_visible: false,
            level: 1,
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
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.chr, BackgroundFlag::None);
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
                    self.name, target.name, damage
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
                    self.name, target.name
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

// TODO: Replace all occurrences of objects[PLAYER].unwrap() with custom function!
#[derive(Serialize, Deserialize, Default)]
pub struct ObjectVec(Vec<Option<Object>>);

impl ObjectVec {
    pub fn new() -> Self {
        ObjectVec(Vec::new())
    }

    pub fn get_vector(&self) -> &Vec<Option<Object>> {
        &self.0
    }

    pub fn get_vector_mut(&mut self) -> &mut Vec<Option<Object>> {
        &mut self.0
    }

    pub fn push(&mut self, object: Object) {
        println!("pushing object {}", object.name);
        self.0.push(Some(object));
    }

    pub fn extract(&mut self, index: usize) -> Option<Object> {
        match self.0.get_mut(index) {
            Some(item) => match item.take() {
                Some(object) => {
                    // println!("extract object {} @ index {}", object.name, index);
                    Some(object)
                }
                None => None,
            },
            None => panic!("[ObjectVec::index] Error: invalid index {}", index),
        }
    }

    pub fn replace(&mut self, index: usize, object: Object) {
        let item = self.0.get_mut(index);
        match item {
            Some(obj) => {
                // println!("replace object {} @ index {}", object.name, index);
                obj.replace(object);
            }
            None => {
                panic!(
                    "[ObjectVec::replace] Error: object {} with given index {} does not exist!",
                    object.name, index
                );
            }
        }
    }
}

use std::ops::{Index, IndexMut};

impl Index<usize> for ObjectVec {
    type Output = Option<Object>;

    fn index(&self, i: usize) -> &Self::Output {
        let item = self.0.get(i);
        match item {
            Some(obj_option) => obj_option,
            None => panic!("[ObjectVec::index] Error: invalid index {}", i),
        }
    }
}

impl IndexMut<usize> for ObjectVec {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        let item = self.0.get_mut(i);
        match item {
            Some(obj_option) => obj_option,
            None => panic!("[ObjectVec::index] Error: invalid index {}", i),
        }
    }
}
