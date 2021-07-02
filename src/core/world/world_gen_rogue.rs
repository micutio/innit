use crate::core::game_state::GameState;
use crate::core::position::Position;
use crate::core::world::{Tile, WorldGen};
use crate::core::{game_objects::GameObjects, innit_env};
use crate::entity::object::Object;
use crate::game::{RunState, WORLD_HEIGHT, WORLD_WIDTH};
use crate::raws::object_template::ObjectTemplate;
use crate::raws::spawn::Spawn;
use crate::ui::menu::main_menu::main_menu;
use crate::ui::palette;
use rand::Rng;
use std::{cmp, thread, time};

// room generation constraints
const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

/// Module World Generator Rogue-style
///
/// This world generator is based on the genre-defining game `Rogue`.
pub struct RogueWorldGenerator {
    rooms: Vec<Rect>,
    player_start: (i32, i32),
}

impl RogueWorldGenerator {
    pub fn _new() -> Self {
        RogueWorldGenerator {
            rooms: vec![],
            player_start: (0, 0),
        }
    }
}

impl WorldGen for RogueWorldGenerator {
    fn make_world(
        &mut self,
        state: &mut GameState,
        objects: &mut GameObjects,
        spawns: &[Spawn],
        object_templates: &[ObjectTemplate],
        level: u32,
    ) -> RunState {
        // fill the world with `unblocked` tiles
        // create rooms randomly

        for _ in 0..MAX_ROOMS {
            // random width and height
            let w = state.rng.gen_range(ROOM_MIN_SIZE..=ROOM_MAX_SIZE);
            let h = state.rng.gen_range(ROOM_MIN_SIZE..=ROOM_MAX_SIZE);

            // random position without exceeding the boundaries of the map
            let x = state.rng.gen_range(0..WORLD_WIDTH - w);
            let y = state.rng.gen_range(0..WORLD_HEIGHT - h);

            // create room and store in vector
            let new_room = Rect::new(x, y, w, h);
            let failed = self
                .rooms
                .iter()
                .any(|other_room| new_room.intersects_with(other_room));

            if !failed {
                // no intersections, we have a valid room.
                create_room(objects, new_room);

                // add some content to the room
                // TODO: fix
                place_objects(state, objects, spawns, object_templates, level);

                let (new_x, new_y) = new_room.center();
                if self.rooms.is_empty() {
                    // this is the first room, save position as starting point for the player
                    debug!("setting new player start position: ({}, {})", new_x, new_y);
                    // player.set_pos(new_x, new_y);
                    self.player_start = (new_x, new_y);
                } else {
                    // all rooms after the first:
                    // connect it to the previous room with a tunnel

                    // center coordinates of the previous room
                    let (prev_x, prev_y) = self.rooms[self.rooms.len() - 1].center();

                    // connect both rooms with a horizontal and a vertical tunnel - in random order
                    if rand::random() {
                        // move horizontally, then vertically
                        create_h_tunnel(objects, prev_x, new_x, prev_y);
                        create_v_tunnel(objects, prev_y, new_y, new_x);
                    } else {
                        // move vertically, then horizontally
                        create_v_tunnel(objects, prev_y, new_y, prev_x);
                        create_h_tunnel(objects, prev_x, new_x, new_y);
                    }
                }
                // finally, append new room to list
                self.rooms.push(new_room);
            }

            if innit_env().debug_mode {
                let ten_millis = time::Duration::from_millis(100);
                thread::sleep(ten_millis);
            }
        }
        RunState::MainMenu(main_menu())
    }

    fn get_player_start_pos(&self) -> (i32, i32) {
        self.player_start
    }
}

fn create_room(objects: &mut GameObjects, room: Rect) {
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            objects
                .get_tile_at(x as usize, y as usize)
                .replace(Tile::empty(x, y, innit_env().debug_mode));
        }
    }
}

fn create_h_tunnel(objects: &mut GameObjects, x1: i32, x2: i32, y: i32) {
    for x in cmp::min(x1, x2)..=cmp::max(x1, x2) {
        objects
            .get_tile_at(x as usize, y as usize)
            .replace(Tile::empty(x, y, innit_env().debug_mode));
    }
}

fn create_v_tunnel(objects: &mut GameObjects, y1: i32, y2: i32, x: i32) {
    for y in cmp::min(y1, y2)..=cmp::max(y1, y2) {
        objects
            .get_tile_at(x as usize, y as usize)
            .replace(Tile::empty(x, y, innit_env().debug_mode));
    }
}

fn place_objects(
    state: &mut GameState,
    objects: &mut GameObjects,
    _spawns: &[Spawn],
    _object_templates: &[ObjectTemplate],
    _level: u32,
) {
    // use rand::distributions::WeightedIndex;
    use rand::prelude::*;

    // TODO: Pull spawn tables out of here and pass as parameters in make_world().
    // TODO: Set monster number per level via transitions.
    let max_monsters = 100;

    // let monster_chances: Vec<(&String, u32)> = spawns
    //     .iter()
    //     .map(|s| (&s.npc, from_dungeon_level(&s.spawn_transitions, level)))
    //     .collect();

    // let monster_dist = WeightedIndex::new(monster_chances.iter().map(|item| item.1)).unwrap();

    // choose random number of monsters
    let num_monsters = state.rng.gen_range(0..max_monsters);
    for _ in 0..num_monsters {
        // choose random spot for this monster
        let x = state.rng.gen_range(0 + 1..WORLD_WIDTH);
        let y = state.rng.gen_range(0 + 1..WORLD_HEIGHT);

        if !objects.is_pos_occupied(&Position::new(x, y)) {
            // let monster_type = monster_chances[monster_dist.sample(&mut state.rng)].0;
            let monster = Object::new()
                .position(x, y)
                .living(true)
                .visualize("Virus", 'v', palette().entity_virus)
                .physical(true, false, false)
                // .genome(
                //     0.75,
                //     state
                //         .gene_library
                //         .new_genetics(&mut state.rng, DnaType::Rna, true, GENE_LEN),
                // )
                // .control(Controller::Npc(Box::new(AiVirus::new())));
                ;
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
