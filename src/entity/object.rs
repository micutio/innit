/// Module Object
///
/// An Object represents the base structure for all entities in the game.
use tcod::colors::{self, Color};
use tcod::console::*;

// internal modules
use entity::ai::Ai;
use entity::fighter::Fighter;
use game_io::MessageLog;
use game_state::GameState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Object {
    pub x: i32,
    pub y: i32,
    pub dna: String,
    pub name: String,
    pub blocks: bool,
    pub alive: bool,
    pub chr: char,            // move into a UI component
    pub color: Color,         // move into a UI component
    pub always_visible: bool, // move into a UI component
    pub level: i32,           // could be changed into some pseudo-progress like allowed DNA length
    pub fighter: Option<Fighter>,
    pub ai: Option<Ai>,
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
