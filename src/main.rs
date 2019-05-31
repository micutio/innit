extern crate rand;
extern crate serde;
extern crate tcod;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use rand::Rng;
use std::cmp;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use tcod::colors::{self, Color};
use tcod::console::*;
use tcod::input::{self, Event, Key, Mouse};
use tcod::map::{FovAlgorithm, Map as FovMap};

// window size
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
// target fps
const LIMIT_FPS: i32 = 20;
// constraints for field of view computing and rendering
const FOV_ALG: FovAlgorithm = FovAlgorithm::Shadow;
const FOV_LIGHT_WALLS: bool = true;
const TORCH_RADIUS: i32 = 10;
// player object
const PLAYER: usize = 0;
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
// map constraints
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 43;
const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_LIGHT_WALL: Color = Color {
    r: 130,
    g: 110,
    b: 50,
};
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};
const COLOR_LIGHT_GROUND: Color = Color {
    r: 200,
    g: 180,
    b: 50,
};
// room generation constraints
const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;
// UI constraints
const BAR_WIDTH: i32 = 20;
const PANEL_HEIGHT: i32 = 7;
const PANEL_Y: i32 = SCREEN_HEIGHT - PANEL_HEIGHT;
const MSG_X: i32 = BAR_WIDTH + 2;
const MSG_WIDTH: i32 = SCREEN_WIDTH - BAR_WIDTH - 2;
const MSG_HEIGHT: usize = PANEL_HEIGHT as usize - 1;
type Messages = Vec<(String, Color)>;
const INVENTORY_WIDTH: i32 = 50;
const CHARACTER_SCREEN_WIDTH: i32 = 30;

struct Tcod {
    root: Root,
    con: Offscreen,
    panel: Offscreen,
    fov: FovMap,
    mouse: Mouse,
}

