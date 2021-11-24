pub struct ColorPalette {
    // base color palette
    /// Main color, used as base for the UI and the world, probably.
    pub col_main: (u8, u8, u8, u8),
    /// Main complementary color
    pub col_comp: (u8, u8, u8, u8),
    /// Accent color 1
    pub col_acc1: (u8, u8, u8, u8),
    /// Accent color 2
    pub col_acc2: (u8, u8, u8, u8),
    /// Accent color 3
    pub col_acc3: (u8, u8, u8, u8),

    // hud colors - background
    pub hud_bg: (u8, u8, u8, u8),
    pub hud_bg_bar: (u8, u8, u8, u8),
    pub hud_bg_dna: (u8, u8, u8, u8),
    pub hud_bg_content: (u8, u8, u8, u8),
    pub hud_bg_active: (u8, u8, u8, u8),
    pub hud_bg_log1: (u8, u8, u8, u8),
    pub hud_bg_log2: (u8, u8, u8, u8),
    pub hud_bg_tooltip: (u8, u8, u8, u8),

    // hud colors - foreground
    pub hud_fg: (u8, u8, u8, u8),
    pub hud_fg_border: (u8, u8, u8, u8),
    pub hud_fg_highlight: (u8, u8, u8, u8),
    pub hud_fg_inactive: (u8, u8, u8, u8),
    pub hud_fg_dna_processor: (u8, u8, u8, u8),
    pub hud_fg_dna_actuator: (u8, u8, u8, u8),
    pub hud_fg_dna_sensor: (u8, u8, u8, u8),
    pub hud_fg_bar_health: (u8, u8, u8, u8),
    pub hud_fg_bar_energy: (u8, u8, u8, u8),
    pub hud_fg_bar_lifetime: (u8, u8, u8, u8),
    pub hud_fg_msg_alert: (u8, u8, u8, u8),
    pub hud_fg_msg_info: (u8, u8, u8, u8),
    pub hud_fg_msg_action: (u8, u8, u8, u8),
    pub hud_fg_msg_story: (u8, u8, u8, u8),

    // world colors
    pub world_bg: (u8, u8, u8, u8),
    pub world_bg_wall_fov_true: (u8, u8, u8, u8),
    pub world_bg_wall_fov_false: (u8, u8, u8, u8),
    pub world_bg_ground_fov_true: (u8, u8, u8, u8),
    pub world_bg_ground_fov_false: (u8, u8, u8, u8),
    pub world_fg_wall_fov_true: (u8, u8, u8, u8),
    pub world_fg_wall_fov_false: (u8, u8, u8, u8),
    pub world_fg_ground_fov_true: (u8, u8, u8, u8),
    pub world_fg_ground_fov_false: (u8, u8, u8, u8),

    // entity colors
    pub entity_player: (u8, u8, u8, u8),
    pub entity_plasmid: (u8, u8, u8, u8),
    pub entity_virus: (u8, u8, u8, u8),
    pub entity_bacteria: (u8, u8, u8, u8),
}

pub const PALETTE_DEFAULT: ColorPalette = ColorPalette {
    // base color palette
    /// Main color, used as base for the UI and the world, probably.
    col_main: (124, 8, 59, 255),
    /// Main complementary color
    col_comp: (9, 124, 172, 255),
    /// Accent color 1
    col_acc1: (167, 223, 204, 255),
    /// Accent color 2
    col_acc2: (182, 191, 118, 255),
    /// Accent color 3
    col_acc3: (220, 98, 42, 255),

    // hud colors - background
    hud_bg: (124, 8, 59, 255),
    hud_bg_bar: (99, 99, 99, 255),
    hud_bg_content: (82, 59, 99, 255),
    hud_bg_dna: (105, 7, 50, 255),
    hud_bg_active: (145, 9, 68, 255),
    hud_bg_log1: (124, 8, 59, 255),
    hud_bg_log2: (145, 9, 68, 255),
    hud_bg_tooltip: (42, 82, 99, 255),

    // hud colors - foreground
    hud_fg: (196, 196, 196, 255),
    hud_fg_border: (9, 124, 172, 255),
    hud_fg_highlight: (9, 124, 172, 255),
    hud_fg_inactive: (99, 99, 99, 255),
    hud_fg_dna_actuator: (240, 50, 30, 255),
    hud_fg_dna_processor: (97, 83, 224, 255),
    hud_fg_dna_sensor: (124, 182, 52, 255),
    hud_fg_bar_health: (240, 50, 30, 255),
    hud_fg_bar_energy: (220, 184, 68, 255),
    hud_fg_bar_lifetime: (68, 184, 220, 255),
    hud_fg_msg_alert: (255, 100, 100, 255),
    hud_fg_msg_info: (255, 255, 255, 255),
    hud_fg_msg_action: (100, 100, 255, 255),
    hud_fg_msg_story: (100, 180, 255, 255),

    // world colors
    world_bg: (39, 39, 39, 255),
    world_bg_wall_fov_true: (176, 52, 96, 255),
    world_bg_wall_fov_false: (39, 39, 39, 255),
    world_bg_ground_fov_true: (124, 8, 59, 255),
    world_bg_ground_fov_false: (39, 39, 39, 255),
    world_fg_wall_fov_true: (218, 85, 135, 255),
    world_fg_wall_fov_false: (49, 49, 49, 255),
    world_fg_ground_fov_true: (144, 48, 90, 255),
    world_fg_ground_fov_false: (49, 49, 49, 255),

    // entity colors
    entity_player: (170, 170, 170, 255),
    entity_plasmid: (50, 50, 250, 255),
    entity_virus: (100, 255, 150, 255),
    entity_bacteria: (80, 235, 120, 255),
};
