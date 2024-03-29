// environment constraints
// game window
pub const SCREEN_WIDTH: i32 = 80;
pub const SCREEN_HEIGHT: i32 = 60;
// world
pub const WORLD_WIDTH: i32 = 60;
pub const WORLD_HEIGHT: i32 = 60;
// sidebar
pub const SIDE_PANEL_WIDTH: i32 = 21;
pub const SIDE_PANEL_HEIGHT: i32 = 60;

// consoles

/// id of world console
pub const WORLD_CON: usize = 0;

/// starting z-level of world console
pub const WORLD_TILE_Z: usize = 0;

/// starting z-level of non-blocking world tiles and objects
pub const WORLD_NBL_Z: usize = 300_000;

/// starting z-level of blocking world tiles and objects
pub const WORLD_BLK_Z: usize = 600_000;

/// id of particle console
pub const PAR_CON: usize = 1;

/// starting z-level of particle console
pub const PAR_CON_Z: usize = 1_000_000;

/// id of shader console
pub const SHADER_CON: usize = 2;

/// starting z-level of shader console
pub const SHADER_CON_Z: usize = 2_000_000;

// id of hud console
pub const HUD_CON: usize = 3;

/// starting z-level of hud console
pub const HUD_CON_Z: usize = 3_000_000;

pub const MENU_WIDTH: i32 = 20;

pub const PLAYER: usize = (WORLD_WIDTH * WORLD_HEIGHT) as usize; // player object reference, index of the object vector
