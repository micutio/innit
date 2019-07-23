/// Module World Generator Rogue-style
///
/// This world generator is based on the genre-defining game `Rogue`.
/// NOTE: In the future, NPCs should be generated based on a spawning table.
use rand::Rng;
use std::cmp;
use tcod::colors;

use crate::{
    core::{
        game_objects::GameObjects,
        game_state::{from_dungeon_level, Transition, PLAYER},
        world::world_gen::{Tile, WorldGen},
    },
    entity::{
        action::AttackAction,
        ai::Ai,
        fighter::{DeathCallback, Fighter},
        object::Object,
    },
    game::{WORLD_HEIGHT, WORLD_WIDTH},
};

// room generation constraints
const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

pub struct RogueWorldGenerator {
    rooms: Vec<Rect>,
}

impl RogueWorldGenerator {
    pub fn new() -> Self {
        RogueWorldGenerator { rooms: vec![] }
    }
}

impl WorldGen for RogueWorldGenerator {
    fn make_world(&mut self, game_objects: &mut GameObjects, level: u32) {
        // fill the world with `unblocked` tiles
        // create rooms randomly

        for _ in 0..MAX_ROOMS {
            // random width and height
            let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
            let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);

            // random position without exceeding the boundaries of the map
            let x = rand::thread_rng().gen_range(0, WORLD_WIDTH - w);
            let y = rand::thread_rng().gen_range(0, WORLD_HEIGHT - h);

            // create room and store in vector
            let new_room = Rect::new(x, y, w, h);
            let failed = self
                .rooms
                .iter()
                .any(|other_room| new_room.intersects_with(other_room));

            if !failed {
                // no intersections, we have a valid room.
                create_room(game_objects, new_room);

                // add some content to the room
                place_objects(game_objects, new_room, level);

                let (new_x, new_y) = new_room.center();
                if self.rooms.is_empty() {
                    // this is the first room, save position as starting point for the player
                    if let Some(ref mut player) = game_objects[PLAYER] {
                        player.set_pos(new_x, new_y);
                    }
                } else {
                    // all rooms after the first:
                    // connect it to the previous room with a tunnel

                    // center coordinates of the previous room
                    let (prev_x, prev_y) = self.rooms[self.rooms.len() - 1].center();

                    // connect both rooms with a horizontal and a vertical tunnel - in random order
                    if rand::random() {
                        // move horizontally, then vertically
                        create_h_tunnel(game_objects, prev_x, new_x, prev_y);
                        create_v_tunnel(game_objects, prev_y, new_y, new_x);
                    } else {
                        // move vertically, then horizontally
                        create_v_tunnel(game_objects, prev_y, new_y, prev_x);
                        create_h_tunnel(game_objects, prev_x, new_x, new_y);
                    }
                }
                // finally, append new room to list
                self.rooms.push(new_room);
            }
        }
    }
}

fn create_room(objects: &mut GameObjects, room: Rect) {
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            objects
                .get_tile_at(x as usize, y as usize)
                .replace(Tile::empty(x, y));
        }
    }
}

fn create_h_tunnel(objects: &mut GameObjects, x1: i32, x2: i32, y: i32) {
    for x in cmp::min(x1, x2)..=cmp::max(x1, x2) {
        objects
            .get_tile_at(x as usize, y as usize)
            .replace(Tile::empty(x, y));
    }
}

fn create_v_tunnel(objects: &mut GameObjects, y1: i32, y2: i32, x: i32) {
    for y in cmp::min(y1, y2)..=cmp::max(y1, y2) {
        objects
            .get_tile_at(x as usize, y as usize)
            .replace(Tile::empty(x, y));
    }
}

fn place_objects(objects: &mut GameObjects, room: Rect, level: u32) {
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

        if !objects.is_blocked(x, y) {
            let mut monster = match monster_chances[monster_dist.sample(&mut rand::thread_rng())].0
            {
                "virus" => {
                    let mut virus = Object::new(
                        x,
                        y,
                        "virus",
                        'v',
                        colors::DESATURATED_GREEN,
                        true,
                        false,
                        false,
                    );
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
                    let mut bacteria = Object::new(
                        x,
                        y,
                        "bacteria",
                        'b',
                        colors::DARKER_GREEN,
                        true,
                        false,
                        false,
                    );
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
