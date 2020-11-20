// TODO: Read constants from file.
// TODO: Better readable colors between dark and light scheme
use rltk::RGB;
pub struct ColorPalette {
    // background colors
    pub bg_world: RGB,
    pub bg_dialog: RGB,
    pub bg_wall_fov_true: RGB,
    pub bg_wall_fov_false: RGB,
    pub bg_ground_fov_true: RGB,
    pub bg_ground_fov_false: RGB,

    // foreground colors
    pub fg_dialog: RGB,
    pub fg_dialog_border: RGB,
    pub fg_dialog_highlight: RGB,
    pub fg_wall_fov_true: RGB,
    pub fg_wall_fov_false: RGB,
    pub fg_ground_fov_true: RGB,
    pub fg_ground_fov_false: RGB,

    pub cyan: RGB,
    pub magenta: RGB,
    pub yellow: RGB,
    pub player: RGB,
}

impl ColorPalette {
    pub fn light() -> Self {
        ColorPalette {
            bg_world: RGB::from_u8(250, 250, 250),
            bg_dialog: RGB::from_u8(215, 133, 144),
            bg_wall_fov_true: RGB::from_u8(250, 110, 130),
            bg_wall_fov_false: RGB::from_u8(240, 240, 240),
            bg_ground_fov_true: RGB::from_u8(250, 160, 180),
            bg_ground_fov_false: RGB::from_u8(250, 250, 250),
            fg_dialog: RGB::from_u8(85, 85, 85),
            fg_dialog_border: RGB::from_u8(89, 198, 217),
            fg_dialog_highlight: RGB::from_u8(212, 192, 80),
            fg_wall_fov_true: RGB::from_u8(255, 80, 105),
            fg_wall_fov_false: RGB::from_u8(230, 230, 230),
            fg_ground_fov_true: RGB::from_u8(255, 130, 150),
            fg_ground_fov_false: RGB::from_u8(210, 210, 210),
            cyan: RGB::from_u8(0, 190, 190),
            magenta: RGB::from_u8(190, 0, 190),
            yellow: RGB::from_u8(190, 190, 0),
            player: RGB::from_u8(100, 100, 100),
        }
    }

    pub fn dark() -> Self {
        ColorPalette {
            bg_world: RGB::from_u8(0, 0, 0),
            bg_dialog: RGB::from_u8(144, 48, 90),
            bg_wall_fov_true: RGB::from_u8(176, 52, 96),
            bg_wall_fov_false: RGB::from_u8(30, 30, 30),
            bg_ground_fov_true: RGB::from_u8(124, 8, 59),
            bg_ground_fov_false: RGB::from_u8(20, 20, 20),

            fg_dialog: RGB::from_u8(220, 220, 220),

            fg_dialog_border: RGB::from_u8(72, 143, 181),
            fg_dialog_highlight: RGB::from_u8(44, 88, 112),
            fg_wall_fov_true: RGB::from_u8(218, 85, 135),
            fg_wall_fov_false: RGB::from_u8(30, 30, 30),
            fg_ground_fov_true: RGB::from_u8(144, 48, 90),
            fg_ground_fov_false: RGB::from_u8(20, 20, 20),
            cyan: RGB::from_u8(0, 150, 150),
            magenta: RGB::from_u8(180, 0, 220),
            yellow: RGB::from_u8(150, 150, 0),
            player: RGB::from_u8(170, 170, 170),
        }
    }
}