#[derive(Serialize, Deserialize)]
struct GameState {
    map: Map,
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

#[derive(Debug, Serialize, Deserialize)]
struct Object {
    x: i32,
    y: i32,
    name: String,
    blocks: bool,
    alive: bool,
    chr: char,
    color: Color,
    fighter: Option<Fighter>,
    ai: Option<Ai>,
    item: Option<Item>,
    always_visible: bool,
    level: i32,
    equipment: Option<Equipment>,
}

impl Object {
    pub fn new(x: i32, y: i32, name: &str, blocks: bool, chr: char, color: Color) -> Self {
        Object {
            x: x,
            y: y,
            name: name.into(),
            blocks: blocks,
            alive: false,
            chr: chr,
            color: color,
            fighter: None,
            ai: None,
            item: None,
            always_visible: false,
            level: 1,
            equipment: None,
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

    /// Erase the character that represents this object
    pub fn clear(&self, con: &mut Console) {
        con.put_char(self.x, self.y, ' ', BackgroundFlag::None);
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

    pub fn power(&self, game_state: &GameState) -> i32 {
        let base_power = self.fighter.map_or(0, |f| f.power);
        let bonus: i32 = self
            .get_all_equipped(game_state)
            .iter()
            .map(|e| e.power_bonus)
            .sum();
        base_power + bonus
    }

    pub fn attack(&mut self, target: &mut Object, game_state: &mut GameState) {
        // simple formula for attack damage
        let damage = self.fighter.map_or(0, |f| f.power) - target.fighter.map_or(0, |f| f.defense);
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

    /// heal by the given amount, without going over the maxmimum
    pub fn heal(&mut self, amount: i32) {
        if let Some(ref mut fighter) = self.fighter {
            fighter.hp += amount;
            if fighter.hp > fighter.max_hp {
                fighter.hp = fighter.max_hp;
            }
        }
    }

    /// Try to equip an object and show a message about it.
    pub fn equip(&mut self, log: &mut Vec<(String, Color)>) {
        if self.item.is_none() {
            log.add(
                format!("Can't equip{:?} because it's not an item.'", self),
                colors::RED,
            );
            return;
        };
        if let Some(ref mut equipment) = self.equipment {
            if !equipment.equipped {
                equipment.equipped = true;
                log.add(
                    format!("Equipped {:?} on {}.", self.name, equipment.slot),
                    colors::LIGHT_GREEN,
                );
            }
        } else {
            log.add(
                format!("Can't equip {:?} because it's not an Equipment.'", self),
                colors::RED,
            );
        }
    }

    /// Try to unequip an object and show a message about it
    pub fn unequip(&mut self, log: &mut Vec<(String, Color)>) {
        if self.item.is_none() {
            log.add(
                format!("Can't unequip {:?} because it's not an item.", self),
                colors::RED,
            );
            return;
        };
        if let Some(ref mut equipment) = self.equipment {
            if equipment.equipped {
                equipment.equipped = false;
                log.add(
                    format!("Unequipped {} from {}.", self.name, equipment.slot),
                    colors::LIGHT_YELLOW,
                );
            }
        } else {
            log.add(
                format!("Can't uneqip {:?} because it's not an Equipment.", self),
                colors::RED,
            );
        }
    }

    /// Return a list of all equipped items
    pub fn get_all_equipped(&self, game_state: &GameState) -> Vec<Equipment> {
        if self.name == "player" {
            game_state
                .inventory
                .iter()
                .filter(|item| item.equipment.map_or(false, |e| e.equipped))
                .map(|item| item.equipment.unwrap())
                .collect()
        } else {
            vec![] // other objects have no equipment
        }
    }
}

/// An object that can be equipped for bonuses.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
struct Equipment {
    slot: Slot,
    equipped: bool,
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
                    format!("Constitution (+20 HP, from {})", fighter.max_hp),
                    format!("Strength (+1 attack, from {})", fighter.power),
                    format!("Agility (+1 defense, from {})", fighter.defense),
                ],
                LEVEL_SCREEN_WIDTH,
                &mut tcod.root,
            );
        }
        fighter.xp -= level_up_xp;
        match choice.unwrap() {
            0 => {
                fighter.max_hp += 20;
                fighter.hp += 20;
            }
            1 => {
                fighter.power += 1;
            }
            2 => {
                fighter.defense += 1;
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
enum Item {
    Heal,
    Lightning,
    Fireball,
    Confuse,
    Equipment,
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
            Equipment => toggle_equipment,
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
    if let Some(fighter) = objects[PLAYER].fighter {
        if fighter.hp == fighter.max_hp {
            game_state
                .log
                .add("You are already at full health.", colors::RED);
            return UseResult::Cancelled;
        }
        game_state
            .log
            .add("Your wounds start to feel better!", colors::LIGHT_VIOLET);
        objects[PLAYER].heal(HEAL_AMOUNT);
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
struct Fighter {
    max_hp: i32,
    hp: i32,
    defense: i32,
    power: i32,
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
enum Ai {
    Basic,
    Confused {
        previous_ai: Box<Ai>,
        num_turns: i32,
    },
}

fn move_by(map: &Map, objects: &mut [Object], id: usize, dx: i32, dy: i32) {
    // move by the given amount
    let (x, y) = objects[id].pos();
    if !is_blocked(map, objects, x + dx, y + dy) {
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
            move_by(&game_state.map, objects, PLAYER, dx, dy);
        }
    }
}

fn move_towards(map: &Map, objects: &mut [Object], id: usize, target_x: i32, target_y: i32) {
    // vector from this object to the target, and distance
    let dx = target_x - objects[id].x;
    let dy = target_y - objects[id].y;
    let distance = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();

    // normalize it to length 1 (preserving direction), then round it and
    // convert to integer so the movement is restricted to the map grid
    let dx = (dx as f32 / distance).round() as i32;
    let dy = (dy as f32 / distance).round() as i32;
    move_by(map, objects, id, dx, dy);
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
            move_towards(&game_state.map, objects, monster_id, player_x, player_y);
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
            &game_state.map,
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

// tiles, map and rooms

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct Tile {
    blocked: bool,
    block_sight: bool,
    explored: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
            explored: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            block_sight: true,
            explored: false,
        }
    }
}

type Map = Vec<Vec<Tile>>;

fn make_map(objects: &mut Vec<Object>, level: u32) -> Map {
    // fill the map with `unblocked` tiles
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];

    // PLayer is the first element, remove everything else.
    // NOTE: works only if player is the first object!
    assert_eq!(&objects[PLAYER] as *const _, &objects[0] as *const _);
    objects.truncate(1);

    // create rooms randomly
    let mut rooms = vec![];

    for _ in 0..MAX_ROOMS {
        // random width and height
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);

        // random position without exceeding the boundaries of the map
        let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
        let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);

        // create room and store in vector
        let new_room = Rect::new(x, y, w, h);
        let failed = rooms
            .iter()
            .any(|other_room| new_room.intersects_with(other_room));

        if !failed {
            // no intersections, we have a valid room.
            create_room(&mut map, new_room);

            // add some content to the room
            place_objects(&map, objects, new_room, level);

            let (new_x, new_y) = new_room.center();
            if rooms.is_empty() {
                // this is the first room, save position as starting point for the player
                objects[PLAYER].set_pos(new_x, new_y);
            } else {
                // all rooms after the first:
                // connect it to the previous room with a tunnel

                // center coordinates of the previous room
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                // connect both rooms with a horizontal and a vertical tunnel - in random order
                if rand::random() {
                    // move horizontally, then vertically
                    create_h_tunnel(&mut map, prev_x, new_x, prev_y);
                    create_v_tunnel(&mut map, prev_y, new_y, new_x);
                } else {
                    // move vertically, then horizontally
                    create_v_tunnel(&mut map, prev_y, new_y, prev_x);
                    create_h_tunnel(&mut map, prev_x, new_x, new_y);
                }
            }
            // finally, append new room to list
            rooms.push(new_room);
        }
    }

    // create stairs at the center of the last room
    let (last_room_x, last_room_y) = rooms[rooms.len() - 1].center();
    let mut stairs = Object::new(
        last_room_x,
        last_room_y,
        "stairs",
        false,
        '<',
        colors::WHITE,
    );
    stairs.always_visible = true;
    objects.push(stairs);

    map
}

/// Advance to the next level
fn next_level(tcod: &mut Tcod, objects: &mut Vec<Object>, game_state: &mut GameState) {
    game_state.log.add(
        "You take a moment to rest, and recover your strength.",
        colors::VIOLET,
    );
    let heal_hp = objects[PLAYER].fighter.map_or(0, |f| f.max_hp / 2);
    objects[PLAYER].heal(heal_hp);

    game_state.log.add(
        "After a rare moment of peace, you descend deeper into the heart of the dungeon...",
        colors::RED,
    );
    game_state.dungeon_level += 1;
    game_state.map = make_map(objects, game_state.dungeon_level);
    initialise_fov(&game_state.map, tcod);
}

struct Transition {
    level: u32,
    value: u32,
}

/// Return a value that depends on level.
/// The table specifies what value occurs after each level, default is 0.
fn from_dungeon_level(table: &[Transition], level: u32) -> u32 {
    table
        .iter()
        .rev()
        .find(|transition| level >= transition.level)
        .map_or(0, |transition| transition.value)
}

// data structures for room generation
#[derive(Clone, Copy, Debug)]
struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }

