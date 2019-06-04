extern crate rand;
extern crate serde;
extern crate tcod;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod gui;
mod object;
mod world;

use gui::{
    initialize_fov, main_menu, menu, target_monster, target_tile, MessageLog, Messages, Tcod,
    LIMIT_FPS, PANEL_HEIGHT, SCREEN_HEIGHT, SCREEN_WIDTH,
};
use object::Object;
use world::{is_blocked, make_world, World, WORLD_HEIGHT, WORLD_WIDTH};

use rand::Rng;
use std::cmp;

use tcod::colors;
use tcod::console::*;
use tcod::map::Map as FovMap;

const TORCH_RADIUS: i32 = 10;
// player object
pub const PLAYER: usize = 0;
const HEAL_AMOUNT: i32 = 40;
const LIGHTNING_DAMAGE: i32 = 40;
const LIGHTNING_RANGE: i32 = 5;
const CONFUSE_RANGE: i32 = 8;
const CONFUSE_NUM_TURNS: i32 = 10;
const FIREBALL_RADIUS: i32 = 3;
const FIREBALL_DAMAGE: i32 = 25;
// experience and level-ups
const LEVEL_UP_BASE: i32 = 200;
const LEVEL_UP_FACTOR: i32 = 150;
const LEVEL_SCREEN_WIDTH: i32 = 40;

