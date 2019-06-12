/// Module Main
///
/// This module contains the color palette and related constants and methods
/// for color calculation and manipulation.
use tcod::colors::Color;

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

pub fn get_col_dark_wall() -> Color {
    COLOR_DARK_WALL
}

pub fn get_col_light_wall() -> Color {
    COLOR_LIGHT_WALL
}

pub fn get_col_dark_ground() -> Color {
    COLOR_DARK_GROUND
}

pub fn get_col_light_ground() -> Color {
    COLOR_LIGHT_GROUND
}
