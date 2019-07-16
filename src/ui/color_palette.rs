/// Module Main
///
/// This module contains the color palette and related constants and methods
/// for color calculation and manipulation.
///
/// TODO: Read constants from file.
/// TODO: Distinguish between dark and light theme
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
    r: 72,
    g: 143,
    b: 181,
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
            dark_factor: 0.3,
            light_factor: 0.9,
            full_saturation_factor: 0.9,
            de_saturation_factor: 0.3,
        }
    }

    pub fn toggle_dark_light_mode(&mut self) {
        self.dark_factor = 1.0 - self.dark_factor;
        self.light_factor = 1.0 - self.light_factor;
        println!("toggle dark/light");
    }

    pub fn get_col_ground_in_fov(&self) -> Color {
        self.col_main
            .scale_hsv(self.full_saturation_factor, self.light_factor)
    }

    pub fn get_col_wall_in_fov(&self) -> Color {
        self.col_main
            .scale_hsv(self.full_saturation_factor, self.light_factor * 0.7)
    }

    pub fn get_col_ground_out_fov(&self) -> Color {
        self.col_main
            .scale_hsv(self.de_saturation_factor, self.dark_factor)
        // .scale_hsv(self.de_saturation_factor, self.light_factor * 1.3)
    }

    pub fn get_col_wall_out_fov(&self) -> Color {
        self.col_main
            .scale_hsv(self.de_saturation_factor, self.dark_factor / 2.0)
        // .scale_hsv(self.de_saturation_factor, self.light_factor * 1.0)
    }

    pub fn get_col_acc_warm(&self) -> Color {
        self.col_acc_warm
    }

    pub fn get_col_acc_cold(&self) -> Color {
        self.col_acc_cold
    }

    pub fn get_col_menu_fg(&self) -> Color {
        self.col_acc_cold
    }

    pub fn get_col_menu_bg(&self) -> Color {
        self.col_main
            .scale_hsv(self.de_saturation_factor, self.light_factor * 2.0)
    }

    pub fn get_col_world_bg(&self) -> Color {
        self.col_main
            .scale_hsv(self.de_saturation_factor, self.dark_factor)
    }
}
