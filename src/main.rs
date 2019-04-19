extern crate tcod;

use tcod::console::*;
use tcod::colors::{self, Color};

// window size
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
// target fps
const LIMIT_FPS: i32 = 20;


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

    pub fn move_by(&mut self, dx: i32, dy: i32) {
        // move by the given amount
        self.x += dx;
        self.y += dy;
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


/// Handle user input
fn handle_keys(root: &mut Root, player: &mut Object) -> bool {
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
        Key { code: Up, .. } => player.move_by(0, -1),
        Key { code: Down, .. } => player.move_by(0, 1),
        Key { code: Left, .. } => player.move_by(-1, 0),
        Key { code: Right, ..} => player.move_by(1, 0),
        
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

    // create object representing the player
    let player = Object::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, '@', colors::WHITE);

    // create an NPC object
    let npc = Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', colors::YELLOW);

    // create array holding all objects
    let mut objects = [player, npc];

    while !root.window_closed() {
        // draw all objects in the list
        for object in &objects {
            object.draw(&mut con);
        }
        
        // blit contents of offscreen console to root console and present it
        blit(&mut con, (0, 0), (SCREEN_WIDTH, SCREEN_HEIGHT), &mut root, (0, 0), 1.0, 1.0);
        root.flush(); // draw everything on the window at once
        
        // erase all objects from their old locations before they move
        for object in &objects {
            object.clear(&mut con);
        }

        // handle keys and exit game if needed
        let player = &mut objects[0];
        let exit = handle_keys(&mut root, player);
        if exit {
            break
        }
    }
}
