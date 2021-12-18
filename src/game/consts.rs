// environment constraints
// game window
pub const SCREEN_WIDTH: i32 = 100;
pub const SCREEN_HEIGHT: i32 = 60;
// world
pub const WORLD_WIDTH: i32 = 80;
pub const WORLD_HEIGHT: i32 = 60;
// sidebar
pub const SIDE_PANEL_WIDTH: i32 = 20;
pub const SIDE_PANEL_HEIGHT: i32 = 60;
// consoles
pub const WORLD_CON: usize = 0;
pub const WORLD_CON_Z: usize = 1000;
pub const HUD_CON: usize = 1;
pub const HUD_CON_Z: usize = 10000;
pub const PAR_CON: usize = 2;
pub const PAR_CON_Z: usize = 20000;

pub const MENU_WIDTH: i32 = 20;

pub const PLAYER: usize = (WORLD_WIDTH * WORLD_HEIGHT) as usize; // player object reference, index of the object vector
