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
    r: 210,
    g: 152,
    b: 107,
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

    is_light_theme: bool,

    brightness_none: f32,
    brightness_less: f32,
    brightness_medium: f32,
    brightness_more: f32,
    brightness_full: f32,
    saturation_none: f32,
    saturation_less: f32,
    saturation_medium: f32,
    saturation_more: f32,
    saturation_full: f32,
}

impl ColorPalette {
    pub fn new() -> Self {
        ColorPalette {
            col_main: COL_MAIN,
            col_acc_warm: COL_ACCENT_WARM,
            col_acc_cold: COL_ACCENT_COLD,

            is_light_theme: true,

            brightness_none: 0.0,
            brightness_less: 0.3,
            brightness_medium: 1.0,
            brightness_more: 1.3,
            brightness_full: 2.0,
            saturation_none: 0.0,
            saturation_less: 0.3,
            saturation_medium: 1.0,
            saturation_more: 1.3,
            saturation_full: 2.0,
        }
    }

    pub fn toggle_dark_light_mode(&mut self) {
        self.is_light_theme = !self.is_light_theme;
    }

    pub fn get_col_ground_in_fov(&self) -> Color {
        if self.is_light_theme {
            self.col_main
                .scale_hsv(self.saturation_medium, self.brightness_full)
        } else {
            self.col_main
                .scale_hsv(self.brightness_full, self.saturation_medium)
        }
    }

    pub fn get_col_wall_in_fov(&self) -> Color {
        if self.is_light_theme {
            self.col_main
                .scale_hsv(self.saturation_medium, self.brightness_more)
        } else {
            self.col_main
                .scale_hsv(self.brightness_more, self.saturation_medium)
        }
    }

    pub fn get_col_ground_out_fov(&self) -> Color {
        if self.is_light_theme {
            self.col_main
                .scale_hsv(self.saturation_none, self.brightness_medium * 1.3)
        } else {
            self.col_main
                .scale_hsv(self.brightness_medium * 1.3, self.saturation_none)
        }
    }

    pub fn get_col_wall_out_fov(&self) -> Color {
        if self.is_light_theme {
            self.col_main
                .scale_hsv(self.saturation_none, self.brightness_medium)
        } else {
            self.col_main
                .scale_hsv(self.brightness_medium, self.saturation_none)
        }
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
        if self.is_light_theme {
            self.col_main
                .scale_hsv(self.saturation_medium, self.brightness_full)
        } else {
            self.col_main
                .scale_hsv(self.brightness_full, self.saturation_medium)
        }
    }

    pub fn get_col_world_bg(&self) -> Color {
        if self.is_light_theme {
            self.col_main
                .scale_hsv(self.saturation_less, self.brightness_full)
        } else {
            self.col_main
                .scale_hsv(self.brightness_full, self.saturation_less)
        }
    }
}