    pub fn center(&self) -> (i32, i32) {
        let center_x = (self.x1 + self.x2) / 2;
        let center_y = (self.y1 + self.y2) / 2;
        (center_x, center_y)
    }

    /// Return true if this rect intersects with another one.
    pub fn intersects_with(&self, other: &Rect) -> bool {
        (self.x1 <= other.x2)
            && (self.x2 >= other.x1)
            && (self.y1 <= other.y2)
            && (self.y2 >= other.y1)
    }
}

fn create_room(map: &mut Map, room: Rect) {
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

fn create_h_tunnel(map: &mut Map, x1: i32, x2: i32, y: i32) {
    for x in cmp::min(x1, x2)..=cmp::max(x1, x2) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn create_v_tunnel(map: &mut Map, y1: i32, y2: i32, x: i32) {
    for y in cmp::min(y1, y2)..=cmp::max(y1, y2) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn place_objects(map: &Map, objects: &mut Vec<Object>, room: Rect, level: u32) {
    use rand::distributions::WeightedIndex;
    use rand::prelude::*;

    let max_monsters = from_dungeon_level(
        &[
            Transition { level: 1, value: 2 },
            Transition { level: 4, value: 3 },
            Transition { level: 6, value: 5 },
        ],
        level,
    );

    // monster random table
    let troll_chance = from_dungeon_level(
        &[
            Transition {
                level: 3,
                value: 15,
            },
            Transition {
                level: 5,
                value: 30,
            },
            Transition {
                level: 7,
                value: 60,
            },
        ],
        level,
    );

    let monster_chances = [("orc", 80), ("troll", troll_chance)];
    let monster_dist = WeightedIndex::new(monster_chances.iter().map(|item| item.1)).unwrap();

    // choose random number of monsters
    let num_monsters = rand::thread_rng().gen_range(0, max_monsters + 1);
    for _ in 0..num_monsters {
        // choose random spot for this monster
        let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
        let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);

        if !is_blocked(map, objects, x, y) {
            let mut monster = match monster_chances[monster_dist.sample(&mut rand::thread_rng())].0
            {
                "orc" => {
                    let mut orc = Object::new(x, y, "orc", true, 'o', colors::DESATURATED_GREEN);
                    orc.fighter = Some(Fighter {
                        max_hp: 10,
                        hp: 10,
                        defense: 0,
                        power: 3,
                        on_death: DeathCallback::Monster,
                        xp: 35,
                    });
                    orc.ai = Some(Ai::Basic);
                    orc
                }
                "troll" => {
                    let mut troll = Object::new(x, y, "troll", true, 'T', colors::DARKER_GREEN);
                    troll.fighter = Some(Fighter {
                        max_hp: 16,
                        hp: 16,
                        defense: 1,
                        power: 4,
                        on_death: DeathCallback::Monster,
                        xp: 100,
                    });
                    troll.ai = Some(Ai::Basic);
                    troll
                }
                _ => unreachable!(),
            };

            monster.alive = true;
            objects.push(monster);
        }
    }

    // maximum number of items per room
    let max_items = from_dungeon_level(
        &[
            Transition { level: 1, value: 1 },
            Transition { level: 4, value: 2 },
        ],
        level,
    );

    // item random table
    let item_chances = &mut [
        // healing potion always shows up, even if all other items have 0 chance
        (Item::Heal, 35),
        (
            Item::Lightning,
            from_dungeon_level(
                &[Transition {
                    level: 4,
                    value: 25,
                }],
                level,
            ),
        ),
        (
            Item::Fireball,
            from_dungeon_level(
                &[Transition {
                    level: 6,
                    value: 25,
                }],
                level,
            ),
        ),
        (
            Item::Confuse,
            from_dungeon_level(
                &[Transition {
                    level: 4,
                    value: 25,
                }],
                level,
            ),
        ),
        (Item::Equipment, 1000),
    ];
    let item_dist = WeightedIndex::new(item_chances.iter().map(|item| item.1)).unwrap();

    // choose random number of items
    let num_items = rand::thread_rng().gen_range(0, max_items + 1);
    for _ in 0..num_items {
        // choose random spot for this item
        let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
        let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);

        // only place it if the tile is not blocked
        if !is_blocked(map, objects, x, y) {
            let mut item = match item_chances[item_dist.sample(&mut rand::thread_rng())].0 {
                Item::Heal => {
                    // create healing potion
                    let mut object =
                        Object::new(x, y, "healing potion", false, '!', colors::VIOLET);
                    object.item = Some(Item::Heal);
                    object
                }
                Item::Lightning => {
                    // create lightning bolt scroll
                    let mut object = Object::new(
                        x,
                        y,
                        "scroll of lightning bolt",
                        false,
                        '#',
                        colors::LIGHT_YELLOW,
                    );
                    object.item = Some(Item::Lightning);
                    object
                }
                Item::Fireball => {
                    // create a fireball scroll
                    let mut object =
                        Object::new(x, y, "scroll of fireball", false, '#', colors::LIGHT_YELLOW);
                    object.item = Some(Item::Fireball);
                    object
                }
                Item::Confuse => {
                    // create a confuse scroll
                    let mut object = Object::new(
                        x,
                        y,
                        "scroll of confusion",
                        false,
                        '#',
                        colors::LIGHT_YELLOW,
                    );
                    object.item = Some(Item::Confuse);
                    object
                }
                Item::Equipment => {
                    // create a sword
                    let mut object = Object::new(x, y, "sword", false, '/', colors::SKY);
                    object.item = Some(Item::Equipment);
                    object.equipment = Some(Equipment {
                        equipped: false,
                        slot: Slot::RightHand,
                        power_bonus: 0,
                    });
                    object
                }
                _ => unreachable!(),
            };
            item.always_visible = true;
            objects.push(item);
        }
    }
}

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

fn is_blocked(map: &Map, objects: &[Object], x: i32, y: i32) -> bool {
    // first test the map tile
    if map[x as usize][y as usize].blocked {
        return true;
    }
    // now check for any blocking objects
    objects
        .iter()
        .any(|object| object.blocks && object.pos() == (x, y))
}

#[allow(clippy::too_many_arguments)]
fn render_bar(
    panel: &mut Offscreen,
    x: i32,
    y: i32,
    total_width: i32,
    name: &str,
    value: i32,
    maximum: i32,
    bar_color: Color,
    back_color: Color,
) {
    // render a bar (HP, EXP, etc)
    let bar_width = (value as f32 / maximum as f32 * total_width as f32) as i32;

    // render background first
    panel.set_default_background(back_color);
    panel.rect(x, y, total_width, 1, false, BackgroundFlag::Screen);

    // now render bar on top
    panel.set_default_background(bar_color);
    if bar_width > 0 {
        panel.rect(x, y, bar_width, 1, false, BackgroundFlag::Screen);
    }

    // finally some centered text with the values
    panel.set_default_foreground(colors::WHITE);
    panel.print_ex(
        x + total_width / 2,
        y,
        BackgroundFlag::None,
        TextAlignment::Center,
        &format!("{}: {}/{}", name, value, maximum),
    );
}

trait MessageLog {
    fn add<T: Into<String>>(&mut self, message: T, color: Color);
}

impl MessageLog for Vec<(String, Color)> {
    fn add<T: Into<String>>(&mut self, message: T, color: Color) {
        self.push((message.into(), color));
    }
}

/// Render all objects and tiles.
fn render_all(
    tcod: &mut Tcod,
    game_state: &mut GameState,
    objects: &[Object],
    fov_recompute: bool,
) {
    if fov_recompute {
        // recompute fov if needed (the player moved or something)
        let player = &objects[PLAYER];
        tcod.fov
            .compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALG);
    }

    // go through all tiles and set their background color
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let visible = tcod.fov.is_in_fov(x, y);
            let wall = game_state.map[x as usize][y as usize].block_sight;
            let tile_color = match (visible, wall) {
                // outside field of view:
                (false, true) => COLOR_DARK_WALL,
                (false, false) => COLOR_DARK_GROUND,
                // inside fov:
                (true, true) => COLOR_LIGHT_WALL,
                (true, false) => COLOR_LIGHT_GROUND,
            };

            let explored = &mut game_state.map[x as usize][y as usize].explored;
            if visible {
                *explored = true;
            }
            if *explored {
                // show explored tiles only (any visible tile is explored already)
                tcod.con
                    .set_char_background(x, y, tile_color, BackgroundFlag::Set);
            }
        }
    }

    let mut to_draw: Vec<_> = objects
        .iter()
        .filter(|o| {
            tcod.fov.is_in_fov(o.x, o.y)
                || (o.always_visible && game_state.map[o.x as usize][o.y as usize].explored)
        })
        .collect();
    // sort, so that non-blocking objects com first
    to_draw.sort_by(|o1, o2| o1.blocks.cmp(&o2.blocks));
    // draw the objects in the list
    for object in &to_draw {
        object.draw(&mut tcod.con);
    }

    // prepare to render the GUI panel
    tcod.panel.set_default_background(colors::BLACK);
    tcod.panel.clear();

    // show player's stats
    let hp = objects[PLAYER].fighter.map_or(0, |f| f.hp);
    let max_hp = objects[PLAYER].fighter.map_or(0, |f| f.max_hp);
    render_bar(
        &mut tcod.panel,
        1,
        1,
        BAR_WIDTH,
        "HP",
        hp,
        max_hp,
        colors::LIGHT_RED,
        colors::DARKER_RED,
    );
    tcod.panel.print_ex(
        1,
        2,
        BackgroundFlag::None,
        TextAlignment::Left,
        format!("Dungeon level: {}", game_state.dungeon_level),
    );

    // show names of objects under the mouse
    tcod.panel.set_default_foreground(colors::LIGHT_GREY);
    tcod.panel.print_ex(
        1,
        0,
        BackgroundFlag::None,
        TextAlignment::Left,
        get_names_under_mouse(tcod.mouse, objects, &tcod.fov),
    );

    // print game messages, one line at a time
    let mut y = MSG_HEIGHT as i32;
    for &(ref msg, color) in &mut game_state.log.iter().rev() {
        let msg_height = tcod.panel.get_height_rect(MSG_X, y, MSG_WIDTH, 0, msg);
        y -= msg_height;
        if y < 0 {
            break;
        }
        tcod.panel.set_default_foreground(color);
        tcod.panel.print_rect(MSG_X, y, MSG_WIDTH, 0, msg);
    }

    // blit contents of `tcod.panel` to the root console
    blit(
        &tcod.panel,
        (0, 0),
        (SCREEN_WIDTH, SCREEN_HEIGHT),
        &mut tcod.root,
        (0, PANEL_Y),
        1.0,
        1.0,
    );

    // blit contents of offscreen console to root console and present it
    blit(
        &tcod.con,
        (0, 0),
        (MAP_WIDTH, MAP_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0,
    );
}