/// Mutably borrow two *separate* elements from the given slice.
/// Panics when the indexes are equal or out of bounds.
fn mut_two<T>(items: &mut [T], first_index: usize, second_index: usize) -> (&mut T, &mut T) {
    assert!(first_index != second_index);
    let split_at_index = cmp::max(first_index, second_index);
    let (first_slice, second_slice) = items.split_at_mut(split_at_index);
    if first_index < second_index {
        (&mut first_slice[first_index], &mut second_slice[0])
    } else {
        (&mut second_slice[0], &mut first_slice[second_index])
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameState {
    world: World,
    log: Messages,
    inventory: Vec<Object>,
    dungeon_level: u32,
}

// objects and player

#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit,
}

/// An object that can be equipped for bonuses.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Equipment {
    slot: Slot,
    equipped: bool,
    max_hp_bonus: i32,
    defense_bonus: i32,
    power_bonus: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
enum Slot {
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

fn level_up(objects: &mut [Object], game_state: &mut GameState, tcod: &mut Tcod) {
    let player = &mut objects[PLAYER];
    let level_up_xp = LEVEL_UP_BASE + player.level * LEVEL_UP_FACTOR;
    // see if the player's experience is enough to level up
    if player.fighter.as_ref().map_or(0, |f| f.xp) >= level_up_xp {
        // exp is enough, lvl up
        player.level += 1;
        game_state.log.add(
            format!(
                "Your battle skills grow stringer! You reached level {}!",
                player.level
            ),
            colors::YELLOW,
        );
        // TODO: increase player's stats
        let fighter = player.fighter.as_mut().unwrap();
        let mut choice = None;
        while choice.is_none() {
            // keep asking until a choice is made
            choice = menu(
                "Level up! Chose a stat to raise:\n",
                &[
                    format!("Constitution (+20 HP, from {})", fighter.base_max_hp),
                    format!("Strength (+1 attack, from {})", fighter.base_power),
                    format!("Agility (+1 defense, from {})", fighter.base_defense),
                ],
                LEVEL_SCREEN_WIDTH,
                &mut tcod.root,
            );
        }
        fighter.xp -= level_up_xp;
        match choice.unwrap() {
            0 => {
                fighter.base_max_hp += 20;
                fighter.hp += 20;
            }
            1 => {
                fighter.base_power += 1;
            }
            2 => {
                fighter.base_defense += 1;
            }
            _ => unreachable!(),
        }
    }
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
fn pick_item_up(game_state: &mut GameState, objects: &mut Vec<Object>, object_id: usize) {
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

fn use_item(
    tcod: &mut Tcod,
    game_state: &mut GameState,
    objects: &mut [Object],
    inventory_id: usize,
) {
    use Item::*;
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

fn drop_item(game_state: &mut GameState, objects: &mut Vec<Object>, inventory_id: usize) {
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
    tcod: &mut Tcod,
    game_state: &mut GameState,
    objects: &mut [Object],
    _inventory_id: usize,
) -> UseResult {
    let equipment = match game_state.inventory[_inventory_id].equipment {
        Some(equipment) => equipment,
        None => return UseResult::Cancelled,
    };

    // if the slot is already being used, unequip whatever is there first
    if let Some(old_equipment) = get_equipped_in_slot(equipment.slot, &game_state.inventory) {
        game_state.inventory[old_equipment].unequip(&mut game_state.log);
    }

    if equipment.equipped {
        game_state.inventory[_inventory_id].unequip(&mut game_state.log);
    } else {
        game_state.inventory[_inventory_id].equip(&mut game_state.log);
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

// combat related poperties and methods (monster, player, NPC)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fighter {
    hp: i32,
    base_max_hp: i32,
    base_defense: i32,
    base_power: i32,
    on_death: DeathCallback,
    xp: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
enum DeathCallback {
    Player,
    Monster,
}

impl DeathCallback {
    fn callback(self, object: &mut Object, messages: &mut Messages) {
        use DeathCallback::*;
        let callback: fn(&mut Object, &mut Messages) = match self {
            Player => player_death,
            Monster => monster_death,
        };
        callback(object, messages);
    }
}

fn player_death(player: &mut Object, messages: &mut Messages) {
    // the game ended!
    messages.add("You died!", colors::RED);

    // for added effect, transform the player into a corpse
    player.chr = '%';
    player.color = colors::DARK_RED;
}

fn monster_death(monster: &mut Object, messages: &mut Messages) {
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Ai {
    Basic,
    Confused {
        previous_ai: Box<Ai>,
        num_turns: i32,
    },
}

fn move_by(world: &World, objects: &mut [Object], id: usize, dx: i32, dy: i32) {
    // move by the given amount
    let (x, y) = objects[id].pos();
    if !is_blocked(world, objects, x + dx, y + dy) {
        objects[id].set_pos(x + dx, y + dy);
    }
}

fn player_move_or_attack(game_state: &mut GameState, objects: &mut [Object], dx: i32, dy: i32) {
    // the coordinate the player is moving to/attacking
    let x = objects[PLAYER].x + dx;
    let y = objects[PLAYER].y + dy;

    // try to find an attackable object there
    let target_id = objects
        .iter()
        .position(|object| object.fighter.is_some() && object.pos() == (x, y));

    // attack if target found, move otherwise
    match target_id {
        Some(target_id) => {
            let (player, target) = mut_two(objects, PLAYER, target_id);
            player.attack(target, game_state);
        }
        None => {
            move_by(&game_state.world, objects, PLAYER, dx, dy);
        }
    }
}

fn move_towards(world: &World, objects: &mut [Object], id: usize, target_x: i32, target_y: i32) {
    // vector from this object to the target, and distance
    let dx = target_x - objects[id].x;
    let dy = target_y - objects[id].y;
    let distance = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();

    // normalize it to length 1 (preserving direction), then round it and
    // convert to integer so the movement is restricted to the map grid
    let dx = (dx as f32 / distance).round() as i32;
    let dy = (dy as f32 / distance).round() as i32;
    move_by(world, objects, id, dx, dy);
}

fn ai_take_turn(
    game_state: &mut GameState,
    objects: &mut [Object],
    fov_map: &FovMap,
    monster_id: usize,
) {
    use Ai::*;
    if let Some(ai) = objects[monster_id].ai.take() {
        let new_ai = match ai {
            Basic => ai_basic(game_state, objects, fov_map, monster_id),
            Confused {
                previous_ai,
                num_turns,
            } => ai_confused(game_state, objects, monster_id, previous_ai, num_turns),
        };
        objects[monster_id].ai = Some(new_ai);
    }
}

fn ai_basic(
    game_state: &mut GameState,
    objects: &mut [Object],
    fov_map: &FovMap,
    monster_id: usize,
) -> Ai {
    // A basic monster takes its turn. If you can see it, it can see you.
    let (monster_x, monster_y) = objects[monster_id].pos();
    if fov_map.is_in_fov(monster_x, monster_y) {
        if objects[monster_id].distance_to(&objects[PLAYER]) >= 2.0 {
            // move towards player if far away
            let (player_x, player_y) = objects[PLAYER].pos();
            move_towards(&game_state.world, objects, monster_id, player_x, player_y);
        } else if objects[PLAYER].fighter.map_or(false, |f| f.hp > 0) {
            // Close enough, attack! (if player is still alive)
            let (monster, player) = mut_two(objects, monster_id, PLAYER);
            monster.attack(player, game_state);
        }
    }
    Ai::Basic
}

fn ai_confused(
    game_state: &mut GameState,
    objects: &mut [Object],
    monster_id: usize,
    previous_ai: Box<Ai>,
    num_turns: i32,
) -> Ai {
    if num_turns >= 0 {
        // still confused...
        // move in a random direction, and decrease the number of tuns confused
        move_by(
            &game_state.world,
            objects,
            monster_id,
            rand::thread_rng().gen_range(-1, 2),
            rand::thread_rng().gen_range(-1, 2),
        );
        Ai::Confused {
            previous_ai: previous_ai,
            num_turns: num_turns - 1,
        }
    } else {
        // restor the previous AI (this one will be deleted)
        game_state.log.add(
            format!("The {} is no longer confused!", objects[monster_id].name),
            colors::RED,
        );
        *previous_ai
    }
}

/// Advance to the next level
fn next_level(tcod: &mut Tcod, objects: &mut Vec<Object>, game_state: &mut GameState) {
    game_state.log.add(
        "You take a moment to rest, and recover your strength.",
        colors::VIOLET,
    );
    let heal_hp = objects[PLAYER].max_hp(game_state) / 2;
    objects[PLAYER].heal(game_state, heal_hp);

    game_state.log.add(
        "After a rare moment of peace, you descend deeper into the heart of the dungeon...",
        colors::RED,
    );
    game_state.dungeon_level += 1;
    game_state.world = make_world(objects, game_state.dungeon_level);
    initialize_fov(&game_state.world, tcod);
}

pub struct Transition {
    level: u32,
    value: u32,
}

/// Return a value that depends on level.
/// The table specifies what value occurs after each level, default is 0.
pub fn from_dungeon_level(table: &[Transition], level: u32) -> u32 {
    table
        .iter()
        .rev()
        .find(|transition| level >= transition.level)
        .map_or(0, |transition| transition.value)
}

fn main() {
    let root = Root::initializer()
        .font("assets/terminal16x16_gs_ro.png", FontLayout::AsciiInRow)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("roguelike")
        .init();

    tcod::system::set_fps(LIMIT_FPS);

    let mut tcod = Tcod {
        root: root,
        con: Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT),
        panel: Offscreen::new(SCREEN_WIDTH, PANEL_HEIGHT),
        fov: FovMap::new(WORLD_WIDTH, WORLD_HEIGHT),
        mouse: Default::default(),
    };

    main_menu(&mut tcod);
}
