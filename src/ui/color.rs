use crate::ui;

/// The Innit color palette defines all colored elements commonly used in the game.
pub struct Palette {
    /// Main color, used as base for the UI and the world, probably.
    pub col_main: ui::Rgba,
    /// Main complementary color
    pub col_comp: ui::Rgba,
    /// Accent color 1
    pub col_acc1: ui::Rgba,
    /// Accent color 2
    pub col_acc2: ui::Rgba,
    /// Accent color 3
    pub col_acc3: ui::Rgba,
    /// Accent color 4
    pub col_acc4: ui::Rgba,
    /// transparent color
    pub col_transparent: ui::Rgba,

    // hud colors - background
    pub hud_bg: ui::Rgba,
    pub hud_bg_bar: ui::Rgba,
    pub hud_bg_dna: ui::Rgba,
    pub hud_bg_content: ui::Rgba,
    pub hud_bg_active: ui::Rgba,
    pub hud_bg_log1: ui::Rgba,
    pub hud_bg_log2: ui::Rgba,
    pub hud_bg_tooltip: ui::Rgba,
    pub hud_bg_plasmid_dna: ui::Rgba,

    // hud colors - foreground
    pub hud_fg: ui::Rgba,
    pub hud_fg_border: ui::Rgba,
    pub hud_fg_highlight: ui::Rgba,
    pub hud_fg_inactive: ui::Rgba,
    pub hud_fg_dna_processor: ui::Rgba,
    pub hud_fg_dna_actuator: ui::Rgba,
    pub hud_fg_dna_sensor: ui::Rgba,
    pub hud_fg_bar_health: ui::Rgba,
    pub hud_fg_bar_energy: ui::Rgba,
    pub hud_fg_bar_lifetime: ui::Rgba,
    pub hud_fg_msg_alert: ui::Rgba,
    pub hud_fg_msg_info: ui::Rgba,
    pub hud_fg_msg_action: ui::Rgba,
    pub hud_fg_msg_story: ui::Rgba,

    // world colors
    pub world_bg: ui::Rgba,
    pub world_bg_wall_fov_true: ui::Rgba,
    pub world_bg_wall_fov_false: ui::Rgba,
    pub world_bg_floor_fov_true: ui::Rgba,
    pub world_bg_floor_fov_false: ui::Rgba,
    pub world_fg_wall_fov_true: ui::Rgba,
    pub world_fg_wall_fov_false: ui::Rgba,
    pub world_fg_floor_fov_true: ui::Rgba,
    pub world_fg_floor_fov_false: ui::Rgba,

    // entity colors
    pub entity_player: ui::Rgba,
    pub entity_plasmid: ui::Rgba,
    pub entity_virus: ui::Rgba,
    pub entity_bacteria: ui::Rgba,
}

