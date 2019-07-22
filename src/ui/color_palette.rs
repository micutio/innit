/// Module Main
///
/// This module contains the color palette and related constants and methods
/// for color calculation and manipulation.
///
/// TODO: Read constants from file.
/// TODO: Better readable colors between dark and light scheme
use tcod::colors::Color;

const COL_LIGHT_MAIN: Color = Color {
    r: 158,
    g: 53,
    b: 74,
};

const COL_LIGHT_ACCENT_WARM: Color = Color {
    r: 210,
    g: 152,
    b: 107,
};

const COL_LIGHT_ACCENT_COLD: Color = Color {
    r: 72,
    g: 143,
    b: 181,
};

const COL_LIGHT_PLAYER: Color = Color {
    r: 100,
    g: 100,
    b: 100,
};

const COL_DARK_MAIN: Color = Color {
    r: 158,
    g: 53,
    b: 74,
};

const COL_DARK_ACCENT_WARM: Color = Color {
    r: 210,
    g: 152,
    b: 107,
};

const COL_DARK_ACCENT_COLD: Color = Color {
    r: 72,
    g: 143,
    b: 181,
};

const COL_DARK_PLAYER: Color = Color {
    r: 170,
    g: 170,
    b: 170,
};

pub struct ColorPalette {
    // background colors
    pub bg_world: Color,
    pub bg_dialog: Color,
    pub bg_wall_fov_true: Color,
    pub bg_wall_fov_false: Color,
    pub bg_ground_fov_true: Color,
    pub bg_ground_fov_false: Color,

    // foreground colors
    pub fg_dialog: Color,
    pub fg_dialog_border: Color,
    pub fg_dialog_highlight: Color,
    pub fg_wall_fov_true: Color,
    pub fg_wall_fov_false: Color,
    pub fg_ground_fov_true: Color,
    pub fg_ground_fov_false: Color,

    pub player: Color,
}

impl ColorPalette {
    pub fn new_light() -> Self {
        ColorPalette {
            bg_world: Color {
                r: 250,
                g: 250,
                b: 250,
            },
            bg_dialog: Color {
                r: 215,
                g: 133,
                b: 144,
            },
            bg_wall_fov_true: Color {
                r: 250,
                g: 110,
                b: 130,
            },
            bg_wall_fov_false: Color {
                r: 240,
                g: 240,
                b: 240,
            },
            bg_ground_fov_true: Color {
                r: 250,
                g: 160,
                b: 180,
            },
            bg_ground_fov_false: Color {
                r: 250,
                g: 250,
                b: 250,
            },
            fg_dialog: Color {
                r: 85,
                g: 85,
                b: 85,
            },
            fg_dialog_border: Color {
                r: 89,
                g: 198,
                b: 217,
            },
            fg_dialog_highlight: Color {
                r: 212,
                g: 192,
                b: 80,
            },
            fg_wall_fov_true: Color {
                r: 255,
                g: 80,
                b: 105,
            },
            fg_wall_fov_false: Color {
                r: 230,
                g: 230,
                b: 230,
            },
            fg_ground_fov_true: Color {
                r: 255,
                g: 130,
                b: 150,
            },
            fg_ground_fov_false: Color {
                r: 210,
                g: 210,
                b: 210,
            },
            player: COL_LIGHT_PLAYER,
        }
    }

    pub fn new_dark() -> Self {
        ColorPalette {
            bg_world: Color { r: 0, g: 0, b: 0 },
            bg_dialog: Color {
                r: 144,
                g: 48,
                b: 90,
            },
            bg_wall_fov_true: Color {
                r: 176,
                g: 52,
                b: 96,
            },
            bg_wall_fov_false: Color {
                r: 20,
                g: 20,
                b: 20,
            },
            bg_ground_fov_true: Color {
                r: 124,
                g: 8,
                b: 59,
            },
            bg_ground_fov_false: Color { r: 0, g: 0, b: 0 },

            fg_dialog: Color {
                r: 220,
                g: 220,
                b: 220,
            },

            fg_dialog_border: Color {
                r: 72,
                g: 143,
                b: 181,
            },
            fg_dialog_highlight: Color {
                r: 44,
                g: 88,
                b: 112,
            },
            fg_wall_fov_true: Color {
                r: 218,
                g: 85,
                b: 135,
            },
            fg_wall_fov_false: Color {
                r: 20,
                g: 20,
                b: 20,
            },
            fg_ground_fov_true: Color {
                r: 144,
                g: 48,
                b: 90,
            },
            fg_ground_fov_false: Color {
                r: 40,
                g: 40,
                b: 40,
            },
            player: COL_DARK_PLAYER,
        }
    }
}
