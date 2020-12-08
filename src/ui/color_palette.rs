// TODO: Read constants from file.
// TODO: Better readable colors between dark and light scheme

pub struct ColorPalette {
    // background colors
    pub bg_world: (u8, u8, u8),
    pub bg_dialog: (u8, u8, u8),
    pub bg_dialog_selected: (u8, u8, u8),
    pub bg_wall_fov_true: (u8, u8, u8),
    pub bg_wall_fov_false: (u8, u8, u8),
    pub bg_ground_fov_true: (u8, u8, u8),
    pub bg_ground_fov_false: (u8, u8, u8),
    pub bg_bar: (u8, u8, u8),

    // foreground colors
    pub fg_dialog: (u8, u8, u8),
    pub fg_dialog_border: (u8, u8, u8),
    pub fg_dialog_highlight: (u8, u8, u8),
    pub fg_wall_fov_true: (u8, u8, u8),
    pub fg_wall_fov_false: (u8, u8, u8),
    pub fg_ground_fov_true: (u8, u8, u8),
    pub fg_ground_fov_false: (u8, u8, u8),

    pub cyan: (u8, u8, u8),
    pub magenta: (u8, u8, u8),
    pub yellow: (u8, u8, u8),
    pub player: (u8, u8, u8),
    pub msg_alert: (u8, u8, u8),
    pub msg_info: (u8, u8, u8),
    pub msg_action: (u8, u8, u8),
    pub msg_story: (u8, u8, u8),
}

pub const PALETTE_LIGHT: ColorPalette = ColorPalette {
    bg_world: (250, 250, 250),
    bg_dialog: (215, 133, 144),
    bg_dialog_selected: (215, 143, 164),
    bg_wall_fov_true: (250, 110, 130),
    bg_wall_fov_false: (240, 240, 240),
    bg_ground_fov_true: (250, 160, 180),
    bg_ground_fov_false: (250, 250, 250),
    bg_bar: (100, 100, 100),
    fg_dialog: (85, 85, 85),
    fg_dialog_border: (89, 198, 217),
    fg_dialog_highlight: (212, 192, 80),
    fg_wall_fov_true: (255, 80, 105),
    fg_wall_fov_false: (230, 230, 230),
    fg_ground_fov_true: (255, 130, 150),
    fg_ground_fov_false: (210, 210, 210),
    cyan: (0, 190, 190),
    magenta: (190, 0, 190),
    yellow: (190, 190, 0),
    player: (100, 100, 100),
    msg_alert: (255, 100, 100),
    msg_info: (255, 255, 255),
    msg_action: (100, 100, 255),
    msg_story: (100, 180, 255),
};

pub const PALETTE_DARK: ColorPalette = ColorPalette {
    bg_world: (0, 0, 0),
    bg_dialog: (144, 48, 90),
    bg_dialog_selected: (90, 48, 144),
    bg_wall_fov_true: (176, 52, 96),
    bg_wall_fov_false: (30, 30, 30),
    bg_ground_fov_true: (124, 8, 59),
    bg_ground_fov_false: (20, 20, 20),
    bg_bar: (100, 100, 100),

    fg_dialog: (220, 220, 220),

    fg_dialog_border: (72, 143, 181),
    fg_dialog_highlight: (44, 88, 112),
    fg_wall_fov_true: (218, 85, 135),
    fg_wall_fov_false: (30, 30, 30),
    fg_ground_fov_true: (144, 48, 90),
    fg_ground_fov_false: (20, 20, 20),
    cyan: (0, 150, 150),
    magenta: (180, 0, 220),
    yellow: (150, 150, 0),
    player: (170, 170, 170),
    msg_alert: (255, 100, 100),
    msg_info: (255, 255, 255),
    msg_action: (100, 100, 255),
    msg_story: (100, 180, 255),
};

// TODO: Find out what static lifetime is!
impl ColorPalette {
    pub fn get(dark_mode: bool) -> &'static Self {
        if dark_mode {
            &PALETTE_DARK
        } else {
            &PALETTE_LIGHT
        }
    }
}