pub const DEFAULT_PALETTE: Palette = Palette {
    // base color palette
    /// Main color, used as base for the UI and the world, probably.
    col_main: ui::Rgba {
        r: 124,
        g: 59,
        b: 8,
        a: 255,
    },
    /// Main complementary color
    col_comp: ui::Rgba {
        r: 9,
        g: 124,
        b: 172,
        a: 255,
    },
    /// Accent color 1
    col_acc1: ui::Rgba {
        r: 157,
        g: 213,
        b: 194,
        a: 255,
    },
    /// Accent color 2
    col_acc2: ui::Rgba {
        r: 182,
        g: 191,
        b: 118,
        a: 255,
    },
    /// Accent color 3
    col_acc3: ui::Rgba {
        r: 220,
        g: 98,
        b: 42,
        a: 255,
    },
    /// Accent color 4
    col_acc4: ui::Rgba {
        r: 100,
        g: 180,
        b: 240,
        a: 255,
    },
    /// transparent color
    col_transparent: ui::Rgba {
        r: 255,
        g: 255,
        b: 255,
        a: 0,
    },

    // hud colors - background
    hud_bg: ui::Rgba {
        r: 45,
        g: 45,
        b: 45,
        a: 255,
    },
    hud_bg_bar: ui::Rgba {
        r: 85,
        g: 85,
        b: 85,
        a: 255,
    },
    hud_bg_content: ui::Rgba {
        r: 82,
        g: 59,
        b: 59,
        a: 255,
    },
    hud_bg_dna: ui::Rgba {
        r: 75,
        g: 75,
        b: 75,
        a: 255,
    },
    hud_bg_active: ui::Rgba {
        r: 120,
        g: 120,
        b: 120,
        a: 255,
    },
    hud_bg_log1: ui::Rgba {
        r: 35,
        g: 35,
        b: 35,
        a: 255,
    },
    hud_bg_log2: ui::Rgba {
        r: 25,
        g: 25,
        b: 25,
        a: 255,
    },
    hud_bg_tooltip: ui::Rgba {
        r: 85,
        g: 85,
        b: 85,
        a: 255,
    },
    hud_bg_plasmid_dna: ui::Rgba {
        r: 60,
        g: 90,
        b: 125,
        a: 255,
    },

    // hud colors - foreground
    hud_fg: ui::Rgba {
        r: 200,
        g: 200,
        b: 200,
        a: 255,
    },
    hud_fg_border: ui::Rgba {
        r: 9,
        g: 124,
        b: 172,
        a: 255,
    },
    hud_fg_highlight: ui::Rgba {
        r: 9,
        g: 124,
        b: 172,
        a: 255,
    },
    hud_fg_inactive: ui::Rgba {
        r: 140,
        g: 140,
        b: 140,
        a: 255,
    },
    hud_fg_dna_actuator: ui::Rgba {
        r: 220,
        g: 50,
        b: 30,
        a: 255,
    },
    hud_fg_dna_processor: ui::Rgba {
        r: 130,
        g: 90,
        b: 230,
        a: 255,
    },
    hud_fg_dna_sensor: ui::Rgba {
        r: 90,
        g: 220,
        b: 70,
        a: 255,
    },
    hud_fg_bar_health: ui::Rgba {
        r: 240,
        g: 50,
        b: 30,
        a: 255,
    },
    hud_fg_bar_energy: ui::Rgba {
        r: 220,
        g: 184,
        b: 68,
        a: 255,
    },
    hud_fg_bar_lifetime: ui::Rgba {
        r: 68,
        g: 184,
        b: 220,
        a: 255,
    },
    hud_fg_msg_alert: ui::Rgba {
        r: 220,
        g: 80,
        b: 80,
        a: 255,
    },
    hud_fg_msg_info: ui::Rgba {
        r: 96,
        g: 96,
        b: 96,
        a: 255,
    },
    hud_fg_msg_action: ui::Rgba {
        r: 80,
        g: 80,
        b: 220,
        a: 255,
    },
    hud_fg_msg_story: ui::Rgba {
        r: 60,
        g: 140,
        b: 200,
        a: 255,
    },

    // world colors
    world_bg: ui::Rgba {
        r: 49,
        g: 49,
        b: 49,
        a: 255,
    },
    world_bg_wall_fov_false: ui::Rgba {
        r: 100,
        g: 30,
        b: 50,
        a: 255,
    },
    world_fg_wall_fov_false: ui::Rgba {
        r: 105,
        g: 35,
        b: 55,
        a: 255,
    },
    world_bg_wall_fov_true: ui::Rgba {
        r: 154,
        g: 38,
        b: 84,
        a: 255,
    },
    world_fg_wall_fov_true: ui::Rgba {
        r: 206,
        g: 82,
        b: 126,
        a: 255,
    },
    world_bg_floor_fov_false: ui::Rgba {
        r: 80,
        g: 25,
        b: 45,
        a: 255,
    },
    world_fg_floor_fov_false: ui::Rgba {
        r: 80,
        g: 25,
        b: 35,
        a: 255,
    },
    world_bg_floor_fov_true: ui::Rgba {
        r: 124,
        g: 18,
        b: 64,
        a: 255,
    },
    world_fg_floor_fov_true: ui::Rgba {
        r: 206,
        g: 82,
        b: 126,
        a: 255,
    },

    // entity colors
    entity_player: ui::Rgba {
        r: 170,
        g: 170,
        b: 170,
        a: 255,
    },
    entity_plasmid: ui::Rgba {
        r: 50,
        g: 50,
        b: 250,
        a: 255,
    },
    entity_virus: ui::Rgba {
        r: 100,
        g: 255,
        b: 150,
        a: 255,
    },
    entity_bacteria: ui::Rgba {
        r: 80,
        g: 235,
        b: 120,
        a: 255,
    },
};