/// Handle user input
fn handle_keys(
    tcod: &mut Tcod,
    game_state: &mut GameState,
    objects: &mut Vec<Object>,
    key: Key,
) -> PlayerAction {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;
    use PlayerAction::*;

    let player_alive = objects[PLAYER].alive;
    match (key, player_alive) {
        // toggle fullscreen
        (
            Key {
                code: Enter,
                alt: true,
                ..
            },
            _,
        ) => {
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
            DidntTakeTurn
        }

        // exit game
        (Key { code: Escape, .. }, _) => Exit,

        // handle movement
        (Key { code: Up, .. }, true) | (Key { printable: 'w', .. }, true) => {
            player_move_or_attack(game_state, objects, 0, -1);
            TookTurn
        }
        (Key { code: Down, .. }, true) | (Key { printable: 's', .. }, true) => {
            player_move_or_attack(game_state, objects, 0, 1);
            TookTurn
        }
        (Key { code: Left, .. }, true) | (Key { printable: 'a', .. }, true) => {
            player_move_or_attack(game_state, objects, -1, 0);
            TookTurn
        }
        (Key { code: Right, .. }, true) | (Key { printable: 'd', .. }, true) => {
            player_move_or_attack(game_state, objects, 1, 0);
            TookTurn
        }
        (Key { printable: 'x', .. }, true) => {
            // do nothing, i.e. wait for the monster to come to you
            TookTurn
        }
        (Key { printable: 'g', .. }, true) => {
            // pick up an item
            let item_id = objects
                .iter()
                .position(|object| object.pos() == objects[PLAYER].pos() && object.item.is_some());
            if let Some(item_id) = item_id {
                pick_item_up(game_state, objects, item_id);
            }
            DidntTakeTurn
        }
        (Key { printable: 'i', .. }, true) => {
            // show the inventory: if an item is selected, use it
            let inventory_index = inventory_menu(
                &mut tcod.root,
                &game_state.inventory,
                "Press the key next to an item to use it, or any other to cancel.\n",
            );
            if let Some(inventory_index) = inventory_index {
                use_item(tcod, game_state, objects, inventory_index);
            }
            DidntTakeTurn
        }
        (Key { printable: 'd', .. }, true) => {
            // show_inventory; if an item is selected, drop it
            let inventory_index = inventory_menu(
                &mut tcod.root,
                &game_state.inventory,
                "Press the key enxt to an item to drop it, or any other to cancel.\n",
            );
            if let Some(inventory_index) = inventory_index {
                drop_item(game_state, objects, inventory_index);
            }
            DidntTakeTurn
        }
        (Key { printable: 'e', .. }, true) => {
            // go down the stairs, if the player is on them
            println!("trying to go down stairs");
            let player_on_stairs = objects
                .iter()
                .any(|object| object.pos() == objects[PLAYER].pos() && object.name == "stairs");
            if player_on_stairs {
                next_level(tcod, objects, game_state);
            }
            DidntTakeTurn
        }
        (Key { printable: 'c', .. }, true) => {
            // show character information
            let player = &objects[PLAYER];
            let level = player.level;
            let level_up_xp = LEVEL_UP_BASE + player.level * LEVEL_UP_FACTOR;
            if let Some(fighter) = player.fighter.as_ref() {
                let msg = format!(
                    "Character information

                    Level: {}
                    Experience: {}
                    Experience to level up: {}

                    Maximum HP: {}
                    Attack: {}
                    Defense: {}",
                    level, fighter.xp, level_up_xp, fighter.max_hp, fighter.power, fighter.defense
                );
                msgbox(&msg, CHARACTER_SCREEN_WIDTH, &mut tcod.root);
            }

            DidntTakeTurn
        }

        _ => DidntTakeTurn,
    }
}

