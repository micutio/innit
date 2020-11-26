// TODO: Read constants from file.
// TODO: Better readable colors between dark and light scheme
use crate::ui::color::Color;

pub struct ColorPalette {
    // background colors
    pub bg_world: Color,
    pub bg_dialog: Color,
    pub bg_dialog_selected: Color,
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

    pub cyan: Color,
    pub magenta: Color,
    pub yellow: Color,
    pub player: Color,
}

impl ColorPalette {
    pub fn light() -> Self {
        ColorPalette {
            bg_world: Color::new(250, 250, 250),
            bg_dialog: Color::new(215, 133, 144),
            bg_dialog_selected: Color::new(215, 143, 164),
            bg_wall_fov_true: Color::new(250, 110, 130),
            bg_wall_fov_false: Color::new(240, 240, 240),
            bg_ground_fov_true: Color::new(250, 160, 180),
            bg_ground_fov_false: Color::new(250, 250, 250),
            fg_dialog: Color::new(85, 85, 85),
            fg_dialog_border: Color::new(89, 198, 217),
            fg_dialog_highlight: Color::new(212, 192, 80),
            fg_wall_fov_true: Color::new(255, 80, 105),
            fg_wall_fov_false: Color::new(230, 230, 230),
            fg_ground_fov_true: Color::new(255, 130, 150),
            fg_ground_fov_false: Color::new(210, 210, 210),
            cyan: Color::new(0, 190, 190),
            magenta: Color::new(190, 0, 190),
            yellow: Color::new(190, 190, 0),
            player: Color::new(100, 100, 100),
        }
    }

    pub fn dark() -> Self {
        ColorPalette {
            bg_world: Color::new(0, 0, 0),
            bg_dialog: Color::new(144, 48, 90),
            bg_dialog_selected: Color::new(144, 58, 110),
            bg_wall_fov_true: Color::new(176, 52, 96),
            bg_wall_fov_false: Color::new(30, 30, 30),
            bg_ground_fov_true: Color::new(124, 8, 59),
            bg_ground_fov_false: Color::new(20, 20, 20),

            fg_dialog: Color::new(220, 220, 220),

            fg_dialog_border: Color::new(72, 143, 181),
            fg_dialog_highlight: Color::new(44, 88, 112),
            fg_wall_fov_true: Color::new(218, 85, 135),
            fg_wall_fov_false: Color::new(30, 30, 30),
            fg_ground_fov_true: Color::new(144, 48, 90),
            fg_ground_fov_false: Color::new(20, 20, 20),
            cyan: Color::new(0, 150, 150),
            magenta: Color::new(180, 0, 220),
            yellow: Color::new(150, 150, 0),
            player: Color::new(170, 170, 170),
        }
    }
}
