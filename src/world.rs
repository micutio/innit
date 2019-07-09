/// Module World
///
/// The world contains all structures and methods for terrain/dungeon generation
use rand::Rng;
use std::cmp;
use tcod::colors;

// internal modules
use entity::action::AttackAction;
use entity::ai::Ai;
use entity::fighter::{DeathCallback, Fighter};
use entity::object::{Object, ObjectVec};
use game_state::{from_dungeon_level, Transition, PLAYER};

// world constraints
pub const WORLD_WIDTH: i32 = 80;
pub const WORLD_HEIGHT: i32 = 43;
// room generation constraints
const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

// TODO: Move tile properties into phyics object in Object declaration
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub blocked: bool,
    pub block_sight: bool,
    pub explored: bool,
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

pub type World = Vec<Vec<Tile>>;

pub fn make_world(objects: &mut ObjectVec, level: u32) -> World {
    // fill the world with `unblocked` tiles
    let mut world = vec![vec![Tile::wall(); WORLD_HEIGHT as usize]; WORLD_WIDTH as usize];

    // PLayer is the first element, remove everything else.
    // NOTE: works only if player is the first object!
    assert_eq!(&objects[PLAYER] as *const _, &objects[0] as *const _);
    objects.get_vector().truncate(1);

    // create rooms randomly
    let mut rooms = vec![];

    for _ in 0..MAX_ROOMS {
        // random width and height
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);

        // random position without exceeding the boundaries of the map
        let x = rand::thread_rng().gen_range(0, WORLD_WIDTH - w);
        let y = rand::thread_rng().gen_range(0, WORLD_HEIGHT - h);

        // create room and store in vector
        let new_room = Rect::new(x, y, w, h);
        let failed = rooms
            .iter()
            .any(|other_room| new_room.intersects_with(other_room));

        if !failed {
            // no intersections, we have a valid room.
            create_room(&mut world, new_room);

            // add some content to the room
            place_objects(&world, objects, new_room, level);

            let (new_x, new_y) = new_room.center();
            if rooms.is_empty() {
                // this is the first room, save position as starting point for the player
                objects[PLAYER].unwrap().set_pos(new_x, new_y);
            } else {
                // all rooms after the first:
                // connect it to the previous room with a tunnel

                // center coordinates of the previous room
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                // connect both rooms with a horizontal and a vertical tunnel - in random order
                if rand::random() {
                    // move horizontally, then vertically
                    create_h_tunnel(&mut world, prev_x, new_x, prev_y);
                    create_v_tunnel(&mut world, prev_y, new_y, new_x);
                } else {
                    // move vertically, then horizontally
                    create_v_tunnel(&mut world, prev_y, new_y, prev_x);
                    create_h_tunnel(&mut world, prev_x, new_x, new_y);
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

    world
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

fn create_room(world: &mut World, room: Rect) {
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            world[x as usize][y as usize] = Tile::empty();
        }
    }
}

fn create_h_tunnel(world: &mut World, x1: i32, x2: i32, y: i32) {
    for x in cmp::min(x1, x2)..=cmp::max(x1, x2) {
        world[x as usize][y as usize] = Tile::empty();
    }
}

fn create_v_tunnel(world: &mut World, y1: i32, y2: i32, x: i32) {
    for y in cmp::min(y1, y2)..=cmp::max(y1, y2) {
        world[x as usize][y as usize] = Tile::empty();
    }
}

fn place_objects(world: &World, objects: &mut ObjectVec, room: Rect, level: u32) {
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
    let bacteria_chance = from_dungeon_level(
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

    let monster_chances = [("virus", 80), ("bacteria", bacteria_chance)];
    let monster_dist = WeightedIndex::new(monster_chances.iter().map(|item| item.1)).unwrap();

    // choose random number of monsters
    let num_monsters = rand::thread_rng().gen_range(0, max_monsters + 1);
    for _ in 0..num_monsters {
        // choose random spot for this monster
        let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
        let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);

        if !is_blocked(world, objects, x, y) {
            let mut monster = match monster_chances[monster_dist.sample(&mut rand::thread_rng())].0
            {
                "virus" => {
                    let mut virus =
                        Object::new(x, y, "virus", true, 'v', colors::DESATURATED_GREEN);
                    virus.fighter = Some(Fighter {
                        base_max_hp: 10,
                        hp: 10,
                        base_defense: 0,
                        base_power: 3,
                        on_death: DeathCallback::Monster,
                        xp: 35,
                    });
                    virus.attack_action = Some(AttackAction::new(3, 0));
                    virus.ai = Some(Ai::Basic);
                    virus
                }
                "bacteria" => {
                    let mut bacteria =
                        Object::new(x, y, "bacteria", true, 'b', colors::DARKER_GREEN);
                    bacteria.fighter = Some(Fighter {
                        base_max_hp: 16,
                        hp: 16,
                        base_defense: 1,
                        base_power: 4,
                        on_death: DeathCallback::Monster,
                        xp: 100,
                    });
                    bacteria.attack_action = Some(AttackAction::new(4, 0));
                    bacteria.ai = Some(Ai::Basic);
                    bacteria
                }
                _ => unreachable!(),
            };

            monster.alive = true;
            objects.push(monster);
        }
    }
}

pub fn is_blocked(world: &World, objects: &ObjectVec, x: i32, y: i32) -> bool {
    // first test the world tile
    if world[x as usize][y as usize].blocked {
        return true;
    }
    // now check for any blocking objects
    objects
        .get_vector()
        .iter()
        .any(|Some(object)| object.blocks && object.pos() == (x, y))
}