fn get_names_under_mouse(mouse: Mouse, objects: &[Object], fov_map: &FovMap) -> String {
    let (x, y) = (mouse.cx as i32, mouse.cy as i32);

    // create a list with the names of all objects at the mouse's coordinates and in FOV
    let names = objects
        .iter()
        .filter(|obj| obj.pos() == (x, y) && fov_map.is_in_fov(obj.x, obj.y))
        .map(|obj| obj.name.clone())
        .collect::<Vec<_>>();

    names.join(", ") // return names separated by commas
}

/// return the position of a tile left-clicked in player's FOV (optionally in a range),
/// or (None, None) if right-clicked.
fn target_tile(
    tcod: &mut Tcod,
    game_state: &mut GameState,
    objects: &[Object],
    max_range: Option<f32>,
) -> Option<(i32, i32)> {
    use tcod::input::KeyCode::Escape;
    loop {
        // render the screen. this erases the inventory and shows the names of objects under the mouse
        tcod.root.flush();
        let event = input::check_for_event(input::KEY_PRESS | input::MOUSE).map(|e| e.1);
        let mut key = None;
        match event {
            Some(Event::Mouse(m)) => tcod.mouse = m,
            Some(Event::Key(k)) => key = Some(k),
            None => {}
        }
        render_all(tcod, game_state, objects, false);

        let (x, y) = (tcod.mouse.cx as i32, tcod.mouse.cy as i32);

        // accept the target if the player clicked in FOV, and in case a range is specified, if it's in that range
        let in_fov = (x < MAP_WIDTH) && (y < MAP_HEIGHT) && tcod.fov.is_in_fov(x, y);
        let in_range = max_range.map_or(true, |range| objects[PLAYER].distance(x, y) <= range);

        if tcod.mouse.lbutton_pressed && in_fov && in_range {
            return Some((x, y));
        }

        let escape = key.map_or(false, |k| k.code == Escape);
        if tcod.mouse.rbutton_pressed || escape {
            return None; // cancel if the player right-clicked or pressed Escape
        }
    }
}

