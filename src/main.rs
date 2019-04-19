extern crate tcod;
extern crate rand;

use std::cmp;
use rand::Rng;
use tcod::console::*;
use tcod::colors::{self, Color};
use tcod::map::{Map as FovMap, FovAlgorithm};

// window size
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
// target fps
const LIMIT_FPS: i32 = 20;

#[derive(Debug)]
struct Object {
    x: i32,
    y: i32,
    character: char,
    color: Color,
}

impl Object {

    pub fn new(x: i32, y: i32, character: char, color: Color) -> Self {
        Object {
            x: x,
            y: y,
            character: character,
            color: color,
        }
    }

    pub fn move_by(&mut self, map: &Map, dx: i32, dy: i32) {
        // move by the given amount
        if !map[(self.x + dx) as usize][(self.y + dy) as usize].blocked {
            self.x += dx;
            self.y += dy;
        }
    }

    /// Set the color and then draw the char that represents this object at its position.
    pub fn draw(&self, con: &mut Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.character, BackgroundFlag::None);
    }

    /// Erase the character that represents this object
    pub fn clear(&self, con: &mut Console) {
        con.put_char(self.x, self.y, ' ', BackgroundFlag::None);
    }

}


// map constraints
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;
const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_LIGHT_WALL: Color = Color { r: 130, g: 110, b: 50 };
const COLOR_DARK_GROUND: Color = Color { r: 50, g: 50, b: 150 };
const COLOR_LIGHT_GROUND: Color = Color { r: 200, g: 180, b: 50 };

#[derive(Clone, Copy, Debug)]
struct Tile {
    blocked: bool,
    block_sight: bool,
}

impl Tile {

    pub fn empty() -> Self {
        Tile { blocked: false, block_sight: false }
    }

    pub fn wall() -> Self {
        Tile { blocked: true, block_sight: true }
    }

}

type Map = Vec<Vec<Tile>>;

fn make_map() -> (Map, (i32, i32)) {
    // fill the map with `unblocked` tiles
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    
    // create rooms randomly
    let mut rooms = vec![];
    let mut starting_position = (0, 0);

    for _ in 0..MAX_ROOMS {
        // random width and height
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        
        // random position without exceeding the boundaries of the map
        let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
        let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);

        // create room and store in vector
        let new_room = Rect::new(x, y, w, h);
        let failed = rooms.iter().any(|other_room| new_room.intersects_with(other_room));

        if !failed {
            // no intersections, we have a valid room.
            create_room(new_room, &mut map);
            let (new_x, new_y) = new_room.center();
            if rooms.is_empty() {
                // this is the first room, save position as starting point for the player
                starting_position = (new_x, new_y);
            } else {
                // all rooms after the first:
                // connect it to the previous room with a tunnel

                // center coordinates of the previous room
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                // connect both rooms with a horizontal and a vertical tunnel - in random order
                if rand::random() {
                    // move horizontally, then vertically
                    create_h_tunnel(prev_x, new_x, prev_y, &mut map);
                    create_v_tunnel(prev_y, new_y, new_x, &mut map);
                } else {
                    // move vertically, then horizontally
                    create_v_tunnel(prev_y, new_y, prev_x, &mut map);
                    create_h_tunnel(prev_x, new_x, new_y, &mut map);
                }
            }
            // finally, append new room to list
            rooms.push(new_room);
        }
    }
    
    (map, starting_position)
}


// Room generation constraints
const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

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
        Rect { x1: x, y1: y, x2: x + w, y2: y + h}
    }

    pub fn center(&self) -> (i32, i32) {
        let center_x = (self.x1 + self.x2) / 2;
        let center_y = (self.y1 + self.y2) / 2;
        (center_x, center_y)
    }

    /// Return true if this rect intersects with another one.
    pub fn intersects_with(&self, other: &Rect) -> bool {
        (self.x1 <= other.x2) && ( self.x2 >= other.x1) &&
            (self.y1 <= other.y2) && (self.y2 >= other.y1)
    }
}

