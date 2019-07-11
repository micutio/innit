/// Module Main
///
/// This module contains the color palette and related constants and methods
/// for color calculation and manipulation.
///
/// TODO: Read constants from file.
use tcod::colors::Color;

const COL_MAIN: Color = Color {
    r: 158,
    g: 53,
    b: 74,
};

const COL_ACCENT_WARM: Color = Color {
    r: 170,
    g: 92,
    b: 57,
};

const COL_ACCENT_COLD: Color = Color {
    r: 130,
    g: 43,
    b: 102,
};

pub struct ColorPalette {
    col_main: Color,
    col_acc_warm: Color,
    col_acc_cold: Color,
    dark_factor: f32,
    light_factor: f32,
    full_saturation_factor: f32,
    de_saturation_factor: f32,
}

impl ColorPalette {
    pub fn new() -> Self {
        ColorPalette {
            col_main: COL_MAIN,
            col_acc_warm: COL_ACCENT_WARM,
            col_acc_cold: COL_ACCENT_COLD,
            dark_factor: 0.2,
            light_factor: 0.8,
            full_saturation_factor: 0.9,
            de_saturation_factor: 0.3,
        }
    }

    pub fn get_col_light_ground(&self) -> Color {
        self.col_main
            .scale_hsv(self.full_saturation_factor, self.light_factor)
    }

    pub fn get_col_light_wall(&self) -> Color {
        self.col_main
            .scale_hsv(self.full_saturation_factor, self.light_factor / 2.0)
    }

    pub fn get_col_dark_ground(&self) -> Color {
        self.col_main
            .scale_hsv(self.de_saturation_factor, self.dark_factor)
    }

    pub fn get_col_dark_wall(&self) -> Color {
        self.col_main
            .scale_hsv(self.de_saturation_factor, self.dark_factor / 2.0)
    }
}