fn target_monster(
    tcod: &mut Tcod,
    game_state: &mut GameState,
    objects: &[Object],
    max_range: Option<f32>,
) -> Option<usize> {
    loop {
        match target_tile(tcod, game_state, objects, max_range) {
            Some((x, y)) => {
                // return the first clicked monster, otherwise continue looping
                for (id, obj) in objects.iter().enumerate() {
                    if obj.pos() == (x, y) && obj.fighter.is_some() && id != PLAYER {
                        return Some(id);
                    }
                }
            }
            None => return None,
        }
    }
}

fn menu<T: AsRef<str>>(header: &str, options: &[T], width: i32, root: &mut Root) -> Option<usize> {
    assert!(
        options.len() <= 26,
        "Cannot have a mnu with more than 26 options."
    );

    // calculate total height for the header (after auto-wrap) and one line per option
    let header_height = if header.is_empty() {
        0
    } else {
        root.get_height_rect(0, 0, width, SCREEN_HEIGHT, header)
    };

    let height = options.len() as i32 + header_height;

    // create an off-screen console that represents the menu's window
    let mut window = Offscreen::new(width, height);

    // print the header, with auto-wrap
    window.set_default_foreground(colors::WHITE);
    window.print_rect_ex(
        0,
        0,
        width,
        height,
        BackgroundFlag::None,
        TextAlignment::Left,
        header,
    );

    // print all the options
    for (index, option_text) in options.iter().enumerate() {
        let menu_letter = (b'a' + index as u8) as char;
        let text = format!("({}) {}", menu_letter, option_text.as_ref());
        window.print_ex(
            0,
            header_height + index as i32,
            BackgroundFlag::None,
            TextAlignment::Left,
            text,
        );
    }

    // blit contents of "window" to the root console
    let x = SCREEN_WIDTH / 2 - width / 2;
    let y = SCREEN_HEIGHT / 2 - height / 2;
    tcod::console::blit(&window, (0, 0), (width, height), root, (x, y), 1.0, 0.7);

    // present the root console to the player and wait for a key-press
    root.flush();
    let key = root.wait_for_keypress(true);

    // convert the ASCII code to and index; if if corresponds to an option, return it
    if key.printable.is_alphabetic() {
        let index = key.printable.to_ascii_lowercase() as usize - 'a' as usize;
        if index < options.len() {
            Some(index)
        } else {
            None
        }
    } else {
        None
    }
}