fn create_room(room: Rect, map: &mut Map) {
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}


// constraints for field of view computing and rendering
const FOV_ALG: FovAlgorithm = FovAlgorithm::Basic;
const FOV_LIGHT_WALLS: bool = true;
const TORCH_RADIUS: i32 = 10;

/// Render all objects and tiles.
fn render_all(root: &mut Root, con: &mut Offscreen, objects: &[Object], map: &Map, fov_map: &mut FovMap, fov_recompute: bool) {
    
    if fov_recompute {
        // recompute fov if needed (the player moved or something)
        let player = &objects[0];
        fov_map.compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALG);
        for y in 0..MAP_HEIGHT {
            for x in  0..MAP_WIDTH {
                let visible = fov_map.is_in_fov(x, y);
                let wall = map[x as usize][y as usize].block_sight;
                let tile_color = match (visible, wall) {
                    // outside field of view:
                    (false, true) => COLOR_DARK_WALL,
                    (false, false) => COLOR_DARK_GROUND,
                    // inside fov:
                    (true, true) => COLOR_LIGHT_WALL,
                    (true, false) => COLOR_LIGHT_GROUND,
                };
                con.set_char_background(x, y, tile_color, BackgroundFlag::Set);
                
            }
        }
    }
    
    for object in objects {
        if fov_map.is_in_fov(object.x, object.y) {
            object.draw(con);
        }
    }
    
    // blit contents of offscreen console to root console and present it
    blit(con, (0, 0), (MAP_WIDTH, MAP_HEIGHT), root, (0, 0), 1.0, 1.0);
}


/// Handle user input
fn handle_keys(root: &mut Root, player: &mut Object, map: &Map) -> bool {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;
    
    let key = root.wait_for_keypress(true);
    match key {
        // toggle fullscreen
        Key { code: Enter, alt: true, .. } => {
            let fullscreen = root.is_fullscreen();
            root.set_fullscreen(!fullscreen);
        }
 
        // exit game
        Key { code: Escape, .. } => return true,

        // handle movement
        Key { code: Up, .. } => player.move_by(map, 0, -1),
        Key { code: Down, .. } => player.move_by(map, 0, 1),
        Key { code: Left, .. } => player.move_by(map, -1, 0),
        Key { code: Right, ..} => player.move_by(map, 1, 0),
        
        _ => {},
    }

    false
}

fn main() {
    let mut root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("roguelike")
        .init();

    let mut con = Offscreen::new(SCREEN_WIDTH, SCREEN_HEIGHT);

    tcod::system::set_fps(LIMIT_FPS);

    // create map and player starting position
    let (map, (player_x, player_y)) = make_map();

    // create object representing the player
    let player = Object::new(player_x, player_y, '@', colors::WHITE);

    // create an NPC object
    let npc = Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', colors::YELLOW);

    // create array holding all objects
    let mut objects = [player, npc];

    // init fov map
    let mut fov_map = FovMap::new(MAP_WIDTH, MAP_HEIGHT);
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            fov_map.set(x, y,
                        !map[x as usize][y as usize].block_sight,
                        !map[x as usize][y as usize].blocked);
        }
    }

    let mut previous_player_position = (-1, -1);

    while !root.window_closed() {
        // render objects and map
        let fov_recompute = previous_player_position != (objects[0].x, objects[0].y);
        render_all(&mut root, &mut con, &objects, &map, &mut fov_map, fov_recompute);

        root.flush(); // draw everything on the window at once
        
        // erase all objects from their old locations before they move
        for object in &objects {
            object.clear(&mut con);
        }

        // handle keys and exit game if needed
        let player = &mut objects[0];
        previous_player_position = (player.x, player.y);
        let exit = handle_keys(&mut root, player, &map);
        if exit {
            break
        }
    }
}
