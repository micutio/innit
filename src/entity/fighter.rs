/// Module Fighter
///
/// This module contains the structures and methods that make up the combat system.

// external imports
use tcod::colors;

// internal imports
use entity::object::Object;
use game_state::{MessageLog, Messages};

// combat related poperties and methods (monster, player, NPC)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fighter {
    pub hp: i32,
    pub base_max_hp: i32,
    pub base_defense: i32,
    pub base_power: i32,
    pub on_death: DeathCallback,
    pub xp: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum DeathCallback {
    Player,
    Monster,
}

impl DeathCallback {
    pub fn callback(self, object: &mut Object, messages: &mut Messages) {
        use entity::fighter::DeathCallback::*;
        let callback: fn(&mut Object, &mut Messages) = match self {
            Player => player_death,
            Monster => monster_death,
        };
        callback(object, messages);
    }
}

pub fn player_death(player: &mut Object, messages: &mut Messages) {
    // the game ended!
    messages.add("You died!", colors::RED);

    // for added effect, transform the player into a corpse
    // TODO: move color and character change to module game_io or object!
    player.chr = '%';
    player.color = colors::DARK_RED;
}

pub fn monster_death(monster: &mut Object, messages: &mut Messages) {
    messages.add(
        format!(
            "{} is dead! You gain {} XP",
            monster.name,
            monster.fighter.unwrap().xp
        ),
        colors::ORANGE,
    );
    monster.chr = '%';
    monster.color = colors::DARK_RED;
    monster.blocks = false;
    monster.fighter = None;
    monster.ai = None;
    monster.name = format!("remains of {}", monster.name);
}