fn msgbox(text: &str, width: i32, root: &mut Root) {
    let options: &[&str] = &[];
    menu(text, options, width, root);
}

fn inventory_menu(root: &mut Root, inventory: &[Object], header: &str) -> Option<usize> {
    // how a menu with each item of the inventory as an option
    let options = if inventory.is_empty() {
        vec!["Inventory is empty.".into()]
    } else {
        // inventory.iter().map(|item| item.name.clone()).collect()
        inventory
            .iter()
            .map(|item| {
                // show additional information, in case it's equipped
                match item.equipment {
                    Some(equipment) if equipment.equipped => {
                        format!("{} (on {})", item.name, equipment.slot)
                    }
                    _ => item.name.clone(),
                }
            })
            .collect()
    };

    let inventory_index = menu(header, &options, INVENTORY_WIDTH, root);

    // if an item was chosen, return if
    if !inventory.is_empty() {
        inventory_index
    } else {
        None
    }
}

fn new_game(tcod: &mut Tcod) -> (Vec<Object>, GameState) {
    // create object representing the player
    let mut player = Object::new(0, 0, "player", true, '@', colors::WHITE);
    player.alive = true;
    player.fighter = Some(Fighter {
        max_hp: 30,
        hp: 30,
        defense: 2,
        power: 5,
        on_death: DeathCallback::Player,
        xp: 0,
    });

    // create array holding all objects
    let mut objects = vec![player];
    let level = 1;

    // create game state holding most game-relevant information
    //  - also creates map and player starting position
    let mut game_state = GameState {
        // generate map (at this point it's not drawn on screen)
        map: make_map(&mut objects, level),
        // create the list of game messages and their colors, starts empty
        log: vec![],
        inventory: vec![],
        dungeon_level: 1,
    };

    initialise_fov(&game_state.map, tcod);

    // a warm welcoming message
    game_state.log.add(
        "Welcome stranger! prepare to perish in the Tombs of the Ancient Kings.",
        colors::RED,
    );

    (objects, game_state)
}

