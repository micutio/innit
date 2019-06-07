//! Module Item
//!
//! This module contains all structures and methods pertaining to items
//! which can be equipped, used and casted.

use ai::Ai;
use game_state::{GameState, PLAYER};
use gui::{target_monster, target_tile, MessageLog, Tcod};
use object::Object;

use tcod::colors;

const HEAL_AMOUNT: i32 = 40;
const LIGHTNING_DAMAGE: i32 = 40;
const LIGHTNING_RANGE: i32 = 5;
const CONFUSE_RANGE: i32 = 8;
const CONFUSE_NUM_TURNS: i32 = 10;
const FIREBALL_RADIUS: i32 = 3;
const FIREBALL_DAMAGE: i32 = 25;

/// An object that can be equipped for bonuses.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Equipment {
    pub slot: Slot,
    pub equipped: bool,
    pub max_hp_bonus: i32,
    pub defense_bonus: i32,
    pub power_bonus: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Slot {
    LeftHand,
    RightHand,
    Head,
}

impl std::fmt::Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Slot::LeftHand => write!(f, "left hand"),
            Slot::RightHand => write!(f, "right hand"),
            Slot::Head => write!(f, "head"),
            _ => unreachable!(),
        }
    }
}

fn get_equipped_in_slot(slot: Slot, inventory: &[Object]) -> Option<usize> {
    for (inventory_id, item) in inventory.iter().enumerate() {
        if item
            .equipment
            .as_ref()
            .map_or(false, |e| e.equipped && e.slot == slot)
        {
            return Some(inventory_id);
        }
    }
    None
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Item {
    Heal,
    Lightning,
    Fireball,
    Confuse,
    Sword,
    Shield,
}

/// Add given item to the player's inventory and remove from the map.
pub fn pick_item_up(game_state: &mut GameState, objects: &mut Vec<Object>, object_id: usize) {
    if game_state.inventory.len() >= 26 {
        game_state.log.add(
            format!(
                "Your inventory is full, cannot pick up {}.",
                objects[object_id].name
            ),
            colors::RED,
        );
    } else {
        let item = objects.swap_remove(object_id);
        game_state
            .log
            .add(format!("You picked up a {}!", item.name), colors::GREEN);
        let index = game_state.inventory.len();
        let slot = item.equipment.map(|e| e.slot);
        game_state.inventory.push(item);

        // automatically equip, if the corresponding equipment slot is unused
        if let Some(slot) = slot {
            if get_equipped_in_slot(slot, &game_state.inventory).is_none() {
                game_state.inventory[index].equip(&mut game_state.log);
            }
        }
    }
}

enum UseResult {
    UsedUp,
    UsedAndKept,
    Cancelled,
}

pub fn use_item(
    tcod: &mut Tcod,
    game_state: &mut GameState,
    objects: &mut [Object],
    inventory_id: usize,
) {
    use item::Item::*;
    // just call the use_function, if it is defined
    if let Some(item) = game_state.inventory[inventory_id].item {
        let on_use = match item {
            Heal => cast_heal,
            Lightning => cast_lightning,
            Fireball => cast_fireball,
            Confuse => cast_confuse,
            Sword => toggle_equipment,
            Shield => toggle_equipment,
            _ => unreachable!(),
        };
        match on_use(tcod, game_state, objects, inventory_id) {
            UseResult::UsedUp => {
                // destroy after use, unless it was cancelled for some reason
                game_state.inventory.remove(inventory_id);
            }
            UseResult::UsedAndKept => {} // do nothing
            UseResult::Cancelled => {
                game_state.log.add("Cancelled", colors::WHITE);
            }
        }
    } else {
        game_state.log.add(
            format!(
                "The {} cannot be used.",
                game_state.inventory[inventory_id].name
            ),
            colors::WHITE,
        );
    }
}

pub fn drop_item(game_state: &mut GameState, objects: &mut Vec<Object>, inventory_id: usize) {
    let mut item = game_state.inventory.remove(inventory_id);

    if item.equipment.is_some() {
        item.unequip(&mut game_state.log);
    }

    item.set_pos(objects[PLAYER].x, objects[PLAYER].y);
    game_state
        .log
        .add(format!("You dropped a {}.", item.name), colors::YELLOW);
    objects.push(item);
}

#[allow(unused_variables)]
fn cast_heal(
    tcod: &mut Tcod,
    game_state: &mut GameState,
    objects: &mut [Object],
    _inventory_id: usize,
) -> UseResult {
    // heal the player
    let player = &mut objects[PLAYER];
    if let Some(fighter) = player.fighter {
        if fighter.hp == player.max_hp(game_state) {
            game_state
                .log
                .add("You are already at full health.", colors::RED);
            return UseResult::Cancelled;
        }
        game_state
            .log
            .add("Your wounds start to feel better!", colors::LIGHT_VIOLET);
        player.heal(game_state, HEAL_AMOUNT);
        return UseResult::UsedUp;
    }
    UseResult::Cancelled
}

fn cast_lightning(
    tcod: &mut Tcod,
    game_state: &mut GameState,
    objects: &mut [Object],
    _inventory_id: usize,
) -> UseResult {
    // find closest enemy (inside a maximum range) and damage it
    let monster_id = closest_monster(tcod, objects, LIGHTNING_RANGE);
    if let Some(monster_id) = monster_id {
        // zap it!
        game_state.log.add(
            format!(
                "A lightning bolt strikes the {} with a loud thunder! The damage is {} hit points.",
                objects[monster_id].name, LIGHTNING_DAMAGE
            ),
            colors::LIGHT_BLUE,
        );
        if let Some(xp) = objects[monster_id].take_damage(LIGHTNING_DAMAGE, game_state) {
            objects[PLAYER].fighter.as_mut().unwrap().xp += xp;
        }
        UseResult::UsedUp
    } else {
        // no enemy found withing maximum range
        game_state
            .log
            .add("No enemy is close enough to strike.", colors::RED);
        UseResult::Cancelled
    }
}

fn cast_confuse(
    tcod: &mut Tcod,
    game_state: &mut GameState,
    objects: &mut [Object],
    _inventory_id: usize,
) -> UseResult {
    // find closest enemy in range and confuse it
    // let monster_id = closest_monster(CONFUSE_RANGE, objects, tcod);
    // ask the player for a target to confuse
    game_state.log.add(
        "Left-click an enemy to confuse, or right-click to cancel",
        colors::LIGHT_CYAN,
    );
    let monster_id = target_monster(tcod, game_state, objects, Some(CONFUSE_RANGE as f32));
    if let Some(monster_id) = monster_id {
        let old_ai = objects[monster_id].ai.take().unwrap_or(Ai::Basic);
        // replace monster's AI with a `confused` one
        // after some turns it will restore the old AI
        objects[monster_id].ai = Some(Ai::Confused {
            previous_ai: Box::new(old_ai),
            num_turns: CONFUSE_NUM_TURNS,
        });
        game_state.log.add(
            format!(
                "The eyes of {} look vacant, as it starts to stumble around!",
                objects[monster_id].name
            ),
            colors::LIGHT_GREEN,
        );
        UseResult::UsedUp
    } else {
        // no enemy found in maximum range
        game_state
            .log
            .add("No enemy is close enough to strike.", colors::RED);
        UseResult::Cancelled
    }
}

fn cast_fireball(
    tcod: &mut Tcod,
    game_state: &mut GameState,
    objects: &mut [Object],
    _inventory_id: usize,
) -> UseResult {
    // ask the player to target a tile to throw a fireball at
    game_state.log.add(
        "Left-click a target tile for the fireball, or right-click to cancel.",
        colors::LIGHT_CYAN,
    );
    let (x, y) = match target_tile(tcod, game_state, objects, None) {
        Some(tile_pos) => tile_pos,
        None => return UseResult::Cancelled,
    };
    game_state.log.add(
        format!(
            "The fireball explodes, burning everything within {} tiles!",
            FIREBALL_RADIUS
        ),
        colors::ORANGE,
    );

    let mut xp_to_gain = 0;
    for (id, obj) in objects.iter_mut().enumerate() {
        if obj.distance(x, y) <= FIREBALL_RADIUS as f32 && obj.fighter.is_some() {
            game_state.log.add(
                format!(
                    "The {} gets burned for {} hit points.",
                    obj.name, FIREBALL_DAMAGE
                ),
                colors::ORANGE,
            );
            if let Some(xp) = obj.take_damage(FIREBALL_DAMAGE, game_state) {
                if id != PLAYER {
                    // Don't reward the player for bunring themself!
                    xp_to_gain += xp;
                }
            }
        }
    }
    objects[PLAYER].fighter.as_mut().unwrap().xp += xp_to_gain;

    UseResult::UsedUp
}

fn toggle_equipment(
    _tcod: &mut Tcod,
    game_state: &mut GameState,
    _objects: &mut [Object],
    inventory_id: usize,
) -> UseResult {
    let equipment = match game_state.inventory[inventory_id].equipment {
        Some(equipment) => equipment,
        None => return UseResult::Cancelled,
    };

    // if the slot is already being used, unequip whatever is there first
    if let Some(old_equipment) = get_equipped_in_slot(equipment.slot, &game_state.inventory) {
        game_state.inventory[old_equipment].unequip(&mut game_state.log);
    }

    if equipment.equipped {
        game_state.inventory[inventory_id].unequip(&mut game_state.log);
    } else {
        game_state.inventory[inventory_id].equip(&mut game_state.log);
    }
    UseResult::UsedAndKept
}

fn closest_monster(tcod: &Tcod, objects: &mut [Object], max_range: i32) -> Option<usize> {
    let mut closest_enemy = None;
    let mut closest_dist = (max_range + 1) as f32;

    for (id, object) in objects.iter().enumerate() {
        if (id != PLAYER)
            && object.fighter.is_some()
            && object.ai.is_some()
            && tcod.fov.is_in_fov(object.x, object.y)
        {
            // calculate distance between this object and the player
            let dist = objects[PLAYER].distance_to(object);
            if dist < closest_dist {
                // it's closer, so remember it
                closest_enemy = Some(id);
                closest_dist = dist;
            }
        }
    }

    closest_enemy
}