fn initialise_fov(map: &Map, tcod: &mut Tcod) {
    // init fov map
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            tcod.fov.set(
                x,
                y,
                !map[x as usize][y as usize].block_sight,
                !map[x as usize][y as usize].blocked,
            );
        }
    }
    tcod.con.clear(); // unexplored areas start black (which is the default background color)
}

fn game_loop(objects: &mut Vec<Object>, game_state: &mut GameState, tcod: &mut Tcod) {
    // force FOV "recompute" first time through the game loop
    let mut previous_player_position = (-1, -1);

    // input processing
    let mut key: Key = Default::default();

    while !tcod.root.window_closed() {
        // clear the screen of the previous frame
        tcod.con.clear();

        // check for input events
        match input::check_for_event(input::MOUSE | input::KEY_PRESS) {
            Some((_, Event::Mouse(m))) => tcod.mouse = m,
            Some((_, Event::Key(k))) => key = k,
            _ => key = Default::default(),
        }

        // render objects and map
        let fov_recompute = previous_player_position != (objects[PLAYER].x, objects[PLAYER].y);
        render_all(tcod, game_state, &objects, fov_recompute);

        // draw everything on the window at once
        tcod.root.flush();

        // level up if needed
        level_up(objects, game_state, tcod);

        // handle keys and exit game if needed
        previous_player_position = objects[PLAYER].pos();
        let player_action = handle_keys(tcod, game_state, objects, key);
        if player_action == PlayerAction::Exit {
            save_game(objects, game_state).unwrap();
            break;
        }

        // let monsters take their turn
        if objects[PLAYER].alive && player_action != PlayerAction::DidntTakeTurn {
            for id in 0..objects.len() {
                if objects[id].ai.is_some() {
                    ai_take_turn(game_state, objects, &tcod.fov, id);
                }
            }
        }
    }
}

fn main_menu(tcod: &mut Tcod) {
    let img = tcod::image::Image::from_file("menu_background.png")
        .ok()
        .expect("Background image not found");

    while !tcod.root.window_closed() {
        // show the background image, at twice the regular console resolution
        tcod::image::blit_2x(&img, (0, 0), (-1, -1), &mut tcod.root, (0, 0));

        tcod.root.set_default_foreground(colors::LIGHT_YELLOW);
        tcod.root.print_ex(
            SCREEN_WIDTH / 2,
            SCREEN_HEIGHT / 2 - 4,
            BackgroundFlag::None,
            TextAlignment::Center,
            "TOMBS OF THE ANCIENT KINGS",
        );
        tcod.root.print_ex(
            SCREEN_WIDTH / 2,
            SCREEN_HEIGHT - 2,
            BackgroundFlag::None,
            TextAlignment::Center,
            "By Michael Wagner",
        );

        // show options and wait for the player's choice
        let choices = &["Play a new game", "Continue last game", "Quit"];
        let choice = menu("", choices, 24, &mut tcod.root);

        match choice {
            Some(0) => {
                // start new game
                let (mut objects, mut game_state) = new_game(tcod);
                game_loop(&mut objects, &mut game_state, tcod);
            }
            Some(1) => {
                // load game from file
                match load_game() {
                    Ok((mut objects, mut game_state)) => {
                        initialise_fov(&game_state.map, tcod);
                        game_loop(&mut objects, &mut game_state, tcod);
                    }
                    Err(_e) => {
                        msgbox("\nNo saved game to load\n", 24, &mut tcod.root);
                        continue;
                    }
                }
            }
            Some(2) => {
                //quit
                break;
            }
            _ => {}
        }
    }
}

fn save_game(objects: &[Object], game_state: &GameState) -> Result<(), Box<Error>> {
    let save_data = serde_json::to_string(&(objects, game_state))?;
    let mut file = File::create("savegame")?;
    file.write_all(save_data.as_bytes())?;
    Ok(())
}

fn load_game() -> Result<(Vec<Object>, GameState), Box<Error>> {
    let mut json_save_state = String::new();
    let mut file = File::open("savegame")?;
    file.read_to_string(&mut json_save_state)?;
    let result = serde_json::from_str::<(Vec<Object>, GameState)>(&json_save_state)?;
    Ok(result)
}

fn main() {
    let root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("roguelike")
        .init();

    tcod::system::set_fps(LIMIT_FPS);

    let mut tcod = Tcod {
        root: root,
        con: Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT),
        panel: Offscreen::new(SCREEN_WIDTH, PANEL_HEIGHT),
        fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
        mouse: Default::default(),
    };

    main_menu(&mut tcod);
}
